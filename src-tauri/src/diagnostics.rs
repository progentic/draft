use serde::Serialize;

use crate::{
    application::runtime_status::current_runtime_status,
    citations::node::CITATION_NODE_SCHEMA_VERSION,
    documents::envelope::DOCUMENT_ENVELOPE_SCHEMA_VERSION,
    jobs::store::JOB_STORE_SCHEMA_VERSION,
    references::{record::REFERENCE_RECORD_SCHEMA_VERSION, store::REFERENCE_STORE_SCHEMA_VERSION},
    workers::python::PYTHON_HELPER_PROTOCOL_VERSION,
};

/// Current schema for the local diagnostic snapshot.
pub(crate) const DIAGNOSTIC_SNAPSHOT_SCHEMA_VERSION: u16 = 1;
/// Maximum serialized size of one diagnostic snapshot.
pub(crate) const MAX_DIAGNOSTIC_SNAPSHOT_BYTES: usize = 2 * 1_024;
const MAX_APPLICATION_VERSION_BYTES: usize = 64;
const CONTRACT_VERSIONS: [ContractVersion; 6] = [
    ContractVersion::new(ContractName::CitationNode, CITATION_NODE_SCHEMA_VERSION),
    ContractVersion::new(
        ContractName::DocumentEnvelope,
        DOCUMENT_ENVELOPE_SCHEMA_VERSION,
    ),
    ContractVersion::new(ContractName::PdfImportJobStore, JOB_STORE_SCHEMA_VERSION),
    ContractVersion::new(
        ContractName::PythonHelperProtocol,
        PYTHON_HELPER_PROTOCOL_VERSION as u64,
    ),
    ContractVersion::new(
        ContractName::ReferenceRecord,
        REFERENCE_RECORD_SCHEMA_VERSION,
    ),
    ContractVersion::new(ContractName::ReferenceStore, REFERENCE_STORE_SCHEMA_VERSION),
];
const SUBSYSTEM_AVAILABILITY: [SubsystemAvailability; 6] = [
    SubsystemAvailability::new(SubsystemName::CoreRuntime, AvailabilityStatus::Ready),
    SubsystemAvailability::new(SubsystemName::DocumentRegistry, AvailabilityStatus::Ready),
    SubsystemAvailability::new(SubsystemName::NetworkClient, AvailabilityStatus::Ready),
    SubsystemAvailability::new(SubsystemName::PdfImportJobStore, AvailabilityStatus::Ready),
    SubsystemAvailability::new(SubsystemName::PythonHelper, AvailabilityStatus::NotChecked),
    SubsystemAvailability::new(SubsystemName::ReferenceStore, AvailabilityStatus::Ready),
];

/// Local, content-free support metadata assembled by the Rust core.
#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DiagnosticSnapshot {
    schema_version: u16,
    application_version: String,
    contract_versions: [ContractVersion; 6],
    subsystems: [SubsystemAvailability; 6],
}

/// Closed failures produced while assembling a diagnostic snapshot.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DiagnosticSnapshotError {
    InvalidApplicationVersion,
    SerializationFailed,
    SnapshotTooLarge,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct ContractVersion {
    name: ContractName,
    version: u64,
}

