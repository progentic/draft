use serde::Serialize;
use serde_json::{Map, Value};

const TYPE_FIELD: &str = "type";
const ATTRS_FIELD: &str = "attrs";
const CONTENT_FIELD: &str = "content";
const MARKS_FIELD: &str = "marks";
const FAMILY_FIELD: &str = "family";
const POINTS_FIELD: &str = "points";
const FONT_FAMILY_MARK: &str = "fontFamily";
const FONT_SIZE_MARK: &str = "fontSize";
const FONT_MARK_FIELDS: [&str; 2] = [TYPE_FIELD, ATTRS_FIELD];

pub(crate) const MIN_FONT_SIZE_POINTS: u64 = 8;
pub(crate) const MAX_FONT_SIZE_POINTS: u64 = 72;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FontFamily {
    Arial,
    AvenirNext,
    Baskerville,
    CourierNew,
    Georgia,
    Helvetica,
    Menlo,
    Palatino,
    TimesNewRoman,
    TrebuchetMs,
    Verdana,
}

#[derive(Clone, Copy)]
struct FontDefinition {
    identifier: &'static str,
    family: FontFamily,
    docx_name: &'static str,
}

const FONT_DEFINITIONS: [FontDefinition; 11] = [
    font("arial", FontFamily::Arial, "Arial"),
    font("avenir_next", FontFamily::AvenirNext, "Avenir Next"),
    font("baskerville", FontFamily::Baskerville, "Baskerville"),
    font("courier_new", FontFamily::CourierNew, "Courier New"),
    font("georgia", FontFamily::Georgia, "Georgia"),
    font("helvetica", FontFamily::Helvetica, "Helvetica"),
    font("menlo", FontFamily::Menlo, "Menlo"),
    font("palatino", FontFamily::Palatino, "Palatino"),
    font(
        "times_new_roman",
        FontFamily::TimesNewRoman,
        "Times New Roman",
    ),
    font("trebuchet_ms", FontFamily::TrebuchetMs, "Trebuchet MS"),
    font("verdana", FontFamily::Verdana, "Verdana"),
];

