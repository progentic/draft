use serde::{Deserialize, Serialize};

use crate::diagnostics::{
    DiagnosticSnapshot, DiagnosticSnapshotError, current_diagnostic_snapshot,
};

/// Bounded request envelope for `get_diagnostic_snapshot`.
#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct GetDiagnosticSnapshotRequest {}

/// Content-free support metadata returned by the Rust core.
#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub(crate) struct GetDiagnosticSnapshotResponse(DiagnosticSnapshot);

/// Stable error codes exposed by `get_diagnostic_snapshot` over Tauri IPC.
#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "code", rename_all = "snake_case")]
pub(crate) enum GetDiagnosticSnapshotError {
    InvalidApplicationVersion,
    SnapshotSerializationFailed,
    SnapshotTooLarge,
}

/// Returns one local diagnostic snapshot without probing external state.
#[tauri::command]
pub(crate) fn get_diagnostic_snapshot(
    request: GetDiagnosticSnapshotRequest,
) -> Result<GetDiagnosticSnapshotResponse, GetDiagnosticSnapshotError> {
    let GetDiagnosticSnapshotRequest {} = request;
    current_diagnostic_snapshot()
        .map(GetDiagnosticSnapshotResponse)
        .map_err(GetDiagnosticSnapshotError::from)
}

impl From<DiagnosticSnapshotError> for GetDiagnosticSnapshotError {
    fn from(error: DiagnosticSnapshotError) -> Self {
        match error {
            DiagnosticSnapshotError::InvalidApplicationVersion => Self::InvalidApplicationVersion,
            DiagnosticSnapshotError::SerializationFailed => Self::SnapshotSerializationFailed,
            DiagnosticSnapshotError::SnapshotTooLarge => Self::SnapshotTooLarge,
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    const TYPED_COMMAND: fn(
        GetDiagnosticSnapshotRequest,
    )
        -> Result<GetDiagnosticSnapshotResponse, GetDiagnosticSnapshotError> =
        get_diagnostic_snapshot;

    #[test]
    fn command_signature_is_typed() {
        assert_eq!(
            std::any::type_name_of_val(&TYPED_COMMAND),
            std::any::type_name::<
                fn(
                    GetDiagnosticSnapshotRequest,
                )
                    -> Result<GetDiagnosticSnapshotResponse, GetDiagnosticSnapshotError>,
            >(),
        );
    }

    #[test]
    fn request_deserialization_is_stable() {
        let request = serde_json::from_value::<GetDiagnosticSnapshotRequest>(json!({}));
        let unknown =
            serde_json::from_value::<GetDiagnosticSnapshotRequest>(json!({ "extra": true }));

        assert_eq!(request.unwrap(), GetDiagnosticSnapshotRequest {});
        assert!(unknown.is_err());
    }

    #[test]
    fn response_serialization_is_stable() {
        let response = get_diagnostic_snapshot(GetDiagnosticSnapshotRequest {})
            .expect("diagnostic snapshot should succeed");
        let value = serde_json::to_value(response).expect("response should serialize");

        assert_eq!(value["schemaVersion"], 1);
        assert_eq!(value["applicationVersion"], env!("CARGO_PKG_VERSION"));
        assert_eq!(value["contractVersions"].as_array().unwrap().len(), 6);
        assert_eq!(value["subsystems"].as_array().unwrap().len(), 6);
    }

    #[test]
    fn error_serialization_is_stable() {
        let errors = [
            GetDiagnosticSnapshotError::from(DiagnosticSnapshotError::InvalidApplicationVersion),
            GetDiagnosticSnapshotError::from(DiagnosticSnapshotError::SerializationFailed),
            GetDiagnosticSnapshotError::from(DiagnosticSnapshotError::SnapshotTooLarge),
        ];

        assert_eq!(
            serde_json::to_value(errors).unwrap(),
            json!([
                { "code": "invalid_application_version" },
                { "code": "snapshot_serialization_failed" },
                { "code": "snapshot_too_large" }
            ]),
        );
    }
}
