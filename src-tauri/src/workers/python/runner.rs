use std::{
    error::Error,
    ffi::OsString,
    fmt,
    path::{Path, PathBuf},
    process::{ExitStatus, Stdio},
    time::Duration,
};

use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWriteExt},
    process::{Child, ChildStderr, ChildStdin, ChildStdout, Command},
    time::sleep,
};

use crate::workers::cancellation::{WorkerCancellation, WorkerRegistration};

use super::protocol::{
    ContractProbeInput, ContractProbeResult, PythonHelperFailureCode, PythonHelperProtocolError,
    PythonHelperRequest, decode_contract_probe_success, decode_failure,
    decode_text_analysis_success, encode_request,
};
use super::text_analysis::{TextAnalysisInput, TextAnalysisResult};

/// Maximum helper stdout retained by the Rust process boundary.
pub const MAX_PYTHON_HELPER_STDOUT_BYTES: usize = 64 * 1024;

/// Maximum helper stderr retained for internal diagnostics and then discarded.
pub const MAX_PYTHON_HELPER_STDERR_BYTES: usize = 16 * 1024;

/// Fixed execution deadline for allowlisted Python helpers.
pub const PYTHON_HELPER_TIMEOUT: Duration = Duration::from_secs(5);

const HELPER_ENTRYPOINT: [&str; 2] = ["draft_helpers", "worker.py"];
const READ_BUFFER_BYTES: usize = 8 * 1024;

/// Trusted Rust configuration failures that expose no filesystem path.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PythonHelperConfigurationError {
    InvalidExecutable,
    InvalidPackageRoot,
    InvalidEntrypoint,
}

/// Bounded helper execution failures with no payload or process diagnostics.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PythonHelperRunError {
    RequestTooLarge,
    SpawnFailed,
    InputOutputFailed,
    TimedOut,
    Cancelled,
    StdoutTooLarge,
    ExecutionFailed,
    HelperRejected { code: PythonHelperFailureCode },
    InvalidOutput,
    ResponseMismatch,
}

/// Rust-owned runner for the fixed allowlisted Python helper entrypoint.
pub struct PythonHelperRunner {
    program: PythonHelperProgram,
}

struct PythonHelperProgram {
    executable: PathBuf,
    package_root: PathBuf,
    entrypoint: PathBuf,
    arguments: Vec<OsString>,
    timeout: Duration,
}

struct ProcessCapture {
    status: ExitStatus,
    stdout: BoundedCapture,
    stderr: BoundedCapture,
}

struct ChildPipes {
    stdin: ChildStdin,
    stdout: ChildStdout,
    stderr: ChildStderr,
}

struct HelperExecution {
    pipes: ChildPipes,
    request: Vec<u8>,
    cancellation: WorkerCancellation,
    timeout: Duration,
}

struct BoundedCapture {
    bytes: Vec<u8>,
    overflowed: bool,
}

enum ProcessTerminal {
    Completed(Result<ProcessCapture, ()>),
    Cancelled,
    TimedOut,
}

impl PythonHelperRunner {
    /// Validates a trusted Python executable and derives the fixed helper path.
    pub fn new(
        python_executable: impl AsRef<Path>,
        package_root: impl AsRef<Path>,
    ) -> Result<Self, PythonHelperConfigurationError> {
        let executable = canonical_file(
            python_executable.as_ref(),
            PythonHelperConfigurationError::InvalidExecutable,
        )?;
        let package_root = canonical_directory(package_root.as_ref())?;
        let entrypoint = fixed_entrypoint(&package_root)?;
        Ok(Self {
            program: PythonHelperProgram {
                executable,
                package_root,
                entrypoint,
                arguments: Vec::new(),
                timeout: PYTHON_HELPER_TIMEOUT,
            },
        })
    }

    /// Runs the protocol probe while owning the supplied worker registration.
    pub async fn run_contract_probe(
        &self,
        input: ContractProbeInput,
        registration: WorkerRegistration,
    ) -> Result<ContractProbeResult, PythonHelperRunError> {
        let request = PythonHelperRequest::contract_probe(input);
        let capture = self.execute_request(&request, registration).await?;
        let output = require_successful_output(capture)?;
        decode_contract_probe_success(&request, &output).map_err(map_protocol_error)
    }

    /// Runs deterministic text checks without persisting or mutating input.
    pub async fn run_text_analysis(
        &self,
        input: TextAnalysisInput,
        registration: WorkerRegistration,
    ) -> Result<TextAnalysisResult, PythonHelperRunError> {
        let request = PythonHelperRequest::text_analysis(input);
        let capture = self.execute_request(&request, registration).await?;
        let output = require_successful_output(capture)?;
        decode_text_analysis_success(&request, &output).map_err(map_protocol_error)
    }

