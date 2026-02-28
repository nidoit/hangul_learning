# Software Requirements Specification (SRS)

## DocConvert — Cross-Platform Document Converter

| Field          | Value          |
| -------------- | -------------- |
| Version        | 1.0            |
| Date           | 2026-02-28     |
| Status         | Draft          |
| Reference      | PRD v1.0       |

---

## 1. Introduction

### 1.1 Purpose

This SRS defines the functional and non-functional requirements for DocConvert, a Tauri-based desktop application that converts PDF documents to Markdown and XLSX spreadsheets to CSV.

### 1.2 Scope

The system receives a user-selected folder path, recursively discovers PDF and XLSX files, converts them to Markdown and CSV respectively, and writes the output to a user-specified (or default) output directory.

### 1.3 Definitions & Acronyms

| Term | Definition |
|------|------------|
| PDF  | Portable Document Format |
| MD   | Markdown (CommonMark-compatible) |
| XLSX | Office Open XML Spreadsheet |
| CSV  | Comma-Separated Values |
| IPC  | Inter-Process Communication (Tauri command bridge) |
| CJK  | Chinese, Japanese, Korean character sets |
| BOM  | Byte Order Mark |

### 1.4 References

- PRD v1.0 — DocConvert Product Requirements Document
- Tauri v2 documentation: https://v2.tauri.app
- CommonMark specification: https://spec.commonmark.org

---

## 2. Overall Description

### 2.1 System Context

```
┌──────────────────────────────────────────────┐
│                  User (Desktop)               │
│                                               │
│   ┌───────────┐         ┌──────────────────┐ │
│   │  Frontend  │◄──IPC──►│   Rust Backend   │ │
│   │  (WebView) │         │  (Tauri Commands)│ │
│   └───────────┘         └────────┬─────────┘ │
│                                  │            │
│                          ┌───────▼────────┐   │
│                          │  Local FS      │   │
│                          │  (PDF/XLSX →   │   │
│                          │   MD/CSV)      │   │
│                          └────────────────┘   │
└──────────────────────────────────────────────┘
```

### 2.2 User Characteristics

Users are expected to be comfortable with basic desktop application usage (file dialogs, buttons). No programming knowledge is required. Users may work with documents in any language supported by UTF-8.

### 2.3 Operating Environment

| Requirement  | Detail |
|-------------|--------|
| OS          | Arch Linux, Debian 11+/Ubuntu 22.04+, Fedora 38+, Windows 10/11 |
| Runtime     | Tauri v2 WebView (WebKitGTK on Linux, WebView2 on Windows) |
| Disk        | 50 MB for installation + space for converted files |
| Memory      | 512 MB minimum; 2 GB recommended for large batches |

---

## 3. Functional Requirements

### 3.1 Folder Selection

| ID    | Requirement |
|-------|-------------|
| FR-01 | The system SHALL display a native OS folder picker dialog when the user clicks "Select Folder". |
| FR-02 | The system SHALL display the selected folder path in the UI after selection. |
| FR-03 | The system SHALL allow the user to optionally select a separate output directory. |
| FR-04 | If no output directory is selected, the system SHALL default to creating an `output/` subdirectory inside the selected source folder. |

### 3.2 File Discovery

| ID    | Requirement |
|-------|-------------|
| FR-05 | The system SHALL recursively scan the selected folder for files with `.pdf` extension (case-insensitive). |
| FR-06 | The system SHALL recursively scan the selected folder for files with `.xlsx` extension (case-insensitive). |
| FR-07 | The system SHALL display the count of discovered PDF and XLSX files before conversion begins. |
| FR-08 | The system SHALL preserve the relative directory structure from the source folder in the output directory. |

### 3.3 PDF to Markdown Conversion

| ID    | Requirement |
|-------|-------------|
| FR-09  | The system SHALL extract text content from each PDF file and write it as a `.md` file. |
| FR-10  | The system SHALL detect tabular structures in PDFs and convert them to Markdown table syntax using pipe (`\|`) and hyphen (`-`) delimiters. |
| FR-11  | The system SHALL preserve paragraph breaks as double newlines in Markdown output. |
| FR-12  | The system SHALL preserve heading hierarchy where detectable (based on font size heuristics). |
| FR-13  | The system SHALL handle multi-page PDFs, producing a single Markdown file per PDF. |
| FR-14  | The system SHALL correctly encode all UTF-8 characters in the output, including CJK, Cyrillic, Arabic, and Latin-extended scripts. |
| FR-15  | The system SHALL name output files as `<original_name>.md` (e.g., `report.pdf` → `report.md`). |

### 3.4 XLSX to CSV Conversion

