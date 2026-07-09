use std::{error::Error, fmt};

use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{Map, Value};
use uuid::Uuid;

const SCHEMA_VERSION_FIELD: &str = "schema_version";
const DOCUMENT_ID_FIELD: &str = "document_id";
const TITLE_FIELD: &str = "title";
const DOCUMENT_FIELD: &str = "document";
const DOCUMENT_TYPE_FIELD: &str = "type";
const DOCUMENT_CONTENT_FIELD: &str = "content";
const DOCUMENT_ROOT_TYPE: &str = "doc";
const ENVELOPE_FIELDS: [&str; 4] = [
    SCHEMA_VERSION_FIELD,
    DOCUMENT_ID_FIELD,
    TITLE_FIELD,
    DOCUMENT_FIELD,
];

/// Current document-envelope schema accepted by the Rust core.
///
/// A different version must fail validation until an explicit migration owns it.
pub const DOCUMENT_ENVELOPE_SCHEMA_VERSION: u64 = 1;

/// Validated version 1 document envelope with no filesystem lifecycle.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct DocumentEnvelope {
    schema_version: u64,
    document_id: DocumentId,
    title: String,
    document: Value,
}

/// Opaque document identity parsed and serialized by the Rust core.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize)]
#[serde(transparent)]
pub struct DocumentId(Uuid);

/// Bounded failures produced while validating an untrusted document envelope.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum DocumentEnvelopeError {
    InvalidEnvelopeObject,
    UnknownEnvelopeField { field: String },
    MissingSchemaVersion,
    InvalidSchemaVersion,
    UnsupportedSchemaVersion { found: u64 },
    MissingDocumentId,
    InvalidDocumentId,
    MissingTitle,
    InvalidTitle,
    MissingDocument,
    InvalidDocumentRoot,
    InvalidDocumentContent,
}

impl DocumentEnvelope {
    /// Validates an untrusted JSON value without reading or writing application data.
    pub fn from_json_value(value: Value) -> Result<Self, DocumentEnvelopeError> {
        let mut fields = envelope_fields(value)?;
        reject_unknown_fields(&fields)?;

        Ok(Self {
            schema_version: parse_schema_version(take_schema_version(&mut fields)?)?,
            document_id: parse_document_id(take_document_id(&mut fields)?)?,
            title: parse_title(take_title(&mut fields)?)?,
            document: parse_document(take_document(&mut fields)?)?,
        })
    }

    /// Returns the validated schema version.
    pub fn schema_version(&self) -> u64 {
        self.schema_version
    }

    /// Returns the Rust-validated document identity.
    pub fn document_id(&self) -> DocumentId {
        self.document_id
    }

    /// Returns the original non-blank document title.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns the preserved structured Tiptap document value.
    pub fn document(&self) -> &Value {
        &self.document
    }
}

impl<'de> Deserialize<'de> for DocumentEnvelope {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        Self::from_json_value(value).map_err(serde::de::Error::custom)
    }
}

impl fmt::Display for DocumentId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl fmt::Display for DocumentEnvelopeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.message())
    }
}

impl Error for DocumentEnvelopeError {}

impl DocumentEnvelopeError {
    fn message(&self) -> &'static str {
        match self {
            Self::InvalidEnvelopeObject => "document envelope must be an object",
            Self::UnknownEnvelopeField { .. } => "document envelope contains an unknown field",
            Self::MissingSchemaVersion => "document envelope is missing schema_version",
            Self::InvalidSchemaVersion => "schema_version must be an unsigned integer",
            Self::UnsupportedSchemaVersion { .. } => "schema_version is not supported",
            Self::MissingDocumentId => "document envelope is missing document_id",
            Self::InvalidDocumentId => "document_id must be a UUID",
            Self::MissingTitle => "document envelope is missing title",
            Self::InvalidTitle => "title must be a non-blank string",
            Self::MissingDocument => "document envelope is missing document",
            Self::InvalidDocumentRoot => "document must be a Tiptap doc object",
            Self::InvalidDocumentContent => "document content must be an array",
        }
    }
}

fn envelope_fields(value: Value) -> Result<Map<String, Value>, DocumentEnvelopeError> {
    match value {
        Value::Object(fields) => Ok(fields),
        _ => Err(DocumentEnvelopeError::InvalidEnvelopeObject),
    }
}

fn reject_unknown_fields(fields: &Map<String, Value>) -> Result<(), DocumentEnvelopeError> {
    let unknown_field = fields
        .keys()
        .find(|field| !ENVELOPE_FIELDS.contains(&field.as_str()));

    match unknown_field {
        Some(field) => Err(DocumentEnvelopeError::UnknownEnvelopeField {
            field: field.clone(),
        }),
        None => Ok(()),
    }
}

fn take_schema_version(fields: &mut Map<String, Value>) -> Result<Value, DocumentEnvelopeError> {
    take_required_field(
        fields,
        SCHEMA_VERSION_FIELD,
        DocumentEnvelopeError::MissingSchemaVersion,
    )
}

