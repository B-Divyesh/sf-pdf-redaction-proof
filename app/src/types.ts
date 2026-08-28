export type Severity = "critical" | "warning" | "info";

export interface Finding {
  code: string;
  severity: Severity;
  page?: number;
  title: string;
  detail: string;
  count: number;
}

export interface AuditReport {
  schema_version: string;
  generated_at: string;
  app_version: string;
  source_name: string;
  source_path: string;
  source_sha256: string;
  byte_size: number;
  page_count: number;
  verdict: "pass" | "review" | "fail";
  recoverable_text_fragments: number;
  redaction_regions: number;
  findings: Finding[];
  sanitized?: {
    path: string;
    sha256: string;
    verification_verdict: string;
  };
  limitations: string[];
}
