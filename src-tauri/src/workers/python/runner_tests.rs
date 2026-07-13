use std::{
    path::{Path, PathBuf},
    process::Command,
    thread,
    time::{Duration, Instant},
};

use crate::network::connectivity::{ConnectivityMode, ConnectivityPolicy};
use crate::workers::{
    cancellation::{CancelWorkerOutcome, WorkerCancellationRegistry},
    python::{TextAnalysisCategory, TextAnalysisFinding, TextAnalysisFindingCode},
};

use super::*;

#[test]
fn runtime_configuration_requires_canonical_trusted_files() {
    assert!(matches!(
        PythonHelperRunner::new("python3", package_root()),
        Err(PythonHelperConfigurationError::InvalidExecutable)
    ));
    assert!(matches!(
        PythonHelperRunner::new(python_executable(), Path::new("python")),
        Err(PythonHelperConfigurationError::InvalidPackageRoot)
    ));
}

#[test]
fn isolated_worker_round_trip_is_typed_and_unicode_safe() {
    let runner = PythonHelperRunner::new(python_executable(), package_root()).unwrap();
    let registry = WorkerCancellationRegistry::new();
    let registration = registry.register().unwrap();
    let worker_id = registration.worker_id();
    let input = ContractProbeInput::new("Résumé").unwrap();

    let result = tauri::async_runtime::block_on(runner.run_contract_probe(input, registration));

    assert_eq!(result.unwrap().utf8_bytes(), 8);
    assert_eq!(
        registry.cancel(worker_id),
        Ok(CancelWorkerOutcome::AlreadyEnded)
    );
}

#[test]
fn text_analysis_round_trip_returns_explainable_non_destructive_findings() {
    let runner = PythonHelperRunner::new(python_executable(), package_root()).unwrap();
    let registry = WorkerCancellationRegistry::new();
    let registration = registry.register().unwrap();
    let input =
        TextAnalysisInput::new("Café café. Because this is IMPORTANT. Because we revise it.")
            .unwrap();

    let result =
        tauri::async_runtime::block_on(runner.run_text_analysis(input, registration)).unwrap();

    let categories = result
        .findings()
        .iter()
        .map(TextAnalysisFinding::category)
        .collect::<Vec<_>>();
    assert!(categories.contains(&TextAnalysisCategory::Grammar));
    assert!(categories.contains(&TextAnalysisCategory::Tone));
    assert!(categories.contains(&TextAnalysisCategory::Cohesion));
    assert!(result.findings().iter().all(|finding| {
        !finding.title().is_empty()
            && !finding.explanation().contains("Café")
            && finding.start_byte() < finding.end_byte()
    }));
}

#[test]
fn text_analysis_runs_while_connectivity_policy_is_offline() {
    let connectivity = ConnectivityPolicy::default();
    connectivity.set_mode(ConnectivityMode::Offline).unwrap();
    let runner = PythonHelperRunner::new(python_executable(), package_root()).unwrap();
    let registry = WorkerCancellationRegistry::new();
    let registration = registry.register().unwrap();

    let result = tauri::async_runtime::block_on(
        runner.run_text_analysis(
            TextAnalysisInput::new("Café café. Because this is IMPORTANT. Because we revise it.")
                .unwrap(),
            registration,
        ),
    );

    assert!(result.is_ok());
    assert_eq!(connectivity.mode(), Ok(ConnectivityMode::Offline));
}

#[cfg(target_os = "macos")]
#[test]
fn macos_system_python_executes_the_production_helper() {
    let runner = PythonHelperRunner::new(Path::new("/usr/bin/python3"), package_root()).unwrap();
    let registry = WorkerCancellationRegistry::new();
    let registration = registry.register().unwrap();

    let result = tauri::async_runtime::block_on(
        runner.run_text_analysis(TextAnalysisInput::new("Word word.").unwrap(), registration),
    );

    assert!(result.is_ok());
}

#[test]
fn overlapping_text_analysis_codes_keep_deterministic_wire_order() {
    let runner = PythonHelperRunner::new(python_executable(), package_root()).unwrap();
    let registry = WorkerCancellationRegistry::new();
    let registration = registry.register().unwrap();

    let result = tauri::async_runtime::block_on(runner.run_text_analysis(
        TextAnalysisInput::new("LOUDER LOUDER").unwrap(),
        registration,
    ))
    .unwrap();

    assert!(
        result
            .findings()
            .iter()
            .any(|finding| finding.code() == TextAnalysisFindingCode::RepeatedWord)
    );
    assert_eq!(result.findings().len(), 3);
}