| ID    | Requirement |
|-------|-------------|
| FR-16 | The system SHALL convert each sheet in an XLSX file to a separate CSV file. |
| FR-17 | Multi-sheet workbooks SHALL produce files named `<original_name>_<sheet_name>.csv`. |
| FR-18 | Single-sheet workbooks SHALL produce files named `<original_name>.csv`. |
| FR-19 | The system SHALL write CSV output in UTF-8 encoding without BOM. |
| FR-20 | The system SHALL properly escape CSV fields containing commas, double quotes, or newlines per RFC 4180. |
| FR-21 | The system SHALL convert date-typed cells to ISO 8601 format (`YYYY-MM-DD`). |
| FR-22 | The system SHALL evaluate formula cells and output the computed value (not the formula string). |

### 3.5 Progress & Reporting

| ID    | Requirement |
|-------|-------------|
| FR-23 | The system SHALL display a progress bar showing `N / Total` files processed. |
| FR-24 | The system SHALL display the name of the file currently being converted. |
| FR-25 | The system SHALL emit real-time progress events from the backend to the frontend via Tauri events. |
| FR-26 | Upon completion, the system SHALL display a summary showing: total files processed, successful conversions, failed conversions, and a list of failed files with error reasons. |
| FR-27 | The system SHALL allow the user to copy the summary report to the clipboard. |

### 3.6 Error Handling

| ID    | Requirement |
|-------|-------------|
| FR-28 | If a PDF file cannot be parsed, the system SHALL skip that file, log the error, and continue processing remaining files. |
| FR-29 | If an XLSX file cannot be parsed, the system SHALL skip that file, log the error, and continue processing remaining files. |
| FR-30 | If the output directory is not writable, the system SHALL display an error message before starting conversion. |
| FR-31 | The system SHALL not overwrite existing output files unless the user enables an "Overwrite existing" option. |

---

## 4. Non-Functional Requirements

### 4.1 Performance

| ID     | Requirement |
|--------|-------------|
| NFR-01 | A 100-page text-only PDF SHALL convert to Markdown in ≤ 30 seconds on a 4-core, 8 GB RAM machine. |
| NFR-02 | An XLSX file with 100,000 rows and 20 columns SHALL convert to CSV in ≤ 10 seconds. |
| NFR-03 | Application cold-start (launch to interactive UI) SHALL complete in ≤ 3 seconds. |
| NFR-04 | Memory usage SHALL not exceed 500 MB when converting a single 200-page PDF. |

### 4.2 Reliability

| ID     | Requirement |
|--------|-------------|
| NFR-05 | A single file failure SHALL NOT abort the entire batch conversion. |
| NFR-06 | The application SHALL handle filenames containing Unicode characters, spaces, and special characters. |
| NFR-07 | The application SHALL handle deeply nested directory structures (≥ 20 levels). |

### 4.3 Usability

| ID     | Requirement |
|--------|-------------|
| NFR-08 | The UI SHALL be usable without a manual — all actions reachable in ≤ 3 clicks from launch. |
| NFR-09 | The UI SHALL provide clear status messages in English. |
| NFR-10 | Error messages SHALL include the file path and a human-readable reason. |

### 4.4 Portability

| ID     | Requirement |
|--------|-------------|
| NFR-11 | The application SHALL build from a single codebase for all four target platforms. |
| NFR-12 | Platform-specific behavior (file dialogs, paths) SHALL be abstracted by the Tauri framework. |
| NFR-13 | No platform-specific code SHALL exist outside of Tauri configuration and build scripts. |

### 4.5 Security

| ID     | Requirement |
|--------|-------------|
| NFR-14 | The application SHALL NOT make any network requests. |
| NFR-15 | File system access SHALL be limited to the user-selected source and output directories. |
| NFR-16 | The Tauri CSP (Content Security Policy) SHALL disallow inline scripts and external resource loading. |

### 4.6 Maintainability

| ID     | Requirement |
|--------|-------------|
| NFR-17 | All Rust crate dependencies SHALL use permissive licenses (MIT, Apache-2.0, BSD). |
| NFR-18 | The project SHALL include CI/CD configuration for automated building and testing. |
| NFR-19 | Unit tests SHALL cover ≥ 80% of the Rust backend conversion logic. |

---

## 5. Use Cases

### UC-01: Convert PDFs in a Folder

| Field           | Detail |
|-----------------|--------|
| **Actor**       | User |
| **Precondition**| Application is running; folder contains `.pdf` files. |
| **Main Flow**   | 1. User clicks "Select Folder".<br>2. Native folder picker opens.<br>3. User selects a folder.<br>4. System scans folder recursively and displays file counts.<br>5. User clicks "Convert".<br>6. System converts each PDF to Markdown, showing progress.<br>7. System displays summary report. |
| **Alt Flow A**  | At step 4, no PDF or XLSX files found → system displays "No supported files found." |
| **Alt Flow B**  | At step 6, a PDF fails to parse → system logs error, skips file, continues. |
| **Postcondition** | Markdown files exist in the output directory mirroring the source structure. |