impl ContractVersion {
    const fn new(name: ContractName, version: u64) -> Self {
        Self { name, version }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum ContractName {
    CitationNode,
    DocumentEnvelope,
    PdfImportJobStore,
    PythonHelperProtocol,
    ReferenceRecord,
    ReferenceStore,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct SubsystemAvailability {
    name: SubsystemName,
    status: AvailabilityStatus,
}

impl SubsystemAvailability {
    const fn new(name: SubsystemName, status: AvailabilityStatus) -> Self {
        Self { name, status }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum SubsystemName {
    CoreRuntime,
    DocumentRegistry,
    NetworkClient,
    PdfImportJobStore,
    PythonHelper,
    ReferenceStore,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum AvailabilityStatus {
    Ready,
    NotChecked,
}

/// Assembles the fixed diagnostic snapshot without probing external state.
pub(crate) fn current_diagnostic_snapshot() -> Result<DiagnosticSnapshot, DiagnosticSnapshotError> {
    let status =
        current_runtime_status().map_err(|_| DiagnosticSnapshotError::InvalidApplicationVersion)?;
    diagnostic_snapshot(status.into_version())
}

fn diagnostic_snapshot(
    application_version: String,
) -> Result<DiagnosticSnapshot, DiagnosticSnapshotError> {
    validate_application_version(&application_version)?;
    let snapshot = DiagnosticSnapshot {
        schema_version: DIAGNOSTIC_SNAPSHOT_SCHEMA_VERSION,
        application_version,
        contract_versions: CONTRACT_VERSIONS,
        subsystems: SUBSYSTEM_AVAILABILITY,
    };
    validate_serialized_size(&snapshot)?;
    Ok(snapshot)
}

fn validate_application_version(version: &str) -> Result<(), DiagnosticSnapshotError> {
    if version.is_empty() || version.len() > MAX_APPLICATION_VERSION_BYTES {
        return Err(DiagnosticSnapshotError::InvalidApplicationVersion);
    }
    Ok(())
}

fn validate_serialized_size(snapshot: &DiagnosticSnapshot) -> Result<(), DiagnosticSnapshotError> {
    let bytes =
        serde_json::to_vec(snapshot).map_err(|_| DiagnosticSnapshotError::SerializationFailed)?;
    if bytes.len() > MAX_DIAGNOSTIC_SNAPSHOT_BYTES {
        return Err(DiagnosticSnapshotError::SnapshotTooLarge);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn snapshot_schema_is_strict_versioned_and_deterministic() {
        let first = current_diagnostic_snapshot().expect("snapshot should be valid");
        let second = current_diagnostic_snapshot().expect("snapshot should be repeatable");

        assert_eq!(first, second);
        assert_eq!(serde_json::to_value(first).unwrap(), expected_snapshot());
    }

    #[test]
    fn serialized_snapshot_is_bounded() {
        let snapshot = current_diagnostic_snapshot().expect("snapshot should be valid");
        let serialized = serde_json::to_vec(&snapshot).expect("snapshot should serialize");

        assert!(serialized.len() <= MAX_DIAGNOSTIC_SNAPSHOT_BYTES);
    }

    #[test]
    fn snapshot_contains_no_redacted_categories() {
        let snapshot = current_diagnostic_snapshot().expect("snapshot should be valid");
        let value = serde_json::to_value(snapshot).expect("snapshot should serialize");
        let fields = collect_field_names(&value);

        for denied in [
            "content",
            "credential",
            "environment",
            "hostname",
            "log",
            "path",
            "secret",
            "url",
            "username",
        ] {
            assert!(!fields.iter().any(|field| field.contains(denied)));
        }
    }

    #[test]
    fn invalid_application_versions_fail_with_closed_error() {
        assert_eq!(
            diagnostic_snapshot(String::new()),
            Err(DiagnosticSnapshotError::InvalidApplicationVersion),
        );
        assert_eq!(
            diagnostic_snapshot("v".repeat(MAX_APPLICATION_VERSION_BYTES + 1)),
            Err(DiagnosticSnapshotError::InvalidApplicationVersion),
        );
    }

    fn expected_snapshot() -> serde_json::Value {
        json!({
            "schemaVersion": 1,
            "applicationVersion": env!("CARGO_PKG_VERSION"),
            "contractVersions": expected_contract_versions(),
            "subsystems": expected_subsystems()
        })
    }

    fn expected_contract_versions() -> serde_json::Value {
        json!([
            { "name": "citation_node", "version": 1 },
            { "name": "document_envelope", "version": 1 },
            { "name": "pdf_import_job_store", "version": 1 },
            { "name": "python_helper_protocol", "version": 1 },
            { "name": "reference_record", "version": 1 },
            { "name": "reference_store", "version": 1 }
        ])
    }

    fn expected_subsystems() -> serde_json::Value {
        json!([
            { "name": "core_runtime", "status": "ready" },
            { "name": "document_registry", "status": "ready" },
            { "name": "network_client", "status": "ready" },
            { "name": "pdf_import_job_store", "status": "ready" },
            { "name": "python_helper", "status": "not_checked" },
            { "name": "reference_store", "status": "ready" }
        ])
    }

    fn collect_field_names(value: &serde_json::Value) -> Vec<String> {
        match value {
            serde_json::Value::Object(fields) => fields
                .iter()
                .flat_map(|(name, value)| {
                    std::iter::once(name.clone()).chain(collect_field_names(value))
                })
                .collect(),
            serde_json::Value::Array(values) => {
                values.iter().flat_map(collect_field_names).collect()
            }
            _ => Vec::new(),
        }
    }
}
