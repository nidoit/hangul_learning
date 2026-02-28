# Product Requirements Document (PRD)

## DocConvert — Cross-Platform Document Converter

| Field          | Value                                      |
| -------------- | ------------------------------------------ |
| Version        | 1.0                                        |
| Date           | 2026-02-28                                 |
| Status         | Draft                                      |
| Product Name   | DocConvert                                 |
| Platform       | Arch Linux, Debian/Ubuntu, Fedora/RHEL, Windows |

---

## 1. Executive Summary

DocConvert is a cross-platform desktop application that converts PDF files to Markdown and XLSX files to CSV. Built with Tauri and Rust, the application provides a native GUI for folder selection and performs batch conversion while preserving table structures and supporting international (UTF-8) character sets including Korean, Chinese, Japanese, Swedish, and other scripts.

---

## 2. Business Goals

| # | Goal |
|---|------|
| BG-1 | Provide a free, open-source, offline document conversion tool that eliminates dependency on cloud services. |
| BG-2 | Support international users by handling UTF-8 text across all major writing systems. |
| BG-3 | Reduce friction in document-to-plain-text workflows for researchers, translators, and content teams. |
| BG-4 | Deliver a single application that runs identically on Linux (Arch, Debian, Fedora) and Windows. |

---

## 3. Target Users

| Persona | Description |
|---------|-------------|
| **Researcher / Academic** | Converts large batches of PDF papers to Markdown for annotation, search, and version control. |
| **Translator / Localizer** | Works with multilingual documents; needs reliable UTF-8 handling for CJK and European scripts. |
| **Data Analyst** | Converts XLSX spreadsheets to CSV for ingestion into data pipelines, scripts, or databases. |
| **Technical Writer** | Extracts content from PDFs into Markdown for documentation systems (MkDocs, Docusaurus, etc.). |

---

## 4. User Stories

| ID    | As a …             | I want to …                                                    | So that …                                                        |
| ----- | ------------------- | -------------------------------------------------------------- | ---------------------------------------------------------------- |
| US-01 | User                | Select a folder from a native file picker                      | I can choose where my source files are without typing paths.     |
| US-02 | User                | Convert all PDFs in the folder (recursively) to Markdown       | I get a complete set of Markdown files mirroring the folder.     |
| US-03 | User                | Have tables in PDFs rendered as Markdown tables                 | The structure of tabular data is preserved in the output.        |
| US-04 | Translator          | Convert PDFs containing Korean/Chinese/Japanese text            | International characters are faithfully preserved in output.     |
| US-05 | Data Analyst        | Convert all XLSX files in the folder to CSV                    | I can immediately import the data into scripts or databases.     |
| US-06 | User                | See a progress indicator during batch conversion                | I know how many files remain and can estimate completion.        |
| US-07 | User                | Review a summary of converted and failed files                  | I can quickly identify any documents that need manual review.    |
| US-08 | Arch Linux User     | Install the app via a native package or AppImage                | I don't need to compile from source to use the tool.             |
| US-09 | Windows User        | Install the app via an .msi or .exe installer                   | Installation follows the standard Windows workflow.              |
| US-10 | User                | Choose the output directory for converted files                 | I can keep source and output files organized separately.         |

---

## 5. Scope

### 5.1 In Scope (v1.0)

- Native folder picker via Tauri dialog API.
- Recursive scanning of selected folder for `.pdf` and `.xlsx` files.
- PDF → Markdown conversion with table detection and Markdown table rendering.
- XLSX → CSV conversion with UTF-8 encoding (BOM-free).
- Progress reporting (file count, current file, errors).
- Conversion summary / report at completion.
- Packaging for Arch Linux, Debian (.deb), Fedora (.rpm), and Windows (.msi / .exe).

### 5.2 Out of Scope (v1.0)

- OCR for scanned/image-based PDFs (only text-based PDFs are supported).
- Conversion of other formats (DOCX, PPTX, ODS, etc.).
- Cloud upload or remote file access.
- PDF rendering / preview within the application.
- Editing of generated Markdown or CSV within the application.
- Automatic language detection / translation.

---

## 6. Success Criteria

| # | Criterion | Measurement |
|---|-----------|-------------|
| SC-1 | Text fidelity | Round-trip comparison: ≥ 95% character-level accuracy on UTF-8 benchmark PDFs. |
| SC-2 | Table preservation | ≥ 90% of tables with simple grid layouts convert to valid Markdown tables. |
| SC-3 | Cross-platform parity | Application builds, installs, and runs on all four target platforms without platform-specific bugs. |
| SC-4 | Performance | Converts a 100-page text PDF in under 30 seconds on mid-range hardware (4-core, 8 GB RAM). |
| SC-5 | XLSX accuracy | CSV output matches XLSX cell values (string comparison) for 100% of non-formula cells. |

---

## 7. Assumptions & Constraints

### 7.1 Assumptions

- PDFs contain extractable text (not scanned images).
- XLSX files use the modern Office Open XML format (not legacy .xls).
- Users have sufficient disk space for output files.

### 7.2 Constraints

- Application must remain fully offline — no network calls at runtime.
- Binary size should stay under 30 MB (compressed installer) to remain lightweight.
- Must use permissively-licensed Rust crates (MIT, Apache-2.0, BSD) for distribution.

---

## 8. Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| Complex PDF layouts (multi-column, nested tables) produce garbled output. | Medium | Document known limitations; provide "raw text" fallback mode. |
| CJK font encoding varies across PDF producers. | High | Test against PDFs from major producers (Adobe, LibreOffice, MS Word, LaTeX). |
| Tauri v2 API changes break build on some platforms. | Low | Pin Tauri version; track upstream release notes. |
| Large XLSX files (>100 MB) may exhaust memory. | Medium | Stream rows instead of loading entire workbook into memory. |

---

## 9. Release Plan

| Milestone       | Target          | Deliverables |
| --------------- | --------------- | ------------ |
| M1 — Prototype  | Week 2          | Working PDF→MD conversion in CLI; basic Tauri window with folder picker. |
| M2 — Core       | Week 4          | XLSX→CSV added; table detection; progress UI. |
| M3 — Polish     | Week 6          | Error handling, summary report, UTF-8 benchmark tests. |
| M4 — Packaging  | Week 8          | .deb, .rpm, .msi, Arch PKGBUILD; CI/CD pipeline. |
| M5 — Release    | Week 9          | v1.0 release; documentation published. |