### UC-02: Convert XLSX Files in a Folder

| Field           | Detail |
|-----------------|--------|
| **Actor**       | User |
| **Precondition**| Application is running; folder contains `.xlsx` files. |
| **Main Flow**   | 1. User clicks "Select Folder".<br>2. User selects a folder.<br>3. System scans and displays file counts.<br>4. User clicks "Convert".<br>5. System converts each XLSX to CSV, showing progress.<br>6. System displays summary report. |
| **Alt Flow A**  | Multi-sheet workbook → system creates one CSV per sheet. |
| **Postcondition** | CSV files exist in the output directory. |

### UC-03: Choose Output Directory

| Field           | Detail |
|-----------------|--------|
| **Actor**       | User |
| **Precondition**| Source folder has been selected. |
| **Main Flow**   | 1. User clicks "Change Output Folder".<br>2. Native folder picker opens.<br>3. User selects an output directory.<br>4. System validates the directory is writable.<br>5. System updates the displayed output path. |
| **Alt Flow A**  | Directory is not writable → system shows error, retains previous output path. |
| **Postcondition** | Subsequent conversions write to the new output directory. |

---

## 6. Data Dictionary

| Entity          | Field           | Type     | Description |
|-----------------|-----------------|----------|-------------|
| ConversionJob   | source_dir      | PathBuf  | Absolute path to the source folder. |
| ConversionJob   | output_dir      | PathBuf  | Absolute path to the output folder. |
| ConversionJob   | overwrite       | bool     | Whether to overwrite existing output files. |
| FileEntry       | path            | PathBuf  | Absolute path to the discovered file. |
| FileEntry       | relative_path   | PathBuf  | Path relative to source_dir. |
| FileEntry       | file_type       | enum     | `Pdf` or `Xlsx`. |
| FileEntry       | size_bytes      | u64      | File size in bytes. |
| ConversionResult| file            | FileEntry| The file that was converted. |
| ConversionResult| status          | enum     | `Success` or `Failed`. |
| ConversionResult| error           | Option\<String\> | Error message if failed. |
| ConversionResult| output_path     | PathBuf  | Path to the generated output file. |
| ConversionResult| duration_ms     | u64      | Time taken to convert in milliseconds. |
| ProgressEvent   | current         | usize    | Number of files processed so far. |
| ProgressEvent   | total           | usize    | Total number of files to process. |
| ProgressEvent   | current_file    | String   | Name of the file being processed. |

---

## 7. Interface Requirements

### 7.1 UI Wireframe (Text)

```
┌─────────────────────────────────────────────────────┐
│  DocConvert                                    [—][X]│
├─────────────────────────────────────────────────────┤
│                                                     │
│  Source Folder:  /home/user/documents    [Browse...] │
│  Output Folder:  /home/user/documents/output [Change]│
│                                                     │
│  Found: 12 PDFs, 5 XLSX files                       │
│                                                     │
│  ☐ Overwrite existing files                         │
│                                                     │
│         [ ▶  Start Conversion ]                     │
│                                                     │
│  ┌─────────────────────────────────────────────┐    │
│  │  ████████████████░░░░░░░░░░  8 / 17         │    │
│  │  Converting: report_2026.pdf                 │    │
│  └─────────────────────────────────────────────┘    │
│                                                     │
│  ── Summary ──────────────────────────────────────  │
│  ✓ 15 succeeded   ✗ 2 failed                       │
│  Failed: corrupt.pdf (Invalid PDF structure)        │
│          empty.xlsx (No sheets found)               │
│                                           [Copy]    │
└─────────────────────────────────────────────────────┘
```

---

## 8. Constraints

| # | Constraint |
|---|-----------|
| C-1 | Must use Tauri v2 for the desktop framework. |
| C-2 | Backend must be written in Rust. |
| C-3 | No runtime dependencies beyond the OS WebView. |
| C-4 | No network access required or permitted. |
| C-5 | All third-party crates must be permissively licensed. |

---

## 9. Traceability Matrix

| PRD User Story | SRS Functional Req | Use Case |
|----------------|--------------------|----------|
| US-01 | FR-01, FR-02 | UC-01, UC-02 |
| US-02 | FR-05, FR-09, FR-13 | UC-01 |
| US-03 | FR-10 | UC-01 |
| US-04 | FR-14 | UC-01 |
| US-05 | FR-06, FR-16, FR-17, FR-18 | UC-02 |
| US-06 | FR-23, FR-24, FR-25 | UC-01, UC-02 |
| US-07 | FR-26, FR-27 | UC-01, UC-02 |
| US-08 | NFR-11 | — |
| US-09 | NFR-11 | — |
| US-10 | FR-03, FR-04 | UC-03 |
