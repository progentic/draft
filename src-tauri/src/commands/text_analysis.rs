use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};

use crate::workers::{
    cancellation::{WorkerCancellationError, WorkerCancellationRegistry},
    python::{
        PythonHelperConfigurationError, PythonHelperRequestError, PythonHelperRunError,
        PythonHelperRunner, TextAnalysisInput, TextAnalysisResult,
    },
};

const PYTHON_EXECUTABLE: &str = "/usr/bin/python3";

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct RunTextAnalysisRequest {
    text: String,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RunTextAnalysisResponse {
    result: TextAnalysisResult,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "code", rename_all = "snake_case")]
pub(crate) enum RunTextAnalysisError {
    EmptyText,
    TextTooLong,
    RuntimeUnavailable,
    WorkerUnavailable,
    TimedOut,
    Cancelled,
    HelperFailed,
    InvalidOutput,
}

#[tauri::command]
pub(crate) async fn run_text_analysis(
    app_handle: AppHandle,
    workers: State<'_, WorkerCancellationRegistry>,
    request: RunTextAnalysisRequest,
) -> Result<RunTextAnalysisResponse, RunTextAnalysisError> {
    let input = TextAnalysisInput::new(request.text).map_err(map_request_error)?;
    let runner = runner_for(&app_handle)?;
    let registration = workers.register().map_err(map_worker_error)?;
    runner
        .run_text_analysis(input, registration)
        .await
        .map(|result| RunTextAnalysisResponse { result })
        .map_err(map_run_error)
}

fn runner_for(app_handle: &AppHandle) -> Result<PythonHelperRunner, RunTextAnalysisError> {
    let resources = app_handle
        .path()
        .resource_dir()
        .map_err(|_| RunTextAnalysisError::RuntimeUnavailable)?;
    configured_runner(Path::new(PYTHON_EXECUTABLE), resources.join("python"))
}

fn configured_runner(
    executable: &Path,
    package_root: PathBuf,
) -> Result<PythonHelperRunner, RunTextAnalysisError> {
    PythonHelperRunner::new(executable, package_root).map_err(map_configuration_error)
}

fn map_request_error(error: PythonHelperRequestError) -> RunTextAnalysisError {
    match error {
        PythonHelperRequestError::EmptyText => RunTextAnalysisError::EmptyText,
        PythonHelperRequestError::TextTooLong => RunTextAnalysisError::TextTooLong,
    }
}

fn map_configuration_error(_: PythonHelperConfigurationError) -> RunTextAnalysisError {
    RunTextAnalysisError::RuntimeUnavailable
}

fn map_worker_error(_: WorkerCancellationError) -> RunTextAnalysisError {
    RunTextAnalysisError::WorkerUnavailable
}

fn map_run_error(error: PythonHelperRunError) -> RunTextAnalysisError {
    match error {
        PythonHelperRunError::TimedOut => RunTextAnalysisError::TimedOut,
        PythonHelperRunError::Cancelled => RunTextAnalysisError::Cancelled,
        PythonHelperRunError::InvalidOutput | PythonHelperRunError::ResponseMismatch => {
            RunTextAnalysisError::InvalidOutput
        }
        _ => RunTextAnalysisError::HelperFailed,
    }
}

#[cfg(test)]
mod tests {
    use std::future::Future;

    use serde_json::json;

    use super::*;

    fn typed_command<'a>(
        app_handle: AppHandle,
        workers: State<'a, WorkerCancellationRegistry>,
        request: RunTextAnalysisRequest,
    ) -> impl Future<Output = Result<RunTextAnalysisResponse, RunTextAnalysisError>> + 'a {
        run_text_analysis(app_handle, workers, request)
    }

    #[test]
    fn command_signature_is_typed() {
        let _ = typed_command;
    }

    #[test]
    fn request_deserialization_is_stable() {
        let request: RunTextAnalysisRequest =
            serde_json::from_value(json!({"text": "Words"})).unwrap();
        assert_eq!(request.text, "Words");
        assert!(
            serde_json::from_value::<RunTextAnalysisRequest>(json!({
                "text": "Words", "provider": "external"
            }))
            .is_err()
        );
    }

    #[test]
    fn response_serialization_is_stable() {
        let response = RunTextAnalysisResponse {
            result: TextAnalysisResult::empty_for_test(),
        };
        assert_eq!(
            serde_json::to_value(response).unwrap(),
            json!({ "result": { "findings": [] } })
        );
    }

    #[test]
    fn error_serialization_is_stable() {
        assert_eq!(
            serde_json::to_value(RunTextAnalysisError::RuntimeUnavailable).unwrap(),
            json!({ "code": "runtime_unavailable" })
        );
    }

    #[test]
    fn request_and_runner_errors_map_without_details() {
        assert_eq!(
            map_request_error(PythonHelperRequestError::EmptyText),
            RunTextAnalysisError::EmptyText
        );
        assert_eq!(
            map_run_error(PythonHelperRunError::TimedOut),
            RunTextAnalysisError::TimedOut
        );
        assert_eq!(
            map_run_error(PythonHelperRunError::SpawnFailed),
            RunTextAnalysisError::HelperFailed
        );
    }
}
