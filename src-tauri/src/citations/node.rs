use std::{error::Error, fmt};

use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{Map, Value};

use crate::references::record::is_valid_citekey;

const SCHEMA_VERSION_FIELD: &str = "schema_version";
const CITEKEY_FIELD: &str = "citekey";
const RENDER_STYLE_FIELD: &str = "render_style";
const NODE_TYPE_FIELD: &str = "type";
const NODE_ATTRS_FIELD: &str = "attrs";
const NODE_CONTENT_FIELD: &str = "content";
const CITATION_NODE_TYPE: &str = "citation";
const CITATION_NODE_FIELDS: [&str; 2] = [NODE_TYPE_FIELD, NODE_ATTRS_FIELD];
const CITATION_ATTR_FIELDS: [&str; 3] = [SCHEMA_VERSION_FIELD, CITEKEY_FIELD, RENDER_STYLE_FIELD];

/// Current citation-node attribute schema accepted by the Rust core.
pub const CITATION_NODE_SCHEMA_VERSION: u64 = 1;

/// Validated attributes for one version 1 Tiptap citation node.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CitationNodeAttributes {
    schema_version: u64,
    citekey: String,
    render_style: CitationRenderStyle,
}

/// Supported citation rendering requests for schema version 1.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CitationRenderStyle {
    Apa7,
}

/// Bounded failures produced while validating citation-node data.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum CitationNodeError {
    UnknownCitationNodeField { field: String },
    MissingCitationAttrs,
    InvalidCitationAttrsObject,
    UnknownCitationAttr { field: String },
    MissingSchemaVersion,
    InvalidSchemaVersion,
    UnsupportedSchemaVersion { found: u64 },
    MissingCitekey,
    InvalidCitekey,
    MissingRenderStyle,
    UnsupportedRenderStyle,
}

/// Location and cause for one invalid citation nested in document JSON.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct LocatedCitationError {
    pub(crate) path: String,
    pub(crate) cause: CitationNodeError,
}

impl CitationNodeAttributes {
    /// Validates one untrusted attrs value without persistence or rendering.
    pub fn from_json_value(value: Value) -> Result<Self, CitationNodeError> {
        let mut fields = citation_attribute_fields(value)?;
        reject_unknown_attributes(&fields)?;

        Ok(Self {
            schema_version: parse_schema_version(take_schema_version(&mut fields)?)?,
            citekey: parse_citekey(take_citekey(&mut fields)?)?,
            render_style: parse_render_style(take_render_style(&mut fields)?)?,
        })
    }

    pub fn schema_version(&self) -> u64 {
        self.schema_version
    }

    pub fn citekey(&self) -> &str {
        &self.citekey
    }

    pub fn render_style(&self) -> CitationRenderStyle {
        self.render_style
    }
}

impl<'de> Deserialize<'de> for CitationNodeAttributes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        Self::from_json_value(value).map_err(serde::de::Error::custom)
    }
}

impl fmt::Display for CitationNodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.message())
    }
}

impl Error for CitationNodeError {}

impl CitationNodeError {
    fn message(&self) -> &'static str {
        match self {
            Self::UnknownCitationNodeField { .. } => "citation node contains an unknown field",
            Self::MissingCitationAttrs => "citation node is missing attrs",
            Self::InvalidCitationAttrsObject => "citation attrs must be an object",
            Self::UnknownCitationAttr { .. } => "citation attrs contain an unknown field",
            Self::MissingSchemaVersion => "citation attrs are missing schema_version",
            Self::InvalidSchemaVersion => "citation schema_version must be an unsigned integer",
            Self::UnsupportedSchemaVersion { .. } => "citation schema_version is not supported",
            Self::MissingCitekey => "citation attrs are missing citekey",
            Self::InvalidCitekey => "citation citekey has an invalid shape",
            Self::MissingRenderStyle => "citation attrs are missing render_style",
            Self::UnsupportedRenderStyle => "citation render_style is not supported",
        }
    }
}

/// Validates every citation node reachable through Tiptap `content` arrays.
pub(crate) fn validate_document_citations(document: &Value) -> Result<(), LocatedCitationError> {
    document_citations(document).map(drop)
}

/// Returns validated citation attrs in document order.
pub(crate) fn document_citations(
    document: &Value,
) -> Result<Vec<CitationNodeAttributes>, LocatedCitationError> {
    let mut citations = Vec::new();
    collect_document_node(document, "document", &mut citations)?;
    Ok(citations)
}

