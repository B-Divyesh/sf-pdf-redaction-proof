import type { AuditReport } from "./types";

export const sampleAudit: AuditReport = {
  schema_version: "1.0",
  generated_at: "2026-08-28T09:00:00Z",
  app_version: "0.1.3",
  source_name: "sample-board-minutes.pdf",
  source_path: "demo:sample-board-minutes.pdf",
  source_sha256: "6b21f0874aa4f56c369162dd61ec95d5ac6716c424819ab533f7b973f017d904",
  byte_size: 862208,
  page_count: 12,
  verdict: "fail",
  recoverable_text_fragments: 1,
  redaction_regions: 1,
  findings: [
    {
      code: "covered_text",
      severity: "critical",
      page: 4,
      title: "Text remains behind a black rectangle",
      detail: "The phrase under the cover can still be recovered from the page content.",
      count: 1,
    },
    {
      code: "document_metadata",
      severity: "warning",
      title: "Author name remains in document details",
      detail: "The author field contains a name that may identify the document source.",
      count: 1,
    },
  ],
  limitations: [
    "Review text inside images by eye.",
    "Unusual clipping paths and rotated objects may need manual review.",
  ],
};
