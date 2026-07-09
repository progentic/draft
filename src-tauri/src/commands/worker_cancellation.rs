use serde::{Deserialize, Serialize};
use tauri::State;

use crate::workers::cancellation::{
    CancelWorkerOutcome, InvalidWorkerId, WorkerCancellationError, WorkerCancellationRegistry,
    WorkerId,
};

/// Bounded request for cancelling one Rust-owned transient worker.
#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(crate) struct CancelWorkerRequest {
    worker_id: String,
}

/// Observable result of an idempotent worker cancellation request.
#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub(crate) enum CancelWorkerResponse {
    CancellationRequested,
    AlreadyEnded,
}

/// Bounded failures returned by the worker cancellation command.
#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "code", rename_all = "snake_case")]
pub(crate) enum CancelWorkerError {
    InvalidWorkerId,
    WorkerNotFound,
    RegistryUnavailable,
}

/// Requests cancellation for a Rust-owned worker without exposing registry internals.
#[tauri::command]
pub(crate) fn cancel_worker(
    registry: State<'_, WorkerCancellationRegistry>,
    request: CancelWorkerRequest,
) -> Result<CancelWorkerResponse, CancelWorkerError> {
    cancel_worker_with_registry(&registry, request)
}

fn cancel_worker_with_registry(
    registry: &WorkerCancellationRegistry,
    request: CancelWorkerRequest,
) -> Result<CancelWorkerResponse, CancelWorkerError> {
    let worker_id = WorkerId::parse(&request.worker_id)?;
    Ok(registry.cancel(worker_id)?.into())
}

impl From<CancelWorkerOutcome> for CancelWorkerResponse {
    fn from(outcome: CancelWorkerOutcome) -> Self {
        match outcome {
            CancelWorkerOutcome::CancellationRequested => Self::CancellationRequested,
            CancelWorkerOutcome::AlreadyEnded => Self::AlreadyEnded,
        }
    }
}

impl From<InvalidWorkerId> for CancelWorkerError {
    fn from(_: InvalidWorkerId) -> Self {
        Self::InvalidWorkerId
    }
}

impl From<WorkerCancellationError> for CancelWorkerError {
    fn from(error: WorkerCancellationError) -> Self {
        match error {
            WorkerCancellationError::WorkerNotFound => Self::WorkerNotFound,
            WorkerCancellationError::RegistryUnavailable => Self::RegistryUnavailable,
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    const UNKNOWN_WORKER_ID: &str = "00000000-0000-4000-8000-000000000001";
    const TYPED_COMMAND: for<'a> fn(
        State<'a, WorkerCancellationRegistry>,
        CancelWorkerRequest,
    ) -> Result<CancelWorkerResponse, CancelWorkerError> = cancel_worker;

    #[test]
    fn command_signature_is_typed() {
        let _ = TYPED_COMMAND;
    }

    #[test]
    fn request_deserialization_is_stable() {
        let request: CancelWorkerRequest = serde_json::from_value(json!({
            "workerId": UNKNOWN_WORKER_ID
        }))
        .expect("request should deserialize");

        assert_eq!(request.worker_id, UNKNOWN_WORKER_ID);
        assert!(
            serde_json::from_value::<CancelWorkerRequest>(json!({
                "workerId": UNKNOWN_WORKER_ID,
                "unexpected": true
            }))
            .is_err()
        );
    }

    #[test]
    fn response_serialization_is_stable() {
        let responses = [
            CancelWorkerResponse::CancellationRequested,
            CancelWorkerResponse::AlreadyEnded,
        ];

        assert_eq!(
            serde_json::to_value(responses).expect("responses should serialize"),
            json!([
                { "status": "cancellation_requested" },
                { "status": "already_ended" }
            ]),
        );
    }

    #[test]
    fn error_serialization_is_stable() {
        let errors = [
            CancelWorkerError::InvalidWorkerId,
            CancelWorkerError::WorkerNotFound,
            CancelWorkerError::RegistryUnavailable,
        ];

        assert_eq!(
            serde_json::to_value(errors).expect("errors should serialize"),
            json!([
                { "code": "invalid_worker_id" },
                { "code": "worker_not_found" },
                { "code": "registry_unavailable" }
            ]),
        );
    }

    #[test]
    fn cancellation_requests_active_worker() {
        let registry = WorkerCancellationRegistry::new();
        let registration = registry.register().expect("worker should register");
        let cancellation = registration.cancellation();

        let response = cancel_registered_worker(&registry, registration.worker_id());

        assert_eq!(response, Ok(CancelWorkerResponse::CancellationRequested));
        assert!(cancellation.is_cancelled());
    }

    #[test]
    fn repeated_cancellation_is_idempotent() {
        let registry = WorkerCancellationRegistry::new();
        let registration = registry.register().expect("worker should register");

        let first = cancel_registered_worker(&registry, registration.worker_id());
        let second = cancel_registered_worker(&registry, registration.worker_id());

        assert_eq!(first, Ok(CancelWorkerResponse::CancellationRequested));
        assert_eq!(second, first);
    }

    #[test]
    fn cancellation_of_ended_worker_is_idempotent() {
        let registry = WorkerCancellationRegistry::new();
        let registration = registry.register().expect("worker should register");
        let worker_id = registration.worker_id();
        drop(registration);

        let first = cancel_registered_worker(&registry, worker_id);
        let second = cancel_registered_worker(&registry, worker_id);

        assert_eq!(first, Ok(CancelWorkerResponse::AlreadyEnded));
        assert_eq!(second, first);
    }

    #[test]
    fn cancellation_of_unknown_worker_returns_error() {
        let registry = WorkerCancellationRegistry::new();

        assert_eq!(
            cancel_worker_with_registry(&registry, request_for(UNKNOWN_WORKER_ID)),
            Err(CancelWorkerError::WorkerNotFound),
        );
    }

    #[test]
    fn malformed_worker_id_returns_error() {
        let registry = WorkerCancellationRegistry::new();

        assert_eq!(
            cancel_worker_with_registry(&registry, request_for("not-a-worker-id")),
            Err(CancelWorkerError::InvalidWorkerId),
        );
    }

    fn cancel_registered_worker(
        registry: &WorkerCancellationRegistry,
        worker_id: WorkerId,
    ) -> Result<CancelWorkerResponse, CancelWorkerError> {
        cancel_worker_with_registry(registry, request_for(&worker_id.to_string()))
    }

    fn request_for(worker_id: &str) -> CancelWorkerRequest {
        CancelWorkerRequest {
            worker_id: worker_id.to_owned(),
        }
    }
}
