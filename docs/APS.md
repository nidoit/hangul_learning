# API Specification (APS)

## DocConvert — Tauri Command & IPC Interface

| Field          | Value          |
| -------------- | -------------- |
| Version        | 1.0            |
| Date           | 2026-02-28     |
| Status         | Draft          |
| Reference      | PRD v1.0, SRS v1.0, ADD v1.0 |

---

## 1. Overview

DocConvert uses Tauri's IPC mechanism to communicate between the frontend (WebView) and the Rust backend. The frontend invokes **Tauri commands** via `window.__TAURI__.invoke()`, and the backend emits **Tauri events** for real-time progress updates.

This document defines every command and event, including input schemas, output schemas, and error responses.

---

## 2. Communication Model

```
Frontend (JS)                           Backend (Rust)
     │                                       │
     │── invoke("command", {args}) ────────►│
     │                                       │── Process
     │◄──────────── Result<T, E> ───────────│
     │                                       │
     │◄──── emit("event-name", payload) ────│  (async events)
     │                                       │
```

- **Commands** are request-response (invoke → return).
- **Events** are fire-and-forget from backend to frontend (one-way push).

---

## 3. Tauri Commands

### 3.1 `select_folder`

Opens the native OS folder picker dialog and returns the selected path.

**Rust Signature:**
```rust
#[tauri::command]
async fn select_folder(app: AppHandle) -> Result<Option<String>, AppError>
```

**Frontend Invocation:**
```javascript
const path = await invoke("select_folder");
```

**Input:** None

**Output:**

| Field | Type | Description |
|-------|------|-------------|
| (return) | `Option<String>` | Absolute path to the selected folder, or `null` if the user cancelled. |

**Errors:**

| Code | Message | Cause |
|------|---------|-------|
| `DIALOG_FAILED` | "Failed to open folder dialog" | OS dialog API error. |

---

### 3.2 `scan_files`

Recursively scans a directory for supported files (.pdf, .xlsx).

**Rust Signature:**
```rust
#[tauri::command]
async fn scan_files(source_dir: String) -> Result<ScanResult, AppError>
```

**Frontend Invocation:**
```javascript
const result = await invoke("scan_files", { sourceDir: "/home/user/docs" });
```

**Input:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `sourceDir` | `string` | Yes | Absolute path to the folder to scan. |

**Output Schema — `ScanResult`:**

```json
{
  "pdf_count": 12,
  "xlsx_count": 5,
  "total_size_bytes": 48923847,
  "files": [
    {
      "path": "/home/user/docs/report.pdf",
      "relative_path": "report.pdf",
      "file_type": "Pdf",
      "size_bytes": 1024567
    },
    {
      "path": "/home/user/docs/data/sales.xlsx",
      "relative_path": "data/sales.xlsx",
      "file_type": "Xlsx",
      "size_bytes": 234891
    }
  ]
}
```

| Field | Type | Description |
|-------|------|-------------|
| `pdf_count` | `u32` | Number of PDF files found. |
| `xlsx_count` | `u32` | Number of XLSX files found. |
| `total_size_bytes` | `u64` | Combined size of all discovered files. |
| `files` | `Vec<FileEntry>` | List of discovered files. |

**`FileEntry` Schema:**

| Field | Type | Description |
|-------|------|-------------|
| `path` | `String` | Absolute path to the file. |
| `relative_path` | `String` | Path relative to the source directory. |
| `file_type` | `"Pdf" \| "Xlsx"` | Type of the file. |
| `size_bytes` | `u64` | File size in bytes. |

**Errors:**

| Code | Message | Cause |
|------|---------|-------|
| `DIR_NOT_FOUND` | "Directory does not exist: {path}" | The provided path doesn't exist. |
| `DIR_NOT_READABLE` | "Cannot read directory: {path}" | Insufficient permissions. |
| `SCAN_FAILED` | "Error scanning directory: {detail}" | I/O error during traversal. |

---

### 3.3 `start_conversion`

Begins batch conversion of all discovered files. Runs asynchronously and emits progress events.

**Rust Signature:**
```rust
#[tauri::command]
async fn start_conversion(
    app: AppHandle,
    job: ConversionJob,
) -> Result<Vec<ConversionResult>, AppError>
```

**Frontend Invocation:**
```javascript
const results = await invoke("start_conversion", {
  job: {
    source_dir: "/home/user/docs",
    output_dir: "/home/user/docs/output",
    overwrite: false,
    files: [/* FileEntry array from scan_files */]
  }
});
```

**Input Schema — `ConversionJob`:**

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `source_dir` | `String` | Yes | — | Absolute path to the source folder. |
| `output_dir` | `String` | Yes | — | Absolute path to the output folder. |
| `overwrite` | `bool` | No | `false` | Whether to overwrite existing output files. |
| `files` | `Vec<FileEntry>` | Yes | — | Files to convert (from `scan_files` output). |

