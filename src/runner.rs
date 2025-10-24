use crate::errors::{AtentoError, Result};
use crate::interpreter;
#[cfg(unix)]
use std::fs::Permissions;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const TEMP_FILENAME: &str = "atento_temp_file_";
const STDERR_FILTER_PATTERNS: &[&str] = &["[Perftrack", "NamedPipeIPC"];
const DEFAULT_RUNNER_TIMEOUT_SECS: u64 = 86400; // 1 day

// A small RAII guard to remove the temp file when dropped
struct TempRemover(PathBuf);
impl Drop for TempRemover {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

pub struct RunnerResult {
    pub exit_code: i32,
    pub duration_ms: u128,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
}

/// Runs a script with a timeout.
///
/// # Errors
/// Returns an error if the script or arguments are empty, if the temp file cannot be created,
/// if the command fails to start, or if the timeout is exceeded.
pub fn run(
    script: &str,
    interpreter: &interpreter::Interpreter,
    timeout_secs: u64,
) -> Result<RunnerResult> {
    if script.is_empty() {
        return Err(AtentoError::Runner("Script cannot be empty".to_string()));
    }

    if !interpreter.is_valid() {
        return Err(AtentoError::Runner(
            "Interpreter has invalid configuration".to_string(),
        ));
    }

    // Create a uniquely-named temporary script file in the OS temp directory.
    // We write and close the file so the spawned process can access it on Windows.
    let mut path = std::env::temp_dir();
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let filename = format!("{TEMP_FILENAME}{nanos}{}", interpreter.extension);
    path.push(filename);

    std::fs::write(&path, format!("{script}\n"))
        .map_err(|e| AtentoError::Runner(format!("Failed to write temp script file: {e}")))?;

    // Set explicit permissions on Unix-like platforms
    #[cfg(unix)]
    {
        let perm = Permissions::from_mode(0o700);
        std::fs::set_permissions(&path, perm)
            .map_err(|e| AtentoError::Runner(format!("Failed to set permissions: {e}")))?;
    }

    // RAII guard to remove the temp file when the function returns
    let _remover = TempRemover(path.clone());

    let mut cmd = Command::new(interpreter.command.as_str());
    if !interpreter.args.is_empty() {
        cmd.args(&interpreter.args);
    }

    // PowerShell: opt out of telemetry
    if interpreter.extension == ".ps1" {
        cmd.env("POWERSHELL_TELEMETRY_OPTOUT", "1");
    }

    let mut child = cmd
        .arg(&path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| AtentoError::Runner(format!("Failed to start command: {e}")))?;

    // temp_file will be dropped when it goes out of scope (after spawn)

    let timeout = if timeout_secs > 0 {
        Duration::from_secs(timeout_secs)
    } else {
        Duration::from_secs(DEFAULT_RUNNER_TIMEOUT_SECS)
    };

    let start = Instant::now();

    loop {
        //        if let Some(status) = child.try_wait().map_err(Ok(op)|e| Err(format!("Failed to check process: {}", e))) {
        if let Some(_status) = child
            .try_wait()
            .map_err(|e| AtentoError::Execution(format!("Failed to check process: {e}")))?
        {
            // Process finished; collect output and return it regardless of exit code.

            let output = child.wait_with_output().map_err(|e| {
                AtentoError::Execution(format!("Failed to wait for process output: {e}"))
            })?;

            return Ok(process_result(&start, &output));
        }

        // Check if the timeout has been reached
        if start.elapsed() >= timeout {
            // Kill the process if timeout exceeded; ignore kill error
            let _ = child
                .kill()
                .map_err(|e| AtentoError::Execution(format!("Failed to kill process: {e}")));

            return Err(AtentoError::Timeout {
                context: "Step execution timed out".to_string(),
                timeout_secs,
            });
        }

        // Sleep for a short duration before checking again
        std::thread::sleep(Duration::from_millis(100)); // Adjust the sleep time as needed
    }
}

fn process_result(start: &Instant, output: &std::process::Output) -> RunnerResult {
    let elapsed = start.elapsed();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let exit_code = output.status.code().unwrap_or(-1);

    // Filter noise from stderr
    let stderr = {
        let raw = String::from_utf8_lossy(&output.stderr);
        raw.lines()
            .filter(|line| !STDERR_FILTER_PATTERNS.iter().any(|pat| line.contains(pat)))
            .collect::<Vec<_>>()
            .join("\n")
    };

    RunnerResult {
        exit_code,
        stdout: Some(stdout.trim().to_string()).filter(|s| !s.is_empty()),
        stderr: Some(stderr.trim().to_string()).filter(|s| !s.is_empty()),
        duration_ms: elapsed.as_millis(),
    }
}
