use serde_json::Value;
use std::{
    io::Write,
    process::{Command, Stdio},
};

fn run_worker(command: &str, fixture: &str) -> (Value, Vec<u8>) {
    let bytes = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../tests/fixtures/pdf-corpus")
            .join(fixture),
    )
    .unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_redaction-proof"))
        .args(["--redaction-proof-worker", command, fixture])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(&bytes).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    assert!(output.stdout.len() >= 8);
    let header_len = u64::from_be_bytes(output.stdout[..8].try_into().unwrap()) as usize;
    let header = serde_json::from_slice(&output.stdout[8..8 + header_len]).unwrap();
    (header, output.stdout[8 + header_len..].to_vec())
}

#[test]
fn claim_local_worker_parses_piped_bytes_inside_the_os_sandbox() {
    let (header, trailing) = run_worker("inspect", "hidden-render-mode.pdf");
    assert!(header["error"].is_null());
    assert_eq!(header["report"]["source_name"], "hidden-render-mode.pdf");
    assert_eq!(header["report"]["source_path"], "hidden-render-mode.pdf");
    assert_eq!(header["report"]["verdict"], "fail");
    assert!(trailing.is_empty());
}

#[test]
fn worker_returns_a_rechecked_sanitized_pdf_over_the_output_pipe() {
    let (header, trailing) = run_worker("sanitize", "info-and-xmp.pdf");
    assert!(header["error"].is_null());
    assert_eq!(
        header["report"]["sanitized"]["verification_verdict"],
        "pass"
    );
    assert_eq!(header["sanitized_bytes"], trailing.len());
    assert!(trailing.starts_with(b"%PDF-"));
}