**Output Schema — `Vec<ConversionResult>`:**

```json
[
  {
    "file": {
      "path": "/home/user/docs/report.pdf",
      "relative_path": "report.pdf",
      "file_type": "Pdf",
      "size_bytes": 1024567
    },
    "status": "Success",
    "output_path": "/home/user/docs/output/report.md",
    "error": null,
    "duration_ms": 1523
  },
  {
    "file": {
      "path": "/home/user/docs/corrupt.pdf",
      "relative_path": "corrupt.pdf",
      "file_type": "Pdf",
      "size_bytes": 512
    },
    "status": "Failed",
    "output_path": null,
    "error": "Invalid PDF structure: missing xref table",
    "duration_ms": 45
  }
]
```

**`ConversionResult` Schema:**

| Field | Type | Description |
|-------|------|-------------|
| `file` | `FileEntry` | The source file. |
| `status` | `"Success" \| "Failed"` | Conversion outcome. |
| `output_path` | `Option<String>` | Path to the generated file (null if failed). |
| `error` | `Option<String>` | Error message (null if successful). |
| `duration_ms` | `u64` | Time taken to convert this file in milliseconds. |

**Errors:**

| Code | Message | Cause |
|------|---------|-------|
| `OUTPUT_NOT_WRITABLE` | "Output directory is not writable: {path}" | Cannot create files in the output directory. |
| `CONVERSION_ABORTED` | "Conversion was cancelled by user" | User cancelled mid-conversion (future feature). |

---

### 3.4 `get_summary`

Returns the results of the most recent conversion job. Useful if the frontend needs to re-render after a page refresh.

**Rust Signature:**
```rust
#[tauri::command]
async fn get_summary(state: State<'_, AppState>) -> Result<Option<Vec<ConversionResult>>, AppError>
```

**Frontend Invocation:**
```javascript
const summary = await invoke("get_summary");
```

**Input:** None

**Output:**

| Field | Type | Description |
|-------|------|-------------|
| (return) | `Option<Vec<ConversionResult>>` | Array of results from the last job, or `null` if no conversion has been run. |

**Errors:** None (always succeeds).

---

## 4. Tauri Events (Backend → Frontend)

### 4.1 `conversion-progress`

Emitted after each file is processed during a conversion job.

**Rust Emission:**
```rust
app.emit("conversion-progress", ProgressEvent {
    current: 3,
    total: 17,
    current_file: "data/sales.xlsx".to_string(),
    status: "processing".to_string(),
})?;
```

**Frontend Listener:**
```javascript
import { listen } from "@tauri-apps/api/event";

const unlisten = await listen("conversion-progress", (event) => {
  const { current, total, current_file, status } = event.payload;
  updateProgressBar(current, total);
  updateCurrentFile(current_file);
});
```

**Payload Schema — `ProgressEvent`:**

| Field | Type | Description |
|-------|------|-------------|
| `current` | `u32` | Number of files processed so far (1-indexed). |
| `total` | `u32` | Total number of files to process. |
| `current_file` | `String` | Relative path of the file currently being processed. |
| `status` | `String` | One of: `"processing"`, `"completed"`, `"failed"`. |

---

### 4.2 `conversion-complete`

Emitted once when the entire batch conversion finishes.

**Rust Emission:**
```rust
app.emit("conversion-complete", CompletionEvent {
    total: 17,
    succeeded: 15,
    failed: 2,
    duration_ms: 34521,
})?;
```

**Frontend Listener:**
```javascript
const unlisten = await listen("conversion-complete", (event) => {
  const { total, succeeded, failed, duration_ms } = event.payload;
  showSummary(event.payload);
});
```

**Payload Schema — `CompletionEvent`:**

| Field | Type | Description |
|-------|------|-------------|
| `total` | `u32` | Total files processed. |
| `succeeded` | `u32` | Number of successful conversions. |
| `failed` | `u32` | Number of failed conversions. |
| `duration_ms` | `u64` | Total elapsed time in milliseconds. |

---

### 4.3 `conversion-error`

Emitted when a single file fails during conversion (informational; does not stop the batch).

**Payload Schema — `FileErrorEvent`:**

| Field | Type | Description |
|-------|------|-------------|
| `file` | `String` | Relative path of the failed file. |
| `error` | `String` | Human-readable error message. |

---

## 5. Rust Type Definitions

Complete type definitions for the IPC interface:

