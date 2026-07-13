use serde::Serialize;
use serde_json::Value;

const SCHEMA_VERSION_FIELD: &str = "schema_version";
const PARAGRAPH_STYLE_FIELD: &str = "paragraphStyle";

pub(crate) const LEGACY_DOCUMENT_ENVELOPE_SCHEMA_VERSION: u64 = 1;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum DocumentMigrationError {
    InvalidLegacyEnvelope,
    ParagraphStyleInLegacyEnvelope,
}

pub(crate) fn migrate_v1_to_v2(mut value: Value) -> Result<Value, DocumentMigrationError> {
    require_legacy_version(&value)?;
    reject_legacy_paragraph_style(&value)?;
    let fields = value
        .as_object_mut()
        .ok_or(DocumentMigrationError::InvalidLegacyEnvelope)?;
    fields.insert(SCHEMA_VERSION_FIELD.to_owned(), Value::from(2));
    Ok(value)
}

fn require_legacy_version(value: &Value) -> Result<(), DocumentMigrationError> {
    let version = value
        .as_object()
        .and_then(|fields| fields.get(SCHEMA_VERSION_FIELD))
        .and_then(Value::as_u64);
    if version == Some(LEGACY_DOCUMENT_ENVELOPE_SCHEMA_VERSION) {
        Ok(())
    } else {
        Err(DocumentMigrationError::InvalidLegacyEnvelope)
    }
}

fn reject_legacy_paragraph_style(value: &Value) -> Result<(), DocumentMigrationError> {
    match value {
        Value::Object(fields) => reject_object_paragraph_style(fields),
        Value::Array(values) => reject_array_paragraph_style(values),
        _ => Ok(()),
    }
}

fn reject_object_paragraph_style(
    fields: &serde_json::Map<String, Value>,
) -> Result<(), DocumentMigrationError> {
    if fields.contains_key(PARAGRAPH_STYLE_FIELD) {
        return Err(DocumentMigrationError::ParagraphStyleInLegacyEnvelope);
    }
    fields.values().try_for_each(reject_legacy_paragraph_style)
}

fn reject_array_paragraph_style(values: &[Value]) -> Result<(), DocumentMigrationError> {
    values.iter().try_for_each(reject_legacy_paragraph_style)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn migration_changes_only_the_envelope_version() {
        let source = json!({
            "schema_version": 1,
            "document": { "type": "doc", "content": [{ "type": "paragraph" }] }
        });
        let migrated = migrate_v1_to_v2(source.clone()).unwrap();

        assert_eq!(source["schema_version"], 1);
        assert_eq!(migrated["schema_version"], 2);
        assert_eq!(migrated["document"], source["document"]);
        assert_eq!(migrate_v1_to_v2(source).unwrap(), migrated);
    }

    #[test]
    fn legacy_paragraph_data_fails_instead_of_being_guessed() {
        let source = json!({
            "schema_version": 1,
            "document": {
                "type": "doc",
                "content": [{
                    "type": "paragraph",
                    "attrs": { "paragraphStyle": {} }
                }]
            }
        });

        assert_eq!(
            migrate_v1_to_v2(source),
            Err(DocumentMigrationError::ParagraphStyleInLegacyEnvelope)
        );
    }

    #[test]
    fn malformed_or_nonlegacy_inputs_fail_the_named_step() {
        for source in [json!(null), json!({}), json!({ "schema_version": 2 })] {
            assert_eq!(
                migrate_v1_to_v2(source),
                Err(DocumentMigrationError::InvalidLegacyEnvelope)
            );
        }
    }
}