fn take_document_id(fields: &mut Map<String, Value>) -> Result<Value, DocumentEnvelopeError> {
    take_required_field(
        fields,
        DOCUMENT_ID_FIELD,
        DocumentEnvelopeError::MissingDocumentId,
    )
}

fn take_title(fields: &mut Map<String, Value>) -> Result<Value, DocumentEnvelopeError> {
    take_required_field(fields, TITLE_FIELD, DocumentEnvelopeError::MissingTitle)
}

fn take_document(fields: &mut Map<String, Value>) -> Result<Value, DocumentEnvelopeError> {
    take_required_field(
        fields,
        DOCUMENT_FIELD,
        DocumentEnvelopeError::MissingDocument,
    )
}

fn take_required_field(
    fields: &mut Map<String, Value>,
    field_name: &str,
    missing_error: DocumentEnvelopeError,
) -> Result<Value, DocumentEnvelopeError> {
    fields.remove(field_name).ok_or(missing_error)
}

fn parse_schema_version(value: Value) -> Result<u64, DocumentEnvelopeError> {
    let version = value
        .as_u64()
        .ok_or(DocumentEnvelopeError::InvalidSchemaVersion)?;

    if version == DOCUMENT_ENVELOPE_SCHEMA_VERSION {
        Ok(version)
    } else {
        Err(DocumentEnvelopeError::UnsupportedSchemaVersion { found: version })
    }
}

fn parse_document_id(value: Value) -> Result<DocumentId, DocumentEnvelopeError> {
    let raw_id = value
        .as_str()
        .ok_or(DocumentEnvelopeError::InvalidDocumentId)?;
    Uuid::parse_str(raw_id)
        .map(DocumentId)
        .map_err(|_| DocumentEnvelopeError::InvalidDocumentId)
}

fn parse_title(value: Value) -> Result<String, DocumentEnvelopeError> {
    let Value::String(title) = value else {
        return Err(DocumentEnvelopeError::InvalidTitle);
    };

    if title.trim().is_empty() {
        Err(DocumentEnvelopeError::InvalidTitle)
    } else {
        Ok(title)
    }
}

fn parse_document(value: Value) -> Result<Value, DocumentEnvelopeError> {
    let document = value
        .as_object()
        .ok_or(DocumentEnvelopeError::InvalidDocumentRoot)?;
    validate_document_type(document)?;
    validate_document_content(document)?;
    Ok(value)
}

fn validate_document_type(document: &Map<String, Value>) -> Result<(), DocumentEnvelopeError> {
    match document.get(DOCUMENT_TYPE_FIELD).and_then(Value::as_str) {
        Some(DOCUMENT_ROOT_TYPE) => Ok(()),
        _ => Err(DocumentEnvelopeError::InvalidDocumentRoot),
    }
}