```rust
use serde::{Deserialize, Serialize};

// ── Enums ──────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileType {
    Pdf,
    Xlsx,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConversionStatus {
    Success,
    Failed,
}

// ── Structs ────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: String,
    pub relative_path: String,
    pub file_type: FileType,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub pdf_count: u32,
    pub xlsx_count: u32,
    pub total_size_bytes: u64,
    pub files: Vec<FileEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionJob {
    pub source_dir: String,
    pub output_dir: String,
    #[serde(default)]
    pub overwrite: bool,
    pub files: Vec<FileEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionResult {
    pub file: FileEntry,
    pub status: ConversionStatus,
    pub output_path: Option<String>,
    pub error: Option<String>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressEvent {
    pub current: u32,
    pub total: u32,
    pub current_file: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionEvent {
    pub total: u32,
    pub succeeded: u32,
    pub failed: u32,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileErrorEvent {
    pub file: String,
    pub error: String,
}

// ── Error Type ─────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Directory not found: {0}")]
    DirNotFound(String),

    #[error("Directory not readable: {0}")]
    DirNotReadable(String),

    #[error("Output directory not writable: {0}")]
    OutputNotWritable(String),

    #[error("Scan failed: {0}")]
    ScanFailed(String),

    #[error("PDF conversion error: {0}")]
    PdfError(String),

    #[error("XLSX conversion error: {0}")]
    XlsxError(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Dialog failed: {0}")]
    DialogFailed(String),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
```

---

## 6. Frontend API Wrapper

Recommended `api.js` module for the frontend:

```javascript
const { invoke } = window.__TAURI__.core;

/**
 * Open native folder picker.
 * @returns {Promise<string|null>} Selected folder path, or null if cancelled.
 */
export async function selectFolder() {
  return await invoke("select_folder");
}

/**
 * Scan a directory for PDF and XLSX files.
 * @param {string} sourceDir - Absolute path to scan.
 * @returns {Promise<ScanResult>}
 */
export async function scanFiles(sourceDir) {
  return await invoke("scan_files", { sourceDir });
}

/**
 * Start batch conversion.
 * @param {ConversionJob} job
 * @returns {Promise<ConversionResult[]>}
 */
export async function startConversion(job) {
  return await invoke("start_conversion", { job });
}

/**
 * Get results from the last conversion run.
 * @returns {Promise<ConversionResult[]|null>}
 */
export async function getSummary() {
  return await invoke("get_summary");
}
```

---

## 7. IPC Security

### 7.1 Tauri Capability Configuration

The following capabilities must be declared in `src-tauri/capabilities/default.json`:

```json
{
  "identifier": "default",
  "description": "Default capability for DocConvert",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "dialog:default",
    "dialog:allow-open",
    "event:default",
    "event:allow-listen",
    "event:allow-emit"
  ]
}
```

### 7.2 Content Security Policy

In `tauri.conf.json`:

```json
{
  "app": {
    "security": {
      "csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'"
    }
  }
}
```

### 7.3 Input Validation

All commands SHALL validate inputs before processing:

| Command | Validation |
|---------|-----------|
| `scan_files` | Verify `source_dir` exists and is a directory. |
| `start_conversion` | Verify `source_dir` exists; verify `output_dir` is writable or can be created; validate all `files` entries exist. |

---

## 8. Error Response Format

All Tauri command errors are serialized as strings via the `AppError` implementation. The frontend receives errors in the standard Tauri rejection format:

```javascript
try {
  const result = await invoke("scan_files", { sourceDir: "/nonexistent" });
} catch (error) {
  // error is a string: "Directory not found: /nonexistent"
  showError(error);
}
```

For richer error handling in future versions, the error format can be extended to:

```json
{
  "code": "DIR_NOT_FOUND",
  "message": "Directory not found: /nonexistent",
  "detail": null
}
```

---

## 9. Sequence Diagrams

### 9.1 Full Conversion Flow

```
User        Frontend         Backend            FileSystem
 │              │                │                   │
 │─ click ─────►│                │                   │
 │              │── select_folder ──►│               │
 │              │                │── open dialog ───►│
 │              │                │◄── path ─────────│
 │              │◄── "/docs" ───│                   │
 │              │                │                   │
 │              │── scan_files ────►│                │
 │              │                │── walkdir ───────►│
 │              │                │◄── entries ──────│
 │              │◄── ScanResult ─│                   │
 │              │                │                   │
 │◄── show counts               │                   │
 │              │                │                   │
 │─ click ─────►│                │                   │
 │              │── start_conversion ►│              │
 │              │                │── read PDF ──────►│
 │              │                │◄── bytes ────────│
 │              │◄── progress ──│                   │
 │              │                │── write .md ─────►│
 │              │◄── progress ──│                   │
 │              │                │── read XLSX ─────►│
 │              │                │◄── bytes ────────│
 │              │                │── write .csv ────►│
 │              │◄── complete ──│                   │
 │              │◄── results ───│                   │
 │              │                │                   │
 │◄── show summary              │                   │
 │              │                │                   │
```