    async fn execute_request(
        &self,
        request: &PythonHelperRequest,
        registration: WorkerRegistration,
    ) -> Result<ProcessCapture, PythonHelperRunError> {
        let cancellation = registration.cancellation();
        let encoded = encode_request(request).map_err(map_protocol_error)?;
        if cancellation.is_cancelled() {
            return Err(PythonHelperRunError::Cancelled);
        }
        let child = self.spawn_helper()?;
        run_child(child, encoded, cancellation, self.program.timeout).await
    }

    fn spawn_helper(&self) -> Result<Child, PythonHelperRunError> {
        let mut command = Command::new(&self.program.executable);
        command
            .arg("-I")
            .arg("-B")
            .arg(&self.program.entrypoint)
            .args(&self.program.arguments)
            .current_dir(&self.program.package_root)
            .env_clear()
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        command
            .spawn()
            .map_err(|_| PythonHelperRunError::SpawnFailed)
    }

    #[cfg(test)]
    fn for_fixture(
        python_executable: &Path,
        mode: &str,
        timeout: Duration,
    ) -> Result<Self, PythonHelperConfigurationError> {
        let executable = canonical_file(
            python_executable,
            PythonHelperConfigurationError::InvalidExecutable,
        )?;
        let package_root = canonical_directory(Path::new(env!("CARGO_MANIFEST_DIR")))?;
        let entrypoint = canonical_file(
            &package_root.join("src/workers/python/worker_fixture.py"),
            PythonHelperConfigurationError::InvalidEntrypoint,
        )?;
        Ok(Self {
            program: PythonHelperProgram {
                executable,
                package_root,
                entrypoint,
                arguments: vec![OsString::from(mode)],
                timeout,
            },
        })
    }
}

impl fmt::Display for PythonHelperConfigurationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidExecutable => "Python helper executable is invalid",
            Self::InvalidPackageRoot => "Python helper package root is invalid",
            Self::InvalidEntrypoint => "Python helper entrypoint is invalid",
        })
    }
}

impl Error for PythonHelperConfigurationError {}

impl fmt::Display for PythonHelperRunError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::RequestTooLarge => "Python helper request is too large",
            Self::SpawnFailed => "Python helper could not start",
            Self::InputOutputFailed => "Python helper communication failed",
            Self::TimedOut => "Python helper timed out",
            Self::Cancelled => "Python helper was cancelled",
            Self::StdoutTooLarge => "Python helper output is too large",
            Self::ExecutionFailed => "Python helper execution failed",
            Self::HelperRejected { .. } => "Python helper rejected the request",
            Self::InvalidOutput => "Python helper output is invalid",
            Self::ResponseMismatch => "Python helper response does not match the request",
        })
    }
}

impl Error for PythonHelperRunError {}

async fn run_child(
    mut child: Child,
    request: Vec<u8>,
    cancellation: WorkerCancellation,
    timeout: Duration,
) -> Result<ProcessCapture, PythonHelperRunError> {
    let execution = HelperExecution {
        pipes: take_child_pipes(&mut child)?,
        request,
        cancellation,
        timeout,
    };
    let terminal = await_terminal(&mut child, execution).await;
    finish_terminal(&mut child, terminal).await
}

async fn await_terminal(child: &mut Child, execution: HelperExecution) -> ProcessTerminal {
    let exchange = exchange_with_child(child, execution.pipes, execution.request);
    tokio::pin!(exchange);
    tokio::select! {
        result = &mut exchange => ProcessTerminal::Completed(result),
        () = execution.cancellation.cancelled() => ProcessTerminal::Cancelled,
        () = sleep(execution.timeout) => ProcessTerminal::TimedOut,
    }
}

async fn finish_terminal(
    child: &mut Child,
    terminal: ProcessTerminal,
) -> Result<ProcessCapture, PythonHelperRunError> {
    match terminal {
        ProcessTerminal::Completed(result) => {
            result.map_err(|_| PythonHelperRunError::InputOutputFailed)
        }
        ProcessTerminal::Cancelled => {
            terminate_child(child).await;
            Err(PythonHelperRunError::Cancelled)
        }
        ProcessTerminal::TimedOut => {
            terminate_child(child).await;
            Err(PythonHelperRunError::TimedOut)
        }
    }
}