fn validate_document_content(document: &Map<String, Value>) -> Result<(), DocumentEnvelopeError> {
    match document.get(DOCUMENT_CONTENT_FIELD) {
        Some(Value::Array(_)) => Ok(()),
        _ => Err(DocumentEnvelopeError::InvalidDocumentContent),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    const DOCUMENT_ID: &str = "00000000-0000-4000-8000-000000000001";

    #[test]
    fn minimal_envelope_deserializes() {
        let envelope: DocumentEnvelope = serde_json::from_value(minimal_envelope())
            .expect("minimal envelope should deserialize");

        assert_eq!(envelope.schema_version(), DOCUMENT_ENVELOPE_SCHEMA_VERSION);
        assert_eq!(envelope.document_id().to_string(), DOCUMENT_ID);
        assert_eq!(envelope.title(), "Untitled document");
        assert_eq!(
            envelope.document(),
            &json!({ "type": "doc", "content": [] })
        );
    }

    #[test]
    fn envelope_serialization_is_stable() {
        let envelope = parse_envelope(minimal_envelope());

        assert_eq!(
            serde_json::to_value(envelope).expect("envelope should serialize"),
            minimal_envelope(),
        );
    }

    #[test]
    fn envelope_round_trip_is_stable() {
        let expected = parse_envelope(nested_envelope());
        let serialized = serde_json::to_value(&expected).expect("envelope should serialize");
        let actual: DocumentEnvelope =
            serde_json::from_value(serialized).expect("envelope should deserialize");

        assert_eq!(actual, expected);
    }

    #[test]
    fn missing_required_fields_fail_predictably() {
        assert_missing_field(
            SCHEMA_VERSION_FIELD,
            DocumentEnvelopeError::MissingSchemaVersion,
        );
        assert_missing_field(DOCUMENT_ID_FIELD, DocumentEnvelopeError::MissingDocumentId);
        assert_missing_field(TITLE_FIELD, DocumentEnvelopeError::MissingTitle);
        assert_missing_field(DOCUMENT_FIELD, DocumentEnvelopeError::MissingDocument);
    }

    #[test]
    fn non_object_envelope_fails() {
        assert_eq!(
            DocumentEnvelope::from_json_value(json!([])),
            Err(DocumentEnvelopeError::InvalidEnvelopeObject),
        );
    }

    #[test]
    fn unknown_top_level_fields_fail() {
        let mut value = minimal_envelope();
        value["references"] = json!([]);

        assert_eq!(
            DocumentEnvelope::from_json_value(value),
            Err(DocumentEnvelopeError::UnknownEnvelopeField {
                field: "references".to_owned(),
            }),
        );
    }

    #[test]
    fn unsupported_schema_versions_fail() {
        for version in [0, 2] {
            let mut value = minimal_envelope();
            value[SCHEMA_VERSION_FIELD] = json!(version);

            assert_eq!(
                DocumentEnvelope::from_json_value(value),
                Err(DocumentEnvelopeError::UnsupportedSchemaVersion { found: version }),
            );
        }
    }

    #[test]
    fn malformed_schema_versions_fail() {
        for version in [json!("1"), json!(1.5), json!(-1), json!(true)] {
            let mut value = minimal_envelope();
            value[SCHEMA_VERSION_FIELD] = version;

            assert_eq!(
                DocumentEnvelope::from_json_value(value),
                Err(DocumentEnvelopeError::InvalidSchemaVersion),
            );
        }
    }

    #[test]
    fn malformed_document_id_fails() {
        for document_id in [json!("not-a-uuid"), json!(7)] {
            let mut value = minimal_envelope();
            value[DOCUMENT_ID_FIELD] = document_id;

            assert_eq!(
                DocumentEnvelope::from_json_value(value),
                Err(DocumentEnvelopeError::InvalidDocumentId),
            );
        }
    }

    #[test]
    fn blank_title_fails() {
        for title in ["", "   ", "\n\t"] {
            let mut value = minimal_envelope();
            value[TITLE_FIELD] = json!(title);

            assert_eq!(
                DocumentEnvelope::from_json_value(value),
                Err(DocumentEnvelopeError::InvalidTitle),
            );
        }

        let mut value = minimal_envelope();
        value[TITLE_FIELD] = json!({ "text": "Title" });
        assert_eq!(
            DocumentEnvelope::from_json_value(value),
            Err(DocumentEnvelopeError::InvalidTitle),
        );
    }

    #[test]
    fn invalid_document_root_fails() {
        assert_document_error(json!([]), DocumentEnvelopeError::InvalidDocumentRoot);
        assert_document_error(
            json!({ "type": "paragraph", "content": [] }),
            DocumentEnvelopeError::InvalidDocumentRoot,
        );
    }

    #[test]
    fn invalid_document_content_fails() {
        assert_document_error(
            json!({ "type": "doc" }),
            DocumentEnvelopeError::InvalidDocumentContent,
        );
        assert_document_error(
            json!({ "type": "doc", "content": {} }),
            DocumentEnvelopeError::InvalidDocumentContent,
        );
    }

    #[test]
    fn unicode_and_nested_tiptap_json_round_trip() {
        let expected = nested_envelope();
        let envelope = parse_envelope(expected.clone());
        let actual = serde_json::to_value(envelope).expect("envelope should serialize");

        assert_eq!(actual, expected);
    }

    #[test]
    fn envelope_failure_shape_is_stable() {
        let errors = [
            DocumentEnvelopeError::MissingSchemaVersion,
            DocumentEnvelopeError::UnsupportedSchemaVersion { found: 2 },
            DocumentEnvelopeError::UnknownEnvelopeField {
                field: "references".to_owned(),
            },
            DocumentEnvelopeError::InvalidDocumentContent,
        ];

        assert_eq!(
            serde_json::to_value(errors).expect("errors should serialize"),
            json!([
                { "code": "missing_schema_version" },
                { "code": "unsupported_schema_version", "found": 2 },
                { "code": "unknown_envelope_field", "field": "references" },
                { "code": "invalid_document_content" }
            ]),
        );
    }

    fn parse_envelope(value: Value) -> DocumentEnvelope {
        DocumentEnvelope::from_json_value(value).expect("envelope should validate")
    }

    fn assert_missing_field(field: &str, expected_error: DocumentEnvelopeError) {
        let mut value = minimal_envelope();
        value
            .as_object_mut()
            .expect("envelope should be an object")
            .remove(field);

        assert_eq!(
            DocumentEnvelope::from_json_value(value),
            Err(expected_error)
        );
    }

    fn assert_document_error(document: Value, expected_error: DocumentEnvelopeError) {
        let mut value = minimal_envelope();
        value[DOCUMENT_FIELD] = document;

        assert_eq!(
            DocumentEnvelope::from_json_value(value),
            Err(expected_error)
        );
    }

    fn minimal_envelope() -> Value {
        json!({
            "schema_version": DOCUMENT_ENVELOPE_SCHEMA_VERSION,
            "document_id": DOCUMENT_ID,
            "title": "Untitled document",
            "document": { "type": "doc", "content": [] }
        })
    }

    fn nested_envelope() -> Value {
        json!({
            "schema_version": DOCUMENT_ENVELOPE_SCHEMA_VERSION,
            "document_id": DOCUMENT_ID,
            "title": "Résumé 日本語",
            "document": {
                "type": "doc",
                "content": [{
                    "type": "paragraph",
                    "attrs": { "alignment": null, "level": 2 },
                    "content": [{ "type": "text", "text": "naïve — 日本語" }]
                }]
            }
        })
    }
}
