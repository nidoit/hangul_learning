# Architecture Design Document (ADD)

## DocConvert — Cross-Platform Document Converter

| Field          | Value          |
| -------------- | -------------- |
| Version        | 1.0            |
| Date           | 2026-02-28     |
| Status         | Draft          |
| Reference      | PRD v1.0, SRS v1.0 |

---

## 1. Introduction

### 1.1 Purpose

This document describes the software architecture of DocConvert, covering component decomposition, data flow, technology stack decisions, and deployment strategy.

### 1.2 Architectural Goals

| Goal | Description |
|------|-------------|
| Separation of concerns | Frontend handles UI/UX; Rust backend handles all file I/O and conversion logic. |
| Testability | Conversion logic is decoupled from Tauri commands so it can be unit-tested independently. |
| Cross-platform | A single codebase produces binaries for Linux (Arch, Debian, Fedora) and Windows. |
| Performance | Conversions run on native Rust threads; UI remains responsive via async IPC. |
| Minimal footprint | No bundled runtime beyond the OS WebView; small binary size. |

---

## 2. System Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                        DocConvert Application                    │
│                                                                  │
│  ┌──────────────────────────┐    ┌────────────────────────────┐ │
│  │       Frontend Layer      │    │       Backend Layer         │ │
│  │      (Tauri WebView)      │    │         (Rust)             │ │
│  │                           │    │                            │ │
│  │  ┌─────────────────────┐  │    │  ┌──────────────────────┐ │ │
│  │  │    UI Components    │  │    │  │   Tauri Commands     │ │ │
│  │  │  - FolderPicker     │  │IPC │  │  - select_folder     │ │ │
│  │  │  - FileList         │◄─┼────┼─►│  - scan_files        │ │ │
│  │  │  - ProgressBar      │  │    │  │  - start_conversion  │ │ │
│  │  │  - Summary          │  │    │  │  - get_summary       │ │ │
│  │  └─────────────────────┘  │    │  └──────────┬───────────┘ │ │
│  │                           │    │             │              │ │
│  │  ┌─────────────────────┐  │    │  ┌──────────▼───────────┐ │ │
│  │  │   Event Listener    │  │    │  │  Conversion Engine   │ │ │
│  │  │  (progress events)  │◄─┼────┼──│  - PdfConverter      │ │ │
│  │  └─────────────────────┘  │    │  │  - XlsxConverter     │ │ │
│  │                           │    │  │  - TableDetector     │ │ │
│  └──────────────────────────┘    │  │  - FileScanner       │ │ │
│                                   │  └──────────┬───────────┘ │ │
│                                   │             │              │ │
│                                   │  ┌──────────▼───────────┐ │ │
│                                   │  │    File System I/O   │ │ │
│                                   │  │  - Read PDF/XLSX     │ │ │
│                                   │  │  - Write MD/CSV      │ │ │
│                                   │  └──────────────────────┘ │ │
│                                   └────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

---

## 3. Component Architecture

### 3.1 Component Diagram

```
src-tauri/src/
├── main.rs                 # Tauri application entry point
├── commands/
│   ├── mod.rs              # Command module exports
│   ├── folder.rs           # select_folder, scan_files commands
│   └── convert.rs          # start_conversion, get_summary commands
├── converter/
│   ├── mod.rs              # Converter trait + shared types
│   ├── pdf.rs              # PdfConverter implementation
│   ├── xlsx.rs             # XlsxConverter implementation
│   └── table_detect.rs     # Table detection and MD table formatting
├── scanner/
│   ├── mod.rs              # FileScanner — recursive file discovery
│   └── types.rs            # FileEntry, FileType enums
├── models/
│   ├── mod.rs              # Shared data types
│   ├── job.rs              # ConversionJob
│   ├── result.rs           # ConversionResult
│   └── progress.rs         # ProgressEvent
└── error.rs                # AppError type, From impls

src/ (frontend)
├── index.html              # Main HTML shell
├── styles.css              # Application styles
├── main.js                 # Entry point
├── components/
│   ├── folder-picker.js    # Folder selection UI
│   ├── file-list.js        # Discovered files display
│   ├── progress-bar.js     # Conversion progress
│   └── summary.js          # Completion report
└── api.js                  # Tauri invoke wrappers
```