#[test]
fn helper_environment_is_cleared_before_execution() {
    let runner = fixture_runner("environment", Duration::from_secs(2));
    let result = run_fixture(&runner);

    assert_eq!(result.unwrap().utf8_bytes(), 4);
}

#[test]
fn helper_timeout_kills_and_reaps_child() {
    let runner = fixture_runner("hang", Duration::from_millis(100));
    let started = Instant::now();

    let result = run_fixture(&runner);

    assert_eq!(result, Err(PythonHelperRunError::TimedOut));
    assert!(started.elapsed() < Duration::from_secs(3));
}

#[test]
fn helper_cancellation_kills_and_reaps_child() {
    let runner = fixture_runner("hang", Duration::from_secs(5));
    let registry = WorkerCancellationRegistry::new();
    let registration = registry.register().unwrap();
    let worker_id = registration.worker_id();
    let canceller = registry.clone();
    let cancellation_thread = thread::spawn(move || {
        thread::sleep(Duration::from_millis(100));
        canceller.cancel(worker_id).unwrap()
    });

    let result = tauri::async_runtime::block_on(
        runner.run_contract_probe(ContractProbeInput::new("text").unwrap(), registration),
    );

    assert_eq!(result, Err(PythonHelperRunError::Cancelled));
    assert_eq!(
        cancellation_thread.join().unwrap(),
        CancelWorkerOutcome::CancellationRequested
    );
    assert_eq!(
        registry.cancel(worker_id),
        Ok(CancelWorkerOutcome::AlreadyEnded)
    );
}

#[test]
fn malformed_excessive_and_stderr_output_fail_closed() {
    let malformed = run_fixture(&fixture_runner("malformed", Duration::from_secs(2)));
    assert_eq!(malformed, Err(PythonHelperRunError::InvalidOutput));

    let oversized = run_fixture(&fixture_runner("oversized", Duration::from_secs(2)));
    assert_eq!(oversized, Err(PythonHelperRunError::StdoutTooLarge));

    let stderr = run_fixture(&fixture_runner("stderr", Duration::from_secs(2)));
    assert_eq!(stderr, Err(PythonHelperRunError::InvalidOutput));
}

#[test]
fn nonzero_helper_failure_maps_to_closed_code() {
    let result = run_fixture(&fixture_runner("rejected", Duration::from_secs(2)));

    assert_eq!(
        result,
        Err(PythonHelperRunError::HelperRejected {
            code: PythonHelperFailureCode::InternalFailure,
        })
    );
}

#[test]
fn cancellation_before_spawn_avoids_process_work() {
    let runner = PythonHelperRunner::new(python_executable(), package_root()).unwrap();
    let registry = WorkerCancellationRegistry::new();
    let registration = registry.register().unwrap();
    registry.cancel(registration.worker_id()).unwrap();

    let result = tauri::async_runtime::block_on(
        runner.run_contract_probe(ContractProbeInput::new("text").unwrap(), registration),
    );

    assert_eq!(result, Err(PythonHelperRunError::Cancelled));
}

#[test]
fn runner_errors_do_not_expose_payload_stderr_or_paths() {
    assert_eq!(
        PythonHelperConfigurationError::InvalidEntrypoint.to_string(),
        "Python helper entrypoint is invalid"
    );
    assert_eq!(
        PythonHelperRunError::ExecutionFailed.to_string(),
        "Python helper execution failed"
    );
    assert_eq!(
        PythonHelperRunError::HelperRejected {
            code: PythonHelperFailureCode::InvalidRequest,
        }
        .to_string(),
        "Python helper rejected the request"
    );
}

fn run_fixture(runner: &PythonHelperRunner) -> Result<ContractProbeResult, PythonHelperRunError> {
    let registry = WorkerCancellationRegistry::new();
    let registration = registry.register().unwrap();
    tauri::async_runtime::block_on(
        runner.run_contract_probe(ContractProbeInput::new("text").unwrap(), registration),
    )
}

fn fixture_runner(mode: &str, timeout: Duration) -> PythonHelperRunner {
    PythonHelperRunner::for_fixture(&python_executable(), mode, timeout).unwrap()
}

fn python_executable() -> PathBuf {
    let output = Command::new("python3")
        .args(["-c", "import sys; print(sys.executable)"])
        .output()
        .expect("Python 3 should be available for the repository test suite");
    assert!(output.status.success());
    let path = String::from_utf8(output.stdout).unwrap();
    PathBuf::from(path.trim()).canonicalize().unwrap()
}

fn package_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../python")
        .canonicalize()
        .unwrap()
}
