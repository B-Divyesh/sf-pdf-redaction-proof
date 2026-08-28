import type { AuditReport } from "./types";

export function verdictCopy(report: AuditReport): { title: string; detail: string } {
  if (report.verdict === "pass") return { title: "No recoverable risks found", detail: "The inspected structures passed this tool’s checks. Review the limitations before sharing." };
  if (report.verdict === "fail") return { title: "Recoverable content found", detail: "This file should not be shared yet. Sanitize it, then verify the new copy." };
  return { title: "Manual review needed", detail: "The file contains structures that need a closer look before sharing." };
}

export function safeBasename(name: string): string {
  return name.replace(/\.pdf$/i, "").replace(/[^a-z0-9._-]+/gi, "-").replace(/^-|-$/g, "") || "document";
}

export function reportJson(report: AuditReport): string {
  const portable = {
    ...report,
    source_path: undefined,
    sanitized: report.sanitized ? { ...report.sanitized, path: undefined } : undefined,
  };
  return JSON.stringify(portable, null, 2);
}