### 3.2 Component Responsibilities

#### 3.2.1 Frontend Layer

| Component | Responsibility |
|-----------|---------------|
| `folder-picker` | Triggers Tauri `dialog.open` for folder selection; displays selected paths. |
| `file-list` | Renders the list of discovered PDF/XLSX files with counts. |
| `progress-bar` | Listens to `conversion-progress` Tauri events; updates bar and current file name. |
| `summary` | Displays results after conversion; provides clipboard copy. |
| `api.js` | Thin wrapper around `window.__TAURI__.invoke()` calls for type safety and consistency. |

#### 3.2.2 Backend Layer — Commands

| Command | Responsibility |
|---------|---------------|
| `select_folder` | Opens native folder dialog via `tauri::api::dialog`. Returns selected path. |
| `scan_files` | Accepts source path, recursively walks directory, returns `Vec<FileEntry>`. |
| `start_conversion` | Accepts `ConversionJob`, spawns conversion on a background thread, emits progress events. |
| `get_summary` | Returns the `Vec<ConversionResult>` from the last completed job. |

#### 3.2.3 Backend Layer — Conversion Engine

| Module | Responsibility |
|--------|---------------|
| `PdfConverter` | Opens a PDF file, extracts text page-by-page, detects tables, produces Markdown. |
| `XlsxConverter` | Opens an XLSX workbook, iterates sheets and rows, writes CSV per sheet. |
| `TableDetector` | Analyzes text blocks for tabular patterns (aligned columns, grid lines) and outputs Markdown table syntax. |
| `FileScanner` | Uses `walkdir` crate to recursively find `.pdf` and `.xlsx` files. |

---

## 4. Data Flow

### 4.1 Conversion Flow

```
User clicks "Select Folder"
        │
        ▼
Frontend ──invoke──► select_folder command
        │                    │
        │              Opens OS dialog
        │                    │
        ◄────── path ────────┘
        │
        ▼
Frontend ──invoke──► scan_files(path)
        │                    │
        │            walkdir recursive scan
        │            filter .pdf, .xlsx
        │                    │
        ◄─── Vec<FileEntry> ─┘
        │
Display file counts
        │
User clicks "Convert"
        │
        ▼
Frontend ──invoke──► start_conversion(job)
                             │
                    ┌────────▼────────┐
                    │  Spawn async    │
                    │  task on pool   │
                    └────────┬────────┘
                             │
              ┌──────────────┼──────────────┐
              ▼              ▼              ▼
         Process         Process       Process
         file 1          file 2        file N
              │              │              │
              │    emit ProgressEvent       │
              ├──────────────┼──────────────┤
              ▼              ▼              ▼
         Write .md       Write .csv    Write .md
              │              │              │
              └──────────────┼──────────────┘
                             │
                    Collect ConversionResults
                             │
                    emit "conversion-complete"
                             │
Frontend ◄─── event ─────────┘
        │
Display summary
```

### 4.2 Event Flow (Progress Reporting)

```
Backend                                    Frontend
   │                                          │
   │  emit("conversion-progress", {           │
   │    current: 3,                           │
   │    total: 17,                            │
   │    current_file: "report.pdf"            │
   │  })                                      │
   │ ────────────────────────────────────────► │
   │                                          │
   │                              Update progress bar
   │                              Update file label
```

---

## 5. Technology Stack

### 5.1 Decisions and Rationale

