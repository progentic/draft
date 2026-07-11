use serde_json::{Value, json};

use super::*;

#[test]
fn valid_citation_attrs_deserialize() {
    let attrs = parse_attrs(valid_attrs());

    assert_eq!(attrs.schema_version(), CITATION_NODE_SCHEMA_VERSION);
    assert_eq!(attrs.citekey(), "smith2025");
    assert_eq!(attrs.render_style(), CitationRenderStyle::Apa7);
}

#[test]
fn citation_attrs_serialization_is_stable() {
    let attrs = parse_attrs(valid_attrs());

    assert_eq!(serde_json::to_value(attrs).unwrap(), valid_attrs());
}

#[test]
fn citation_attrs_round_trip_is_stable() {
    let expected = parse_attrs(valid_attrs());
    let serialized = serde_json::to_value(&expected).unwrap();
    let actual = serde_json::from_value::<CitationNodeAttributes>(serialized).unwrap();

    assert_eq!(actual, expected);
}

#[test]
fn non_object_citation_attrs_fail() {
    assert_eq!(
        CitationNodeAttributes::from_json_value(json!([])),
        Err(CitationNodeError::InvalidCitationAttrsObject),
    );
}

#[test]
fn unknown_citation_fields_fail() {
    let mut attrs = valid_attrs();
    attrs["reference"] = json!({ "title": "Embedded metadata" });

    assert_eq!(
        CitationNodeAttributes::from_json_value(attrs),
        Err(CitationNodeError::UnknownCitationAttr {
            field: "reference".to_owned(),
        }),
    );
}

#[test]
fn missing_citation_fields_fail_predictably() {
    assert_missing_attr(
        SCHEMA_VERSION_FIELD,
        CitationNodeError::MissingSchemaVersion,
    );
    assert_missing_attr(CITEKEY_FIELD, CitationNodeError::MissingCitekey);
    assert_missing_attr(RENDER_STYLE_FIELD, CitationNodeError::MissingRenderStyle);
}

#[test]
fn malformed_and_unsupported_citation_versions_fail() {
    for malformed in [json!("1"), json!(1.5), json!(-1), json!(true)] {
        assert_attr_error(
            SCHEMA_VERSION_FIELD,
            malformed,
            CitationNodeError::InvalidSchemaVersion,
        );
    }
    for version in [0, 2] {
        assert_attr_error(
            SCHEMA_VERSION_FIELD,
            json!(version),
            CitationNodeError::UnsupportedSchemaVersion { found: version },
        );
    }
}

#[test]
fn malformed_citation_citekeys_fail() {
    for citekey in [json!(""), json!("smith.2025"), json!("é2025"), json!(7)] {
        assert_attr_error(CITEKEY_FIELD, citekey, CitationNodeError::InvalidCitekey);
    }
}

#[test]
fn unsupported_render_styles_fail() {
    for style in [json!("apa6"), json!(""), json!(7)] {
        assert_attr_error(
            RENDER_STYLE_FIELD,
            style,
            CitationNodeError::UnsupportedRenderStyle,
        );
    }
}

#[test]
fn nested_document_citations_validate() {
    validate_document_citations(&document_with(citation_node())).unwrap();
}

#[test]
fn document_citations_are_collected_in_order() {
    let mut second_attrs = valid_attrs();
    second_attrs[CITEKEY_FIELD] = json!("jones2024");
    let document = json!({
        "type": "doc",
        "content": [
            citation_node(),
            { "type": "citation", "attrs": second_attrs }
        ]
    });

    let citekeys = document_citations(&document)
        .unwrap()
        .into_iter()
        .map(|citation| citation.citekey().to_owned())
        .collect::<Vec<_>>();

    assert_eq!(citekeys, ["smith2025", "jones2024"]);
}

#[test]
fn invalid_nested_citation_reports_path_and_cause() {
    let mut node = citation_node();
    node["content"] = json!([]);

    assert_eq!(
        validate_document_citations(&document_with(node)),
        Err(LocatedCitationError {
            path: "document.content[0].content[0]".to_owned(),
            cause: CitationNodeError::UnknownCitationNodeField {
                field: "content".to_owned(),
            },
        }),
    );
}

#[test]
fn unrelated_tiptap_nodes_remain_opaque() {
    let document = json!({
        "type": "doc",
        "content": [{
            "type": "custom_node",
            "attrs": { "future": { "data": true } },
            "content": [{ "type": "text", "text": "Preserved" }]
        }]
    });

    validate_document_citations(&document).unwrap();
}

#[test]
fn citation_failure_shape_is_stable() {
    let errors = [
        CitationNodeError::InvalidCitationAttrsObject,
        CitationNodeError::UnknownCitationNodeField {
            field: "content".to_owned(),
        },
        CitationNodeError::UnknownCitationAttr {
            field: "reference".to_owned(),
        },
        CitationNodeError::UnsupportedSchemaVersion { found: 2 },
        CitationNodeError::InvalidCitekey,
        CitationNodeError::UnsupportedRenderStyle,
    ];

    assert_eq!(
        serde_json::to_value(errors).unwrap(),
        json!([
            { "code": "invalid_citation_attrs_object" },
            { "code": "unknown_citation_node_field", "field": "content" },
            { "code": "unknown_citation_attr", "field": "reference" },
            { "code": "unsupported_schema_version", "found": 2 },
            { "code": "invalid_citekey" },
            { "code": "unsupported_render_style" }
        ]),
    );
}

fn parse_attrs(value: Value) -> CitationNodeAttributes {
    CitationNodeAttributes::from_json_value(value).unwrap()
}

fn assert_missing_attr(field: &str, expected: CitationNodeError) {
    let mut attrs = valid_attrs();
    attrs.as_object_mut().unwrap().remove(field);
    assert_eq!(
        CitationNodeAttributes::from_json_value(attrs),
        Err(expected)
    );
}

fn assert_attr_error(field: &str, value: Value, expected: CitationNodeError) {
    let mut attrs = valid_attrs();
    attrs[field] = value;
    assert_eq!(
        CitationNodeAttributes::from_json_value(attrs),
        Err(expected)
    );
}

fn valid_attrs() -> Value {
    json!({
        "schema_version": CITATION_NODE_SCHEMA_VERSION,
        "citekey": "smith2025",
        "render_style": "apa7"
    })
}

fn citation_node() -> Value {
    json!({ "type": "citation", "attrs": valid_attrs() })
}

fn document_with(node: Value) -> Value {
    json!({
        "type": "doc",
        "content": [{
            "type": "paragraph",
            "content": [node]
        }]
    })
}
