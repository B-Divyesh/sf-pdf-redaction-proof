import "./style.css";
import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { openUrl } from "@tauri-apps/plugin-opener";
import type { AuditReport, Finding } from "./types";
import { cachedUnlock, captureReturnedLicense, storedToken, verifyLicense } from "./license";
import { reportJson, safeBasename, verdictCopy } from "./report";
import { sampleAudit } from "./sample";

const el = <T extends HTMLElement>(id: string) => document.getElementById(id) as T;
const dropZone = el<HTMLDivElement>("drop-zone");
const result = el<HTMLElement>("result");
const errorBox = el<HTMLElement>("error");
const progress = el<HTMLElement>("progress");
let current: AuditReport | null = null;
let proUnlocked = cachedUnlock();

function escape(value: unknown): string {
  return String(value).replace(/[&<>'"]/g, c => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", "'": "&#39;", '"': "&quot;" }[c]!));
}

function setBusy(busy: boolean, name = "") {
  progress.hidden = !busy;
  dropZone.setAttribute("aria-disabled", String(busy));
  el("progress-file").textContent = name ? ` ${name}` : "";
  if (busy) { errorBox.hidden = true; result.hidden = true; }
}

function showError(message: string) {
  errorBox.textContent = message;
  errorBox.hidden = false;
  setBusy(false);
}

function findingMarkup(finding: Finding): string {
  const icon = finding.severity === "critical" ? "!" : finding.severity === "warning" ? "△" : "i";
  return `<li class="finding ${finding.severity}"><span class="finding-icon" aria-hidden="true">${icon}</span><div><strong>${escape(finding.title)}</strong><p>${escape(finding.detail)}</p></div><span class="finding-count">${finding.count}${finding.page ? ` · p${finding.page}` : ""}</span></li>`;
}

function renderReport(report: AuditReport, sample = false) {
  current = report;
  const copy = verdictCopy(report);
  const findings = report.findings.length ? report.findings.map(findingMarkup).join("") : `<li class="finding info"><span class="finding-icon" aria-hidden="true">✓</span><div><strong>Checked structures are clear</strong><p>No risky text overlays, metadata, actions, attachments, or annotations were detected.</p></div></li>`;
  result.innerHTML = `
    <div class="result-head ${report.verdict}">
      <div class="verdict-stamp" aria-hidden="true"><i></i></div>
      <div><p class="eyebrow">AUDIT ${escape(report.verdict.toUpperCase())}</p><h2>${escape(copy.title)}</h2><p>${escape(copy.detail)}</p></div>
    </div>
    <dl class="metrics">
      <div><dt>Pages</dt><dd>${report.page_count}</dd></div>
      <div><dt>Redaction zones</dt><dd>${report.redaction_regions}</dd></div>
      <div><dt>Risk findings</dt><dd>${report.findings.reduce((n, f) => n + f.count, 0)}</dd></div>
      <div><dt>File</dt><dd>${(report.byte_size / 1024 / 1024).toFixed(2)} MB</dd></div>
    </dl>
    <div class="hash-row"><span>SHA-256</span><code title="${escape(report.source_sha256)}">${escape(report.source_sha256)}</code></div>
    <section class="findings" aria-labelledby="findings-title"><h3 id="findings-title">Structural findings</h3><ul>${findings}</ul></section>
    <details class="limits"><summary>What this proof can and cannot establish</summary><ul>${report.limitations.map(x => `<li>${escape(x)}</li>`).join("")}</ul></details>
    ${report.sanitized ? `<div class="sanitized-note"><strong>Sanitized copy created</strong><span>${escape(report.sanitized.path)}</span><small>Verification: ${escape(report.sanitized.verification_verdict)}</small></div>` : ""}
    ${sample ? `<div class="sample-notice"><strong>Sample project · ${escape(report.source_name)}</strong><span>This built-in result stays in memory and saves nothing.</span></div>` : ""}
    <div class="result-actions">
      ${sample ? "" : `<button id="sanitize" class="button primary" type="button">Create sanitized copy</button><button id="export-report" class="button secondary" type="button">Export JSON proof</button>`}
      <button id="inspect-another" class="text-button" type="button">${sample ? "Choose your PDF" : "Inspect another PDF"}</button>
    </div>`;
  result.hidden = false;
  result.querySelector<HTMLButtonElement>("#sanitize")?.addEventListener("click", sanitize);
  result.querySelector<HTMLButtonElement>("#export-report")?.addEventListener("click", exportReport);
  result.querySelector<HTMLButtonElement>("#inspect-another")?.addEventListener("click", chooseFiles);
  result.querySelector<HTMLElement>("h2")?.focus?.();
}

function renderBatch(reports: AuditReport[]) {
  renderReport(reports[reports.length - 1]);
  const summary = document.createElement("section");
  summary.className = "batch-summary";
  summary.setAttribute("aria-labelledby", "batch-title");
  summary.innerHTML = `<p class="eyebrow">BATCH COMPLETE</p><h2 id="batch-title">${reports.length} PDFs audited</h2><ul>${reports.map(report => `<li><span class="batch-verdict ${report.verdict}">${escape(report.verdict.toUpperCase())}</span><strong>${escape(report.source_name)}</strong><span>${report.findings.length} finding types</span></li>`).join("")}</ul><p>The detailed result below is for the last file. Re-open any file individually to sanitize or export its proof.</p>`;
  result.prepend(summary);
}

async function audit(path: string) {
  setBusy(true, path.split(/[\\/]/).pop());
  try {
    const report = await invoke<AuditReport>("inspect_pdf", { path });
    setBusy(false);
    renderReport(report);
  } catch (error) { showError(String(error).replace(/^Error: /, "")); }
}

async function chooseFiles() {
  try {
    const selection = await open({ multiple: proUnlocked, filters: [{ name: "PDF documents", extensions: ["pdf"] }] });
    if (!selection) return;
    const paths = Array.isArray(selection) ? selection : [selection];
    if (paths.length > 1) {
      const reports: AuditReport[] = [];
      for (let i = 0; i < paths.length; i++) {
        setBusy(true, `${i + 1}/${paths.length} · ${paths[i].split(/[\\/]/).pop()}`);
        reports.push(await invoke<AuditReport>("inspect_pdf", { path: paths[i] }));
      }
      if (reports.length) renderBatch(reports);
    } else await audit(paths[0]);
  } catch (error) { showError(`Could not open that PDF. ${String(error)}`); }
}

async function sanitize() {
  if (!current) return;
  const button = result.querySelector<HTMLButtonElement>("#sanitize");
  if (button) { button.disabled = true; button.textContent = "Sanitizing…"; }
  try {
    const updated = await invoke<AuditReport>("sanitize_pdf", { path: current.source_path });
    renderReport(updated);
  } catch (error) {
    showError(`A sanitized copy could not be created. ${String(error)}`);
    if (button) { button.disabled = false; button.textContent = "Create sanitized copy"; }
  }
}

async function exportReport() {
  if (!current) return;
  const path = await save({ defaultPath: `${safeBasename(current.source_name)}-redaction-proof.json`, filters: [{ name: "JSON", extensions: ["json"] }] });
  if (!path) return;
  try {
    await invoke("write_report", { path, contents: reportJson(current) });
    const button = result.querySelector<HTMLButtonElement>("#export-report");
    if (button) button.textContent = "JSON proof saved";
  } catch (error) { showError(`The proof report could not be saved. ${String(error)}`); }
}

async function initLicense() {
  const token = captureReturnedLicense();
  const status = el("license-status");
  const update = (valid: boolean) => {
    proUnlocked = valid;
    status.textContent = valid ? "Pro active · batch selection enabled" : token ? "License no longer active · free edition available" : "Free edition · one file at a time";
  };
  update(cachedUnlock());
  if (token) verifyLicense(token).then(update).catch(() => { if (!navigator.onLine) status.textContent = "Offline · using last verified license state"; });
  el("restore-toggle").addEventListener("click", () => {
    const form = el<HTMLFormElement>("license-form");
    form.hidden = !form.hidden;
    el("restore-toggle").setAttribute("aria-expanded", String(!form.hidden));
    if (!form.hidden) el<HTMLInputElement>("license-token").focus();
  });
  el<HTMLFormElement>("license-form").addEventListener("submit", async event => {
    event.preventDefault();
    const value = el<HTMLInputElement>("license-token").value.trim();
    if (!value) { status.textContent = "Paste the license token from your receipt."; return; }
    status.textContent = "Checking license…";
    try { update(await verifyLicense(value, true)); } catch { status.textContent = "Could not reach the license service. Try again when online."; }
  });
  if (storedToken() && !token) update(cachedUnlock());
}

function init() {
  el("pick-file").addEventListener("click", event => { event.stopPropagation(); chooseFiles(); });
  el("load-sample").addEventListener("click", event => { event.stopPropagation(); renderReport(sampleAudit, true); });
  const buyLink = el<HTMLAnchorElement>("buy-pro");
  buyLink.addEventListener("click", event => {
    if ("__TAURI_INTERNALS__" in window) { event.preventDefault(); openUrl(buyLink.href); }
  });
  dropZone.addEventListener("click", chooseFiles);
  if ("__TAURI_INTERNALS__" in window) {
    getCurrentWebview().onDragDropEvent(event => {
      if (event.payload.type === "over") dropZone.classList.add("dragging");
      if (event.payload.type === "leave") dropZone.classList.remove("dragging");
      if (event.payload.type === "drop") {
        dropZone.classList.remove("dragging");
        const paths = event.payload.paths.filter(p => p.toLowerCase().endsWith(".pdf"));
        if (!paths.length) showError("That item is not a PDF. Choose a file ending in .pdf.");
        else if (paths.length > 1 && !proUnlocked) showError("Free edition inspects one PDF at a time. Choose a single file, or unlock batch auditing.");
        else audit(paths[0]);
      }
    }).catch(() => { /* native drag events are optional */ });
  }
  const syncOnline = () => { el("offline-note").hidden = navigator.onLine; };
  addEventListener("online", syncOnline); addEventListener("offline", syncOnline); syncOnline();
  initLicense();
}

init();