| Layer | Technology | Rationale |
|-------|-----------|-----------|
| Desktop framework | **Tauri v2** | Lightweight (no Electron/Chromium); native WebView; Rust backend; built-in packaging for .deb, .rpm, .msi. |
| Backend language | **Rust** | Memory safety, performance, required by Tauri. |
| Frontend | **Vanilla HTML/CSS/JS** | No build step needed; minimal bundle; sufficient for a simple single-page UI. Can upgrade to Svelte/React later. |
| PDF text extraction | **pdf-extract** | Pure Rust; extracts text with position data (needed for table detection). |
| PDF low-level access | **lopdf** | Pure Rust PDF parser; provides page-level access for font/size heuristics. |
| XLSX parsing | **calamine** | Mature Rust crate; reads .xlsx without external dependencies; supports all cell types. |
| CSV writing | **csv** (crate) | Standard Rust CSV writer; RFC 4180 compliant; handles escaping. |
| Directory walking | **walkdir** | Efficient recursive directory traversal; handles symlinks and permissions. |
| Async runtime | **tokio** (via Tauri) | Tauri v2 uses tokio internally; async commands run on the tokio runtime. |
| Serialization | **serde + serde_json** | Standard Rust serialization for IPC data transfer between frontend and backend. |

### 5.2 Dependency Table

| Crate | Version (min) | License | Purpose |
|-------|--------------|---------|---------|
| tauri | 2.x | MIT/Apache-2.0 | Desktop framework |
| pdf-extract | 0.7+ | Apache-2.0 | PDF text extraction |
| lopdf | 0.32+ | MIT | PDF parsing |
| calamine | 0.25+ | MIT | XLSX reading |
| csv | 1.3+ | MIT/Unlicense | CSV writing |
| walkdir | 2.x | MIT/Unlicense | Recursive directory traversal |
| serde | 1.x | MIT/Apache-2.0 | Serialization framework |
| serde_json | 1.x | MIT/Apache-2.0 | JSON serialization |
| chrono | 0.4+ | MIT/Apache-2.0 | Date formatting for XLSX dates |
| thiserror | 1.x | MIT/Apache-2.0 | Error type derivation |
| log | 0.4+ | MIT/Apache-2.0 | Logging facade |
| env_logger | 0.11+ | MIT/Apache-2.0 | Log output |

---

## 6. Module Design

### 6.1 Converter Trait

```rust
pub trait Converter {
    type Input;
    type Output;
    type Error;

    fn convert(&self, input: Self::Input) -> Result<Self::Output, Self::Error>;
}
```

Both `PdfConverter` and `XlsxConverter` implement this trait, enabling uniform handling in the conversion pipeline.

### 6.2 PdfConverter Pipeline

```
PDF bytes
   │
   ▼
lopdf::Document::load()
   │
   ▼
For each page:
   ├── Extract text with positions (pdf-extract)
   ├── Group text into lines (by Y-coordinate)
   ├── Detect headings (font size heuristic)
   ├── Detect tables (column alignment analysis)
   │      └── TableDetector::detect(lines) → Option<MarkdownTable>
   └── Emit Markdown paragraph / heading / table
   │
   ▼
Concatenate pages → single .md String
   │
   ▼
Write to output file (UTF-8)
```

### 6.3 XlsxConverter Pipeline

```
XLSX file path
   │
   ▼
calamine::open_workbook_auto()
   │
   ▼
For each sheet:
   ├── Read sheet name
   ├── Iterate rows
   │     ├── For each cell:
   │     │    ├── String → as-is
   │     │    ├── Float → format with minimal decimals
   │     │    ├── DateTime → chrono format ISO 8601
   │     │    ├── Bool → "true" / "false"
   │     │    └── Empty → ""
   │     └── Join cells with comma, RFC 4180 escaping
   ├── Write CSV rows to output file
   └── UTF-8 encoding, no BOM
```

### 6.4 Table Detection Algorithm

```
Input: Vec<TextLine> (each line has text content + character positions)

1. For each consecutive group of lines:
   a. Compute column boundaries by finding consistent whitespace gaps
      across ≥ 3 consecutive lines.
   b. If ≥ 2 columns detected with ≥ 2 rows → mark as table region.

2. For detected table regions:
   a. Split each line into cells based on column boundaries.
   b. Trim whitespace from each cell.
   c. Format as Markdown:
      | Cell1 | Cell2 | Cell3 |
      |-------|-------|-------|
      | val1  | val2  | val3  |

3. Lines with horizontal rules (----, ====, ____) adjacent to
   aligned text reinforce table detection confidence.

4. Return: list of (line_range, MarkdownTable) pairs.
```

