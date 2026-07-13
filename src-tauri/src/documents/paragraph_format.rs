use serde::Serialize;
use serde_json::{Map, Value};

const TYPE_FIELD: &str = "type";
const ATTRS_FIELD: &str = "attrs";
const CONTENT_FIELD: &str = "content";
const PARAGRAPH_STYLE_FIELD: &str = "paragraphStyle";
const STYLE_FIELDS: [&str; 8] = [
    "schemaVersion",
    "alignment",
    "lineSpacingHundredths",
    "spaceBeforeTwips",
    "spaceAfterTwips",
    "leftIndentTwips",
    "rightIndentTwips",
    "specialIndent",
];
const SPECIAL_INDENT_FIELDS: [&str; 2] = ["kind", "twips"];

pub(crate) const PARAGRAPH_STYLE_SCHEMA_VERSION: u64 = 1;
pub(crate) const MIN_LINE_SPACING_HUNDREDTHS: u64 = 100;
pub(crate) const MAX_LINE_SPACING_HUNDREDTHS: u64 = 300;
pub(crate) const LINE_SPACING_INCREMENT: u64 = 5;
pub(crate) const MAX_PARAGRAPH_SPACING_TWIPS: u64 = 2_880;
pub(crate) const MAX_SPECIAL_INDENT_TWIPS: u64 = 1_440;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ParagraphAlignment {
    Left,
    Center,
    Right,
    Justify,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SpecialIndentKind {
    None,
    FirstLine,
    Hanging,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SpecialIndent {
    kind: SpecialIndentKind,
    twips: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ParagraphStyle {
    alignment: ParagraphAlignment,
    line_spacing_hundredths: u16,
    space_before_twips: u16,
    space_after_twips: u16,
    left_indent_twips: u16,
    right_indent_twips: u16,
    special_indent: SpecialIndent,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum ParagraphStyleError {
    UnsupportedBlock,
    InvalidStyleObject,
    MissingStyleField { field: String },
    UnknownStyleField { field: String },
    InvalidStyleSchemaVersion,
    UnsupportedStyleSchemaVersion { found: u64 },
    InvalidAlignment,
    InvalidLineSpacing,
    InvalidParagraphSpacing,
    InvalidParagraphIndent,
    InvalidSpecialIndentObject,
    MissingSpecialIndentField { field: String },
    UnknownSpecialIndentField { field: String },
    InvalidSpecialIndentKind,
    InvalidSpecialIndentAmount,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct LocatedParagraphStyleError {
    pub(crate) path: String,
    pub(crate) cause: ParagraphStyleError,
}

impl ParagraphAlignment {
    pub(crate) fn docx_value(self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Center => "center",
            Self::Right => "right",
            Self::Justify => "both",
        }
    }
}

impl ParagraphStyle {
    pub(crate) fn alignment(self) -> ParagraphAlignment {
        self.alignment
    }

    pub(crate) fn line_spacing_docx_units(self) -> u16 {
        self.line_spacing_hundredths * 240 / 100
    }

    pub(crate) fn space_before_twips(self) -> u16 {
        self.space_before_twips
    }

    pub(crate) fn space_after_twips(self) -> u16 {
        self.space_after_twips
    }

    pub(crate) fn left_indent_twips(self) -> u16 {
        self.left_indent_twips
    }

    pub(crate) fn right_indent_twips(self) -> u16 {
        self.right_indent_twips
    }

    pub(crate) fn special_indent(self) -> SpecialIndent {
        self.special_indent
    }
}

impl SpecialIndent {
    pub(crate) fn kind(self) -> SpecialIndentKind {
        self.kind
    }

    pub(crate) fn twips(self) -> u16 {
        self.twips
    }
}

pub(crate) fn validate_document_paragraph_styles(
    document: &Value,
) -> Result<(), LocatedParagraphStyleError> {
    validate_node(document, "document")
}

pub(crate) fn parse_paragraph_style_attrs(
    attrs: Option<&Value>,
) -> Result<Option<ParagraphStyle>, ParagraphStyleError> {
    let Some(attrs) = attrs else {
        return Ok(None);
    };
    let fields = attrs
        .as_object()
        .ok_or(ParagraphStyleError::InvalidStyleObject)?;
    match fields.get(PARAGRAPH_STYLE_FIELD) {
        Some(value) => parse_paragraph_style(value).map(Some),
        None => Ok(None),
    }
}

fn validate_node(value: &Value, path: &str) -> Result<(), LocatedParagraphStyleError> {
    let Some(node) = value.as_object() else {
        return Ok(());
    };
    validate_node_style(node, path)?;
    validate_children(node.get(CONTENT_FIELD), path)
}

fn validate_node_style(
    node: &Map<String, Value>,
    path: &str,
) -> Result<(), LocatedParagraphStyleError> {
    let Some(attrs) = node.get(ATTRS_FIELD).and_then(Value::as_object) else {
        return Ok(());
    };
    let Some(style) = attrs.get(PARAGRAPH_STYLE_FIELD) else {
        return Ok(());
    };
    require_supported_block(node)
        .and_then(|()| parse_paragraph_style(style).map(drop))
        .map_err(|cause| LocatedParagraphStyleError {
            path: format!("{path}.attrs.{PARAGRAPH_STYLE_FIELD}"),
            cause,
        })
}

fn require_supported_block(node: &Map<String, Value>) -> Result<(), ParagraphStyleError> {
    match node.get(TYPE_FIELD).and_then(Value::as_str) {
        Some("paragraph" | "heading") => Ok(()),
        _ => Err(ParagraphStyleError::UnsupportedBlock),
    }
}

fn validate_children(value: Option<&Value>, path: &str) -> Result<(), LocatedParagraphStyleError> {
    let Some(Value::Array(children)) = value else {
        return Ok(());
    };
    for (index, child) in children.iter().enumerate() {
        validate_node(child, &format!("{path}.content[{index}]"))?;
    }
    Ok(())
}

fn parse_paragraph_style(value: &Value) -> Result<ParagraphStyle, ParagraphStyleError> {
    let fields = value
        .as_object()
        .ok_or(ParagraphStyleError::InvalidStyleObject)?;
    require_exact_fields(fields, &STYLE_FIELDS, style_field_error)?;
    parse_style_schema_version(fields)?;
    Ok(ParagraphStyle {
        alignment: parse_alignment(fields)?,
        line_spacing_hundredths: parse_line_spacing(fields)?,
        space_before_twips: parse_paragraph_twips(fields, "spaceBeforeTwips")?,
        space_after_twips: parse_paragraph_twips(fields, "spaceAfterTwips")?,
        left_indent_twips: parse_indent_twips(fields, "leftIndentTwips")?,
        right_indent_twips: parse_indent_twips(fields, "rightIndentTwips")?,
        special_indent: parse_special_indent(fields)?,
    })
}

fn parse_style_schema_version(fields: &Map<String, Value>) -> Result<(), ParagraphStyleError> {
    let version = fields
        .get("schemaVersion")
        .and_then(Value::as_u64)
        .ok_or(ParagraphStyleError::InvalidStyleSchemaVersion)?;
    if version == PARAGRAPH_STYLE_SCHEMA_VERSION {
        Ok(())
    } else {
        Err(ParagraphStyleError::UnsupportedStyleSchemaVersion { found: version })
    }
}

fn style_field_error(field: &str, missing: bool) -> ParagraphStyleError {
    if missing {
        ParagraphStyleError::MissingStyleField {
            field: field.to_owned(),
        }
    } else {
        ParagraphStyleError::UnknownStyleField {
            field: field.to_owned(),
        }
    }
}

fn parse_alignment(fields: &Map<String, Value>) -> Result<ParagraphAlignment, ParagraphStyleError> {
    let alignment = fields
        .get("alignment")
        .and_then(Value::as_str)
        .ok_or(ParagraphStyleError::InvalidAlignment)?;
    match alignment {
        "left" => Ok(ParagraphAlignment::Left),
        "center" => Ok(ParagraphAlignment::Center),
        "right" => Ok(ParagraphAlignment::Right),
        "justify" => Ok(ParagraphAlignment::Justify),
        _ => Err(ParagraphStyleError::InvalidAlignment),
    }
}

fn parse_line_spacing(fields: &Map<String, Value>) -> Result<u16, ParagraphStyleError> {
    let value = required_u64(fields, "lineSpacingHundredths")
        .map_err(|_| ParagraphStyleError::InvalidLineSpacing)?;
    let valid_range = (MIN_LINE_SPACING_HUNDREDTHS..=MAX_LINE_SPACING_HUNDREDTHS).contains(&value);
    if valid_range && value.is_multiple_of(LINE_SPACING_INCREMENT) {
        Ok(value as u16)
    } else {
        Err(ParagraphStyleError::InvalidLineSpacing)
    }
}

fn parse_paragraph_twips(
    fields: &Map<String, Value>,
    field: &str,
) -> Result<u16, ParagraphStyleError> {
    parse_bounded_u16(fields, field, MAX_PARAGRAPH_SPACING_TWIPS)
        .map_err(|_| ParagraphStyleError::InvalidParagraphSpacing)
}

fn parse_indent_twips(
    fields: &Map<String, Value>,
    field: &str,
) -> Result<u16, ParagraphStyleError> {
    parse_bounded_u16(fields, field, MAX_PARAGRAPH_SPACING_TWIPS)
        .map_err(|_| ParagraphStyleError::InvalidParagraphIndent)
}

fn parse_special_indent(fields: &Map<String, Value>) -> Result<SpecialIndent, ParagraphStyleError> {
    let value = fields
        .get("specialIndent")
        .ok_or_else(|| style_field_error("specialIndent", true))?;
    let indent = value
        .as_object()
        .ok_or(ParagraphStyleError::InvalidSpecialIndentObject)?;
    require_exact_fields(indent, &SPECIAL_INDENT_FIELDS, special_indent_field_error)?;
    let kind = parse_special_indent_kind(indent)?;
    let twips = parse_special_indent_twips(indent)?;
    if kind == SpecialIndentKind::None && twips != 0 {
        return Err(ParagraphStyleError::InvalidSpecialIndentAmount);
    }
    Ok(SpecialIndent { kind, twips })
}

fn special_indent_field_error(field: &str, missing: bool) -> ParagraphStyleError {
    if missing {
        ParagraphStyleError::MissingSpecialIndentField {
            field: field.to_owned(),
        }
    } else {
        ParagraphStyleError::UnknownSpecialIndentField {
            field: field.to_owned(),
        }
    }
}

fn parse_special_indent_kind(
    fields: &Map<String, Value>,
) -> Result<SpecialIndentKind, ParagraphStyleError> {
    let kind = fields
        .get("kind")
        .and_then(Value::as_str)
        .ok_or(ParagraphStyleError::InvalidSpecialIndentKind)?;
    match kind {
        "none" => Ok(SpecialIndentKind::None),
        "first_line" => Ok(SpecialIndentKind::FirstLine),
        "hanging" => Ok(SpecialIndentKind::Hanging),
        _ => Err(ParagraphStyleError::InvalidSpecialIndentKind),
    }
}

fn parse_special_indent_twips(fields: &Map<String, Value>) -> Result<u16, ParagraphStyleError> {
    parse_bounded_u16(fields, "twips", MAX_SPECIAL_INDENT_TWIPS)
        .map_err(|_| ParagraphStyleError::InvalidSpecialIndentAmount)
}

fn parse_bounded_u16(fields: &Map<String, Value>, field: &str, maximum: u64) -> Result<u16, ()> {
    let value = required_u64(fields, field).map_err(|_| ())?;
    u16::try_from(value)
        .ok()
        .filter(|value| u64::from(*value) <= maximum)
        .ok_or(())
}

fn required_u64(fields: &Map<String, Value>, field: &str) -> Result<u64, ParagraphStyleError> {
    fields
        .get(field)
        .and_then(Value::as_u64)
        .ok_or_else(|| style_field_error(field, true))
}

fn require_exact_fields(
    fields: &Map<String, Value>,
    expected: &[&str],
    error: fn(&str, bool) -> ParagraphStyleError,
) -> Result<(), ParagraphStyleError> {
    if let Some(field) = expected.iter().find(|field| !fields.contains_key(**field)) {
        return Err(error(field, true));
    }
    match fields
        .keys()
        .find(|field| !expected.contains(&field.as_str()))
    {
        Some(field) => Err(error(field, false)),
        None => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn complete_supported_style_validates_and_maps() {
        let style = parse_paragraph_style(&valid_style()).unwrap();

        assert_eq!(style.alignment().docx_value(), "left");
        assert_eq!(style.line_spacing_docx_units(), 240);
        assert_eq!(style.space_before_twips(), 0);
        assert_eq!(style.special_indent().kind(), SpecialIndentKind::None);
    }

    #[test]
    fn every_alignment_and_special_indent_kind_validates() {
        for alignment in ["left", "center", "right", "justify"] {
            let mut style = valid_style();
            style["alignment"] = json!(alignment);
            assert!(parse_paragraph_style(&style).is_ok());
        }
        for (kind, twips) in [("none", 0), ("first_line", 720), ("hanging", 1_440)] {
            let mut style = valid_style();
            style["specialIndent"] = json!({ "kind": kind, "twips": twips });
            assert!(parse_paragraph_style(&style).is_ok());
        }
    }

    #[test]
    fn numeric_boundaries_validate_without_clamping() {
        for line in [100, 115, 300] {
            let mut style = valid_style();
            style["lineSpacingHundredths"] = json!(line);
            assert!(parse_paragraph_style(&style).is_ok());
        }
        for twips in [0, 2_880] {
            let mut style = valid_style();
            style["spaceBeforeTwips"] = json!(twips);
            style["rightIndentTwips"] = json!(twips);
            assert!(parse_paragraph_style(&style).is_ok());
        }
    }

    #[test]
    fn malformed_missing_unknown_and_out_of_range_values_fail() {
        let cases = [
            json!(null),
            style_without("alignment"),
            style_with("css", json!("evil")),
            style_with("schemaVersion", json!(2)),
            style_with("alignment", json!("distributed")),
            style_with("lineSpacingHundredths", json!(101)),
            style_with("lineSpacingHundredths", json!(300.5)),
            style_with("spaceAfterTwips", json!(-1)),
            style_with("leftIndentTwips", json!(2_881)),
            style_with("specialIndent", json!({ "kind": "none", "twips": 1 })),
            style_with(
                "specialIndent",
                json!({ "kind": "first_line", "twips": 1_441 }),
            ),
            style_with("specialIndent", json!({ "kind": "both", "twips": 0 })),
        ];

        for value in cases {
            assert!(parse_paragraph_style(&value).is_err(), "accepted {value}");
        }
    }

    #[test]
    fn every_required_field_and_nested_field_is_enforced() {
        for field in STYLE_FIELDS {
            assert_eq!(
                parse_paragraph_style(&style_without(field)),
                Err(ParagraphStyleError::MissingStyleField {
                    field: field.to_owned()
                })
            );
        }

        for field in SPECIAL_INDENT_FIELDS {
            let mut special = json!({ "kind": "none", "twips": 0 });
            special.as_object_mut().unwrap().remove(field);
            assert!(matches!(
                parse_paragraph_style(&style_with("specialIndent", special)),
                Err(ParagraphStyleError::MissingSpecialIndentField { .. })
            ));
        }
    }

    #[test]
    fn malformed_storage_types_and_nested_unknown_fields_fail() {
        let malformed = [
            style_with("schemaVersion", json!(1.5)),
            style_with("alignment", json!(1)),
            style_with("lineSpacingHundredths", json!("100")),
            style_with("spaceBeforeTwips", json!(1.5)),
            style_with("spaceAfterTwips", json!(null)),
            style_with("leftIndentTwips", json!([])),
            style_with("rightIndentTwips", json!({})),
            style_with("specialIndent", json!([])),
            style_with("specialIndent", json!({ "kind": 1, "twips": 0 })),
            style_with("specialIndent", json!({ "kind": "none", "twips": "0" })),
            style_with(
                "specialIndent",
                json!({ "kind": "none", "twips": 0, "left": 1 }),
            ),
        ];

        for value in malformed {
            assert!(parse_paragraph_style(&value).is_err(), "accepted {value}");
        }
    }

    #[test]
    fn paragraph_failure_shape_is_stable_and_content_free() {
        let errors = [
            ParagraphStyleError::UnknownStyleField {
                field: "css".to_owned(),
            },
            ParagraphStyleError::UnsupportedStyleSchemaVersion { found: 2 },
            ParagraphStyleError::InvalidLineSpacing,
            ParagraphStyleError::InvalidSpecialIndentAmount,
        ];

        assert_eq!(
            serde_json::to_value(errors).unwrap(),
            json!([
                { "code": "unknown_style_field", "field": "css" },
                { "code": "unsupported_style_schema_version", "found": 2 },
                { "code": "invalid_line_spacing" },
                { "code": "invalid_special_indent_amount" }
            ])
        );
    }

    #[test]
    fn style_is_limited_to_paragraphs_and_headings() {
        let valid = document_with("paragraph", valid_style());
        let invalid = document_with("listItem", valid_style());

        assert_eq!(validate_document_paragraph_styles(&valid), Ok(()));
        assert!(matches!(
            validate_document_paragraph_styles(&invalid),
            Err(LocatedParagraphStyleError {
                cause: ParagraphStyleError::UnsupportedBlock,
                ..
            })
        ));
    }

    fn valid_style() -> Value {
        json!({
            "schemaVersion": PARAGRAPH_STYLE_SCHEMA_VERSION,
            "alignment": "left",
            "lineSpacingHundredths": 100,
            "spaceBeforeTwips": 0,
            "spaceAfterTwips": 0,
            "leftIndentTwips": 0,
            "rightIndentTwips": 0,
            "specialIndent": { "kind": "none", "twips": 0 }
        })
    }

    fn style_without(field: &str) -> Value {
        let mut style = valid_style();
        style.as_object_mut().unwrap().remove(field);
        style
    }

    fn style_with(field: &str, value: Value) -> Value {
        let mut style = valid_style();
        style[field] = value;
        style
    }

    fn document_with(node_type: &str, style: Value) -> Value {
        json!({
            "type": "doc",
            "content": [{
                "type": node_type,
                "attrs": { "paragraphStyle": style }
            }]
        })
    }
}