fn collect_document_node(
    value: &Value,
    path: &str,
    citations: &mut Vec<CitationNodeAttributes>,
) -> Result<(), LocatedCitationError> {
    let Some(node) = value.as_object() else {
        return Ok(());
    };
    if is_citation_node(node) {
        let citation = validate_citation_node(node).map_err(|cause| LocatedCitationError {
            path: path.to_owned(),
            cause,
        })?;
        citations.push(citation);
    }
    collect_child_nodes(node, path, citations)
}

fn collect_child_nodes(
    node: &Map<String, Value>,
    path: &str,
    citations: &mut Vec<CitationNodeAttributes>,
) -> Result<(), LocatedCitationError> {
    let Some(Value::Array(children)) = node.get(NODE_CONTENT_FIELD) else {
        return Ok(());
    };
    for (index, child) in children.iter().enumerate() {
        collect_document_node(child, &child_path(path, index), citations)?;
    }
    Ok(())
}

fn child_path(parent: &str, index: usize) -> String {
    format!("{parent}.content[{index}]")
}

fn is_citation_node(node: &Map<String, Value>) -> bool {
    node.get(NODE_TYPE_FIELD).and_then(Value::as_str) == Some(CITATION_NODE_TYPE)
}

fn validate_citation_node(
    node: &Map<String, Value>,
) -> Result<CitationNodeAttributes, CitationNodeError> {
    reject_unknown_node_fields(node)?;
    let attrs = node
        .get(NODE_ATTRS_FIELD)
        .cloned()
        .ok_or(CitationNodeError::MissingCitationAttrs)?;
    CitationNodeAttributes::from_json_value(attrs)
}

fn reject_unknown_node_fields(node: &Map<String, Value>) -> Result<(), CitationNodeError> {
    match first_unknown_field(node, &CITATION_NODE_FIELDS) {
        Some(field) => Err(CitationNodeError::UnknownCitationNodeField { field }),
        None => Ok(()),
    }
}

fn citation_attribute_fields(value: Value) -> Result<Map<String, Value>, CitationNodeError> {
    value
        .as_object()
        .cloned()
        .ok_or(CitationNodeError::InvalidCitationAttrsObject)
}

fn reject_unknown_attributes(fields: &Map<String, Value>) -> Result<(), CitationNodeError> {
    match first_unknown_field(fields, &CITATION_ATTR_FIELDS) {
        Some(field) => Err(CitationNodeError::UnknownCitationAttr { field }),
        None => Ok(()),
    }
}

fn first_unknown_field(fields: &Map<String, Value>, allowed: &[&str]) -> Option<String> {
    fields
        .keys()
        .find(|field| !allowed.contains(&field.as_str()))
        .cloned()
}

fn take_schema_version(fields: &mut Map<String, Value>) -> Result<Value, CitationNodeError> {
    take_required_field(
        fields,
        SCHEMA_VERSION_FIELD,
        CitationNodeError::MissingSchemaVersion,
    )
}

fn take_citekey(fields: &mut Map<String, Value>) -> Result<Value, CitationNodeError> {
    take_required_field(fields, CITEKEY_FIELD, CitationNodeError::MissingCitekey)
}

fn take_render_style(fields: &mut Map<String, Value>) -> Result<Value, CitationNodeError> {
    take_required_field(
        fields,
        RENDER_STYLE_FIELD,
        CitationNodeError::MissingRenderStyle,
    )
}

fn take_required_field(
    fields: &mut Map<String, Value>,
    field: &str,
    error: CitationNodeError,
) -> Result<Value, CitationNodeError> {
    fields.remove(field).ok_or(error)
}

fn parse_schema_version(value: Value) -> Result<u64, CitationNodeError> {
    let version = value
        .as_u64()
        .ok_or(CitationNodeError::InvalidSchemaVersion)?;
    if version == CITATION_NODE_SCHEMA_VERSION {
        Ok(version)
    } else {
        Err(CitationNodeError::UnsupportedSchemaVersion { found: version })
    }
}

fn parse_citekey(value: Value) -> Result<String, CitationNodeError> {
    let Value::String(citekey) = value else {
        return Err(CitationNodeError::InvalidCitekey);
    };
    if is_valid_citekey(&citekey) {
        Ok(citekey)
    } else {
        Err(CitationNodeError::InvalidCitekey)
    }
}

fn parse_render_style(value: Value) -> Result<CitationRenderStyle, CitationNodeError> {
    match value.as_str() {
        Some("apa7") => Ok(CitationRenderStyle::Apa7),
        _ => Err(CitationNodeError::UnsupportedRenderStyle),
    }
}

#[cfg(test)]
#[path = "node_tests.rs"]
mod tests;