---

## 7. Error Handling Strategy

```
                    AppError (thiserror enum)
                         │
            ┌────────────┼────────────┐
            ▼            ▼            ▼
      PdfError      XlsxError     IoError
      │               │              │
      ├─ ParseFailed  ├─ OpenFailed  ├─ ReadFailed
      ├─ NoText       ├─ SheetError  ├─ WriteFailed
      └─ EncodingErr  └─ CellError   └─ PermissionDenied
```

- All errors implement `Display` for user-facing messages.
- All errors implement `serde::Serialize` so they can cross the IPC boundary.
- File-level errors are captured in `ConversionResult`; they do not abort the batch.

---

## 8. Concurrency Model

```
Main Thread (Tauri)
   │
   ├── Handles IPC commands
   ├── Manages application lifecycle
   │
   └── start_conversion command:
          │
          ▼
       Spawn tokio::task::spawn_blocking
          │
          ▼
       Sequential file processing *
          │
          ├── Convert file N
          ├── Emit progress event via AppHandle
          └── Repeat
          │
          ▼
       Send "conversion-complete" event

* Sequential processing is chosen for v1.0 to avoid memory
  pressure from multiple large PDFs loaded simultaneously.
  Future versions may add parallel conversion with a configurable
  concurrency limit (e.g., num_cpus::get()).
```

---

## 9. Packaging & Distribution

### 9.1 Build Targets

| Platform | Package Format | Build Tool | Notes |
|----------|---------------|------------|-------|
| Arch Linux | PKGBUILD / tar.gz | `tauri build` + custom PKGBUILD | Can publish to AUR |
| Debian/Ubuntu | .deb | `tauri build --bundles deb` | Built-in Tauri bundler |
| Fedora/RHEL | .rpm | `tauri build --bundles rpm` | Built-in Tauri bundler |
| Windows | .msi, .exe | `tauri build --bundles msi,nsis` | Built-in Tauri bundler |

### 9.2 CI/CD Pipeline

```
Push to main / PR
       │
       ▼
┌──────────────────┐
│  Lint + Format   │  cargo fmt --check && cargo clippy
└───────┬──────────┘
        │
        ▼
┌──────────────────┐
│   Unit Tests     │  cargo test
└───────┬──────────┘
        │
        ▼
┌──────────────────────────────────────────┐
│          Build Matrix                     │
│  ┌────────┐ ┌────────┐ ┌──────────────┐ │
│  │Linux   │ │Windows │ │ Linux (ARM)  │ │
│  │x86_64  │ │x86_64  │ │ aarch64     │ │
│  │.deb    │ │.msi    │ │ .deb        │ │
│  │.rpm    │ │.exe    │ │             │ │
│  └────────┘ └────────┘ └──────────────┘ │
└───────────────────┬──────────────────────┘
                    │
                    ▼
           Upload artifacts / Release
```

---

## 10. Security Considerations

| Concern | Mitigation |
|---------|------------|
| Arbitrary file read | Tauri scope restricts file access to user-selected directories only via `fs` scope. |
| Path traversal | All paths are canonicalized and validated to be within the source/output directories. |
| Malicious PDFs | Rust memory safety prevents buffer overflows; `lopdf` and `pdf-extract` do not execute embedded JS. |
| XSS in WebView | Tauri CSP: `default-src 'self'; script-src 'self'`. No inline scripts. No external resources. |
| Supply chain | Crate versions pinned in `Cargo.lock`; `cargo audit` run in CI. |

---

## 11. Future Considerations (Post v1.0)

| Feature | Notes |
|---------|-------|
| OCR support | Integrate Tesseract via `leptess` crate for scanned PDFs. |
| Parallel conversion | Process multiple files concurrently with a configurable thread pool. |
| DOCX support | Add `docx-rs` crate for Word document conversion. |
| Drag-and-drop | Accept dropped folders/files in the WebView. |
| CLI mode | Expose conversion engine as a standalone CLI binary (no GUI). |
| i18n UI | Translate UI strings into multiple languages. |
