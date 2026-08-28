mod pdf;
mod sandbox;

use pdf::{inspect_bytes, output_path, sanitize_bytes, validate_input, AuditReport};
use serde::{Deserialize, Serialize};
use std::{
    io::{Read, Write},
    path::Path,
    process::{Command, Stdio},
};

#[derive(Serialize, Deserialize)]
struct WorkerResponse {
    report: Option<AuditReport>,
    error: Option<String>,
    sanitized_bytes: usize,
}

fn worker(command: &str, path: &str) -> Result<AuditReport, String> {
    let input_path = Path::new(path);
    let bytes = validate_input(input_path).map_err(|e| e.to_string())?;
    let source_name = input_path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("document.pdf");
    let executable = std::env::current_exe()
        .map_err(|e| format!("Could not start the isolated PDF worker: {e}"))?;
    let mut process = Command::new(executable);
    process
        .args(["--redaction-proof-worker", command, source_name])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        unsafe {
            process.pre_exec(|| {
                let memory = libc::rlimit {
                    rlim_cur: 1_610_612_736,
                    rlim_max: 1_610_612_736,
                };
                let cpu = libc::rlimit {
                    rlim_cur: 60,
                    rlim_max: 60,
                };
                if libc::setrlimit(libc::RLIMIT_AS, &memory) != 0
                    || libc::setrlimit(libc::RLIMIT_CPU, &cpu) != 0
                {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
    }
    let mut child = process
        .spawn()
        .map_err(|e| format!("Could not run the isolated PDF worker: {e}"))?;
    child
        .stdin
        .take()
        .ok_or_else(|| "Could not open the worker input pipe.".to_string())?
        .write_all(&bytes)
        .map_err(|e| format!("Could not send the PDF to the isolated worker: {e}"))?;
    let output = child
        .wait_with_output()
        .map_err(|e| format!("Could not read the isolated PDF worker: {e}"))?;
    if output.stdout.len() < 8 {
        return Err("The isolated PDF worker stopped before returning a report.".into());
    }
    let header_len = u64::from_be_bytes(output.stdout[..8].try_into().unwrap()) as usize;
    if header_len > 2_000_000 || output.stdout.len() < 8 + header_len {
        return Err("The isolated PDF worker returned an invalid response.".into());
    }
    let response: WorkerResponse = serde_json::from_slice(&output.stdout[8..8 + header_len])
        .map_err(|_| "The isolated PDF worker stopped before returning a report.".to_string())?;
    let mut report = response.report.ok_or_else(|| {
        response
            .error
            .unwrap_or_else(|| "The isolated PDF worker failed.".into())
    })?;
    report.source_path = path.into();
    if command == "sanitize" {
        let clean = &output.stdout[8 + header_len..];
        if clean.len() != response.sanitized_bytes {
            return Err("The isolated PDF worker returned an incomplete sanitized copy.".into());
        }
        let destination = output_path(input_path);
        std::fs::write(&destination, clean)
            .map_err(|e| format!("The sanitized copy could not be written: {e}"))?;
        if let Some(info) = &mut report.sanitized {
            info.path = destination.to_string_lossy().into_owned();
        }
    }
    Ok(report)
}

#[tauri::command]
fn inspect_pdf(path: String) -> Result<AuditReport, String> {
    worker("inspect", &path)
}

#[tauri::command]
fn sanitize_pdf(path: String) -> Result<AuditReport, String> {
    worker("sanitize", &path)
}

#[tauri::command]
fn write_report(path: String, contents: String) -> Result<(), String> {
    if contents.len() > 2_000_000 {
        return Err("Report exceeds the 2 MB safety limit.".into());
    }
    std::fs::write(path, contents).map_err(|e| format!("Could not write report: {e}"))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("--redaction-proof-worker") {
        let mut bytes = Vec::new();
        let read_result = std::io::stdin()
            .take(500 * 1024 * 1024 + 1)
            .read_to_end(&mut bytes);
        let sandbox_result = sandbox::enter();
        let result = match (args.get(2).map(String::as_str), args.get(3)) {
            _ if read_result.is_err() => Err("The worker could not read the PDF bytes.".into()),
            _ if bytes.len() > 500 * 1024 * 1024 => {
                Err("This PDF is larger than the 500 MB safety limit.".into())
            }
            _ if sandbox_result.is_err() => Err(format!(
                "The operating-system PDF sandbox could not start: {}",
                sandbox_result.unwrap_err()
            )),
            (Some("inspect"), Some(name)) => inspect_bytes(name, &bytes)
                .map(|report| (report, Vec::new()))
                .map_err(|e| e.to_string()),
            (Some("sanitize"), Some(name)) => {
                sanitize_bytes(name, &bytes).map_err(|e| e.to_string())
            }
            _ => Err("Invalid worker request.".into()),
        };
        let (response, sanitized) = match result {
            Ok((report, sanitized)) => (
                WorkerResponse {
                    report: Some(report),
                    error: None,
                    sanitized_bytes: sanitized.len(),
                },
                sanitized,
            ),
            Err(error) => (
                WorkerResponse {
                    report: None,
                    error: Some(error.to_string()),
                    sanitized_bytes: 0,
                },
                Vec::new(),
            ),
        };
        let header = serde_json::to_vec(&response).unwrap();
        let mut stdout = std::io::stdout().lock();
        let _ = stdout.write_all(&(header.len() as u64).to_be_bytes());
        let _ = stdout.write_all(&header);
        let _ = stdout.write_all(&sanitized);
        return;
    }
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            inspect_pdf,
            sanitize_pdf,
            write_report
        ])
        .run(tauri::generate_context!())
        .expect("error while running Redaction Proof");
}
