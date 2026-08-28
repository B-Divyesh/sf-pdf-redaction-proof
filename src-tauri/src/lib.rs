mod pdf;

use pdf::{inspect, sanitize, AuditReport};
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Serialize, Deserialize)]
struct WorkerResponse {
    report: Option<AuditReport>,
    error: Option<String>,
}

fn worker(command: &str, path: &str) -> Result<AuditReport, String> {
    let executable = std::env::current_exe()
        .map_err(|e| format!("Could not start the isolated PDF worker: {e}"))?;
    let mut process = Command::new(executable);
    process.args(["--redaction-proof-worker", command, path]);
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
    let output = process
        .output()
        .map_err(|e| format!("Could not run the isolated PDF worker: {e}"))?;
    let response: WorkerResponse = serde_json::from_slice(&output.stdout)
        .map_err(|_| "The isolated PDF worker stopped before returning a report.".to_string())?;
    response.report.ok_or_else(|| {
        response
            .error
            .unwrap_or_else(|| "The isolated PDF worker failed.".into())
    })
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
        let result = match (args.get(2).map(String::as_str), args.get(3)) {
            (Some("inspect"), Some(path)) => inspect(path),
            (Some("sanitize"), Some(path)) => sanitize(path),
            _ => {
                println!(
                    "{}",
                    serde_json::to_string(&WorkerResponse {
                        report: None,
                        error: Some("Invalid worker request.".into())
                    })
                    .unwrap()
                );
                return;
            }
        };
        let response = match result {
            Ok(report) => WorkerResponse {
                report: Some(report),
                error: None,
            },
            Err(error) => WorkerResponse {
                report: None,
                error: Some(error.to_string()),
            },
        };
        println!("{}", serde_json::to_string(&response).unwrap());
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
