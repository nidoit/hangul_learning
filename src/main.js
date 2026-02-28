const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

// ── State ──────────────────────────────────────────────
let state = {
  sourceDir: null,
  outputDir: null,
  scanResult: null,
  results: null,
};

// ── DOM Elements ───────────────────────────────────────
const $ = (id) => document.getElementById(id);

const els = {
  sourcePath: $("source-path"),
  outputPath: $("output-path"),
  btnSelectSource: $("btn-select-source"),
  btnSelectOutput: $("btn-select-output"),
  fileInfo: $("file-info"),
  pdfCount: $("pdf-count"),
  xlsxCount: $("xlsx-count"),
  totalSize: $("total-size"),
  optionsSection: $("options-section"),
  overwriteCheck: $("overwrite-check"),
  actionSection: $("action-section"),
  btnConvert: $("btn-convert"),
  progressSection: $("progress-section"),
  progressFill: $("progress-fill"),
  progressText: $("progress-text"),
  currentFile: $("current-file"),
  summarySection: $("summary-section"),
  successCount: $("success-count"),
  failCount: $("fail-count"),
  totalTime: $("total-time"),
  errorList: $("error-list"),
  btnCopySummary: $("btn-copy-summary"),
  btnNew: $("btn-new"),
};

// ── Helpers ────────────────────────────────────────────
function formatSize(bytes) {
  if (bytes < 1024) return bytes + " B";
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + " KB";
  return (bytes / (1024 * 1024)).toFixed(1) + " MB";
}

function show(...sections) {
  sections.forEach((s) => s.classList.remove("hidden"));
}

function hide(...sections) {
  sections.forEach((s) => s.classList.add("hidden"));
}

// ── Folder Selection ───────────────────────────────────
els.btnSelectSource.addEventListener("click", async () => {
  try {
    const path = await invoke("select_folder");
    if (!path) return;

    state.sourceDir = path;
    els.sourcePath.textContent = path;

    // Default output to source_dir/output
    state.outputDir = path + "/output";
    els.outputPath.textContent = state.outputDir;

    // Scan files
    await scanFiles();
  } catch (err) {
    alert("Error selecting folder: " + err);
  }
});

els.btnSelectOutput.addEventListener("click", async () => {
  try {
    const path = await invoke("select_folder");
    if (!path) return;

    state.outputDir = path;
    els.outputPath.textContent = path;
  } catch (err) {
    alert("Error selecting output folder: " + err);
  }
});

async function scanFiles() {
  try {
    const result = await invoke("scan_files", { sourceDir: state.sourceDir });
    state.scanResult = result;

    els.pdfCount.textContent = result.pdf_count;
    els.xlsxCount.textContent = result.xlsx_count;
    els.totalSize.textContent = formatSize(result.total_size_bytes);

    show(els.fileInfo, els.optionsSection, els.actionSection);
    hide(els.progressSection, els.summarySection);

    // Disable convert if no files
    els.btnConvert.disabled = result.files.length === 0;
    if (result.files.length === 0) {
      els.btnConvert.textContent = "No supported files found";
    } else {
      els.btnConvert.textContent = `Convert ${result.files.length} Files`;
    }
  } catch (err) {
    alert("Error scanning: " + err);
  }
}

// ── Conversion ─────────────────────────────────────────
els.btnConvert.addEventListener("click", async () => {
  if (!state.scanResult || state.scanResult.files.length === 0) return;

  // Show progress, hide summary
  show(els.progressSection);
  hide(els.summarySection, els.actionSection);

  els.progressFill.style.width = "0%";
  els.progressText.textContent = "0 / " + state.scanResult.files.length;
  els.currentFile.textContent = "Starting…";

  try {
    const results = await invoke("start_conversion", {
      job: {
        source_dir: state.sourceDir,
        output_dir: state.outputDir,
        overwrite: els.overwriteCheck.checked,
        files: state.scanResult.files,
      },
    });

    state.results = results;
    // Summary is shown via the conversion-complete event listener
  } catch (err) {
    alert("Conversion error: " + err);
    show(els.actionSection);
    hide(els.progressSection);
  }
});

// ── Event Listeners (Tauri events) ─────────────────────
listen("conversion-progress", (event) => {
  const { current, total, current_file } = event.payload;
  const pct = Math.round((current / total) * 100);

  els.progressFill.style.width = pct + "%";
  els.progressText.textContent = `${current} / ${total}`;
  els.currentFile.textContent = current_file;
});

listen("conversion-complete", (event) => {
  const { total, succeeded, failed, duration_ms } = event.payload;

  els.successCount.textContent = succeeded;
  els.failCount.textContent = failed;
  els.totalTime.textContent = (duration_ms / 1000).toFixed(1);

  // Show errors if any
  if (state.results) {
    const failures = state.results.filter((r) => r.status === "Failed");
    if (failures.length > 0) {
      els.errorList.innerHTML = failures
        .map(
          (f) =>
            `<div class="error-item"><span class="error-file">${f.file.relative_path}</span><span class="error-msg">${f.error || "Unknown error"}</span></div>`
        )
        .join("");
      show(els.errorList);
    } else {
      hide(els.errorList);
    }
  }

  hide(els.progressSection);
  show(els.summarySection);
});

// ── Summary Actions ────────────────────────────────────
els.btnCopySummary.addEventListener("click", () => {
  if (!state.results) return;

  const lines = ["DocConvert — Conversion Report", ""];
  const succeeded = state.results.filter((r) => r.status === "Success");
  const failed = state.results.filter((r) => r.status === "Failed");

  lines.push(`Succeeded: ${succeeded.length}`);
  lines.push(`Failed: ${failed.length}`);
  lines.push("");

  if (failed.length > 0) {
    lines.push("Failed files:");
    failed.forEach((f) => {
      lines.push(`  - ${f.file.relative_path}: ${f.error || "Unknown"}`);
    });
  }

  navigator.clipboard.writeText(lines.join("\n")).then(
    () => (els.btnCopySummary.textContent = "Copied!"),
    () => (els.btnCopySummary.textContent = "Copy failed")
  );

  setTimeout(() => (els.btnCopySummary.textContent = "Copy Report"), 2000);
});

els.btnNew.addEventListener("click", () => {
  state.sourceDir = null;
  state.outputDir = null;
  state.scanResult = null;
  state.results = null;

  els.sourcePath.textContent = "No folder selected";
  els.outputPath.textContent = "—";

  hide(els.fileInfo, els.optionsSection, els.actionSection, els.progressSection, els.summarySection);
});