async fn exchange_with_child(
    child: &mut Child,
    pipes: ChildPipes,
    request: Vec<u8>,
) -> Result<ProcessCapture, ()> {
    let wait = async { child.wait().await.map_err(|_| ()) };
    let ((), status, stdout, stderr) = tokio::try_join!(
        write_request(pipes.stdin, request),
        wait,
        read_bounded(pipes.stdout, MAX_PYTHON_HELPER_STDOUT_BYTES),
        read_bounded(pipes.stderr, MAX_PYTHON_HELPER_STDERR_BYTES),
    )?;
    Ok(ProcessCapture {
        status,
        stdout,
        stderr,
    })
}

fn take_child_pipes(child: &mut Child) -> Result<ChildPipes, PythonHelperRunError> {
    Ok(ChildPipes {
        stdin: child
            .stdin
            .take()
            .ok_or(PythonHelperRunError::InputOutputFailed)?,
        stdout: child
            .stdout
            .take()
            .ok_or(PythonHelperRunError::InputOutputFailed)?,
        stderr: child
            .stderr
            .take()
            .ok_or(PythonHelperRunError::InputOutputFailed)?,
    })
}

async fn write_request(mut stdin: ChildStdin, request: Vec<u8>) -> Result<(), ()> {
    stdin.write_all(&request).await.map_err(|_| ())?;
    stdin.shutdown().await.map_err(|_| ())
}

async fn read_bounded(
    mut reader: impl AsyncRead + Unpin,
    limit: usize,
) -> Result<BoundedCapture, ()> {
    let mut bytes = Vec::with_capacity(limit.min(READ_BUFFER_BYTES));
    let mut buffer = [0_u8; READ_BUFFER_BYTES];
    let mut overflowed = false;
    loop {
        let count = reader.read(&mut buffer).await.map_err(|_| ())?;
        if count == 0 {
            return Ok(BoundedCapture { bytes, overflowed });
        }
        let retained = limit.saturating_sub(bytes.len()).min(count);
        bytes.extend_from_slice(&buffer[..retained]);
        overflowed |= retained < count;
    }
}

async fn terminate_child(child: &mut Child) {
    let _ = child.start_kill();
    let _ = child.wait().await;
}

fn require_successful_output(capture: ProcessCapture) -> Result<Vec<u8>, PythonHelperRunError> {
    if capture.stdout.overflowed {
        return Err(PythonHelperRunError::StdoutTooLarge);
    }
    if capture.status.success() {
        require_clean_stderr(&capture.stderr)?;
        Ok(capture.stdout.bytes)
    } else {
        let code = decode_failure(&capture.stdout.bytes)
            .map_err(|_| PythonHelperRunError::ExecutionFailed)?;
        Err(PythonHelperRunError::HelperRejected { code })
    }
}

fn require_clean_stderr(stderr: &BoundedCapture) -> Result<(), PythonHelperRunError> {
    if stderr.overflowed || !stderr.bytes.is_empty() {
        Err(PythonHelperRunError::InvalidOutput)
    } else {
        Ok(())
    }
}

fn map_protocol_error(error: PythonHelperProtocolError) -> PythonHelperRunError {
    match error {
        PythonHelperProtocolError::RequestTooLarge => PythonHelperRunError::RequestTooLarge,
        PythonHelperProtocolError::InvalidSuccess | PythonHelperProtocolError::InvalidFailure => {
            PythonHelperRunError::InvalidOutput
        }
        PythonHelperProtocolError::ResponseMismatch => PythonHelperRunError::ResponseMismatch,
    }
}

fn fixed_entrypoint(package_root: &Path) -> Result<PathBuf, PythonHelperConfigurationError> {
    let entrypoint = canonical_file(
        &package_root
            .join(HELPER_ENTRYPOINT[0])
            .join(HELPER_ENTRYPOINT[1]),
        PythonHelperConfigurationError::InvalidEntrypoint,
    )?;
    if entrypoint.starts_with(package_root) {
        Ok(entrypoint)
    } else {
        Err(PythonHelperConfigurationError::InvalidEntrypoint)
    }
}

fn canonical_file(
    path: &Path,
    error: PythonHelperConfigurationError,
) -> Result<PathBuf, PythonHelperConfigurationError> {
    if !path.is_absolute() {
        return Err(error);
    }
    let path = path.canonicalize().map_err(|_| error)?;
    if path.is_file() { Ok(path) } else { Err(error) }
}

fn canonical_directory(path: &Path) -> Result<PathBuf, PythonHelperConfigurationError> {
    if !path.is_absolute() {
        return Err(PythonHelperConfigurationError::InvalidPackageRoot);
    }
    let path = path
        .canonicalize()
        .map_err(|_| PythonHelperConfigurationError::InvalidPackageRoot)?;
    if path.is_dir() {
        Ok(path)
    } else {
        Err(PythonHelperConfigurationError::InvalidPackageRoot)
    }
}

#[cfg(test)]
#[path = "runner_tests.rs"]
mod tests;