const fn font(
    identifier: &'static str,
    family: FontFamily,
    docx_name: &'static str,
) -> FontDefinition {
    FontDefinition {
        identifier,
        family,
        docx_name,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FontSizePoints(u8);

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum TextFormatError {
    InvalidMarkObject,
    MissingMarkType,
    InvalidMarkType,
    UnknownFontMarkField { field: String },
    MissingAttrs,
    InvalidAttrsObject,
    UnknownFontAttr { field: String },
    MissingFontFamily,
    UnsupportedFontFamily,
    MissingFontSize,
    InvalidFontSize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct LocatedTextFormatError {
    pub(crate) path: String,
    pub(crate) cause: TextFormatError,
}

impl FontFamily {
    pub(crate) fn from_identifier(identifier: &str) -> Option<Self> {
        FONT_DEFINITIONS
            .iter()
            .find(|definition| definition.identifier == identifier)
            .map(|definition| definition.family)
    }

    pub(crate) fn docx_name(self) -> &'static str {
        FONT_DEFINITIONS
            .iter()
            .find(|definition| definition.family == self)
            .map(|definition| definition.docx_name)
            .expect("every font family must have one definition")
    }
}

impl FontSizePoints {
    pub(crate) fn from_u64(points: u64) -> Option<Self> {
        if (MIN_FONT_SIZE_POINTS..=MAX_FONT_SIZE_POINTS).contains(&points) {
            Some(Self(points as u8))
        } else {
            None
        }
    }

    pub(crate) fn half_points(self) -> u16 {
        u16::from(self.0) * 2
    }
}

pub(crate) fn validate_document_text_formats(
    document: &Value,
) -> Result<(), LocatedTextFormatError> {
    validate_node(document, "document")
}

pub(crate) fn parse_font_family_attrs(value: &Value) -> Result<FontFamily, TextFormatError> {
    let fields = attrs_fields(value)?;
    require_exact_attrs(fields, FAMILY_FIELD, TextFormatError::MissingFontFamily)?;
    let identifier = fields
        .get(FAMILY_FIELD)
        .and_then(Value::as_str)
        .ok_or(TextFormatError::MissingFontFamily)?;
    FontFamily::from_identifier(identifier).ok_or(TextFormatError::UnsupportedFontFamily)
}

pub(crate) fn parse_font_size_attrs(value: &Value) -> Result<FontSizePoints, TextFormatError> {
    let fields = attrs_fields(value)?;
    require_exact_attrs(fields, POINTS_FIELD, TextFormatError::MissingFontSize)?;
    let points = fields
        .get(POINTS_FIELD)
        .and_then(Value::as_u64)
        .ok_or(TextFormatError::InvalidFontSize)?;
    FontSizePoints::from_u64(points).ok_or(TextFormatError::InvalidFontSize)
}

fn validate_node(value: &Value, path: &str) -> Result<(), LocatedTextFormatError> {
    let Some(node) = value.as_object() else {
        return Ok(());
    };
    validate_marks(node.get(MARKS_FIELD), path)?;
    validate_children(node.get(CONTENT_FIELD), path)
}

fn validate_marks(value: Option<&Value>, path: &str) -> Result<(), LocatedTextFormatError> {
    let Some(Value::Array(marks)) = value else {
        return Ok(());
    };
    for (index, mark) in marks.iter().enumerate() {
        validate_mark(mark).map_err(|cause| LocatedTextFormatError {
            path: format!("{path}.marks[{index}]"),
            cause,
        })?;
    }
    Ok(())
}

fn validate_mark(value: &Value) -> Result<(), TextFormatError> {
    let fields = value
        .as_object()
        .ok_or(TextFormatError::InvalidMarkObject)?;
    match required_mark_type(fields)? {
        FONT_FAMILY_MARK => validate_font_mark(fields, parse_font_family_attrs),
        FONT_SIZE_MARK => validate_font_mark(fields, parse_font_size_attrs),
        _ => Ok(()),
    }
}

fn required_mark_type(fields: &Map<String, Value>) -> Result<&str, TextFormatError> {
    fields
        .get(TYPE_FIELD)
        .ok_or(TextFormatError::MissingMarkType)?
        .as_str()
        .ok_or(TextFormatError::InvalidMarkType)
}

fn validate_font_mark<T>(
    fields: &Map<String, Value>,
    parse_attrs: fn(&Value) -> Result<T, TextFormatError>,
) -> Result<(), TextFormatError> {
    require_exact_mark_fields(fields)?;
    let attrs = fields
        .get(ATTRS_FIELD)
        .ok_or(TextFormatError::MissingAttrs)?;
    parse_attrs(attrs).map(drop)
}

fn validate_children(value: Option<&Value>, path: &str) -> Result<(), LocatedTextFormatError> {
    let Some(Value::Array(children)) = value else {
        return Ok(());
    };
    for (index, child) in children.iter().enumerate() {
        validate_node(child, &format!("{path}.content[{index}]"))?;
    }
    Ok(())
}

fn attrs_fields(value: &Value) -> Result<&Map<String, Value>, TextFormatError> {
    value.as_object().ok_or(TextFormatError::InvalidAttrsObject)
}

fn require_exact_mark_fields(fields: &Map<String, Value>) -> Result<(), TextFormatError> {
    match first_unknown_field(fields, &FONT_MARK_FIELDS) {
        Some(field) => Err(TextFormatError::UnknownFontMarkField { field }),
        None => Ok(()),
    }
}

fn require_exact_attrs(
    fields: &Map<String, Value>,
    required: &str,
    missing: TextFormatError,
) -> Result<(), TextFormatError> {
    if !fields.contains_key(required) {
        return Err(missing);
    }
    if fields.len() == 1 {
        return Ok(());
    }
    let field = fields
        .keys()
        .find(|field| field.as_str() != required)
        .cloned()
        .expect("an additional attribute must exist");
    Err(TextFormatError::UnknownFontAttr { field })
}

fn first_unknown_field(fields: &Map<String, Value>, allowed: &[&str]) -> Option<String> {
    fields
        .keys()
        .find(|field| !allowed.contains(&field.as_str()))
        .cloned()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn supported_family_and_size_marks_validate() {
        let document = formatted_document(json!([
            { "type": "fontFamily", "attrs": { "family": "georgia" } },
            { "type": "fontSize", "attrs": { "points": 17 } }
        ]));

        assert_eq!(validate_document_text_formats(&document), Ok(()));
        assert_eq!(FontFamily::Georgia.docx_name(), "Georgia");
        assert_eq!(FontSizePoints::from_u64(17).unwrap().half_points(), 34);
    }

    #[test]
    fn canonical_font_allowlist_maps_every_identifier_to_docx() {
        let actual = FONT_DEFINITIONS
            .iter()
            .map(|definition| {
                let family = FontFamily::from_identifier(definition.identifier).unwrap();
                (definition.identifier, family.docx_name())
            })
            .collect::<Vec<_>>();

        assert_eq!(actual, expected_font_definitions());
    }

    #[test]
    fn unsupported_font_values_fail_with_bounded_paths() {
        let family = formatted_document(json!([
            { "type": "fontFamily", "attrs": { "family": "url(evil)" } }
        ]));
        let size = formatted_document(json!([
            { "type": "fontSize", "attrs": { "points": 73 } }
        ]));

        assert_eq!(
            validate_document_text_formats(&family),
            Err(LocatedTextFormatError {
                path: "document.content[0].content[0].marks[0]".to_owned(),
                cause: TextFormatError::UnsupportedFontFamily,
            })
        );
        assert_eq!(
            validate_document_text_formats(&size),
            Err(LocatedTextFormatError {
                path: "document.content[0].content[0].marks[0]".to_owned(),
                cause: TextFormatError::InvalidFontSize,
            })
        );
    }

    #[test]
    fn fractional_zero_negative_and_malformed_sizes_fail() {
        for points in [
            json!(0),
            json!(-1),
            json!(8.5),
            json!(73),
            json!("12"),
            json!(null),
        ] {
            let document = formatted_document(json!([
                { "type": "fontSize", "attrs": { "points": points } }
            ]));
            assert!(matches!(
                validate_document_text_formats(&document),
                Err(LocatedTextFormatError {
                    cause: TextFormatError::InvalidFontSize,
                    ..
                })
            ));
        }
    }

    #[test]
    fn malformed_mark_shapes_fail_before_persistence() {
        for mark in [json!(null), json!("bold"), json!(1), json!([])] {
            let document = formatted_document(json!([mark]));
            assert!(matches!(
                validate_document_text_formats(&document),
                Err(LocatedTextFormatError {
                    cause: TextFormatError::InvalidMarkObject,
                    ..
                })
            ));
        }

        for mark in [json!({}), json!({ "attrs": {} })] {
            let document = formatted_document(json!([mark]));
            assert!(matches!(
                validate_document_text_formats(&document),
                Err(LocatedTextFormatError {
                    cause: TextFormatError::MissingMarkType,
                    ..
                })
            ));
        }

        let invalid_type = formatted_document(json!([{ "type": 1 }]));
        assert!(matches!(
            validate_document_text_formats(&invalid_type),
            Err(LocatedTextFormatError {
                cause: TextFormatError::InvalidMarkType,
                ..
            })
        ));
    }

    #[test]
    fn strict_font_mark_fields_and_attrs_fail_closed() {
        let malformed = [
            json!({ "type": "fontFamily", "attrs": null }),
            json!({ "type": "fontFamily", "attrs": { "family": "arial", "css": "serif" } }),
            json!({ "type": "fontFamily", "attrs": { "family": "arial" }, "style": "serif" }),
            json!({ "type": "fontSize", "attrs": [] }),
            json!({ "type": "fontSize", "attrs": { "points": 12, "unit": "pt" } }),
            json!({ "type": "fontSize", "attrs": { "points": 12 }, "style": "12px" }),
        ];

        for mark in malformed {
            assert!(validate_document_text_formats(&formatted_document(json!([mark]))).is_err());
        }
    }

    fn formatted_document(marks: Value) -> Value {
        json!({
            "type": "doc",
            "content": [{
                "type": "paragraph",
                "content": [{ "type": "text", "text": "Text", "marks": marks }]
            }]
        })
    }

    fn expected_font_definitions() -> Vec<(&'static str, &'static str)> {
        vec![
            ("arial", "Arial"),
            ("avenir_next", "Avenir Next"),
            ("baskerville", "Baskerville"),
            ("courier_new", "Courier New"),
            ("georgia", "Georgia"),
            ("helvetica", "Helvetica"),
            ("menlo", "Menlo"),
            ("palatino", "Palatino"),
            ("times_new_roman", "Times New Roman"),
            ("trebuchet_ms", "Trebuchet MS"),
            ("verdana", "Verdana"),
        ]
    }
}
