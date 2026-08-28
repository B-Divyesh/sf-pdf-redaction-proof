import { describe, expect, it } from "vitest";
import { reportJson, safeBasename, verdictCopy } from "./report";
import type { AuditReport } from "./types";

const report: AuditReport = { schema_version:"1.0",generated_at:"2026-08-28T00:00:00Z",app_version:"0.1.0",source_name:"Client file.pdf",source_path:"/secret/Client file.pdf",source_sha256:"abc",byte_size:10,page_count:1,verdict:"fail",recoverable_text_fragments:1,redaction_regions:1,findings:[],limitations:[] };

describe("portable reports", () => {
  it("does not export the local source path", () => expect(reportJson(report)).not.toContain("/secret/"));
  it("makes safe report names", () => expect(safeBasename("Client file.pdf")).toBe("Client-file"));
  it("explains failure without claiming certainty", () => expect(verdictCopy(report).title).toContain("Recoverable"));
  it("@claim:json-proof identifies source and sanitized files without exposing paths", () => {
    const proof = reportJson({ ...report, sanitized: { path: "/secret/Client-file.sanitized.pdf", sha256: "def", verification_verdict: "pass" } });
    expect(JSON.parse(proof)).toMatchObject({ source_name: "Client file.pdf", source_sha256: "abc", sanitized: { sha256: "def", verification_verdict: "pass" } });
    expect(proof).not.toContain("/secret/");
  });
});
