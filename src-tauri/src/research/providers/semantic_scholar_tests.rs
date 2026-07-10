use serde_json::json;

use super::*;

#[test]
fn semantic_scholar_request_uses_doi_identifier_and_bounded_fields() {
    let doi = Doi::parse("10.1000/test").unwrap();
    let url = semantic_scholar_request_url(&doi).unwrap();

    assert_eq!(url.scheme(), "https");
    assert_eq!(url.path(), "/graph/v1/paper/DOI:10.1000%2Ftest");
    assert_eq!(
        url.query_pairs().collect::<Vec<_>>(),
        [("fields".into(), SEMANTIC_SCHOLAR_FIELDS.into())]
    );
}

#[test]
fn semantic_scholar_response_normalizes_candidate_metadata() {
    let requested = Doi::parse("10.1000/test").unwrap();
    let body = json!({
        "paperId": "paper-123",
        "title": "Semantic result",
        "authors": [{ "name": "Ada Lovelace" }],
        "year": 2024,
        "venue": "Semantic Journal",
        "externalIds": { "DOI": "10.1000/TEST" },
        "openAccessPdf": { "url": "https://example.org/paper.pdf", "status": "GREEN" }
    });

    let record =
        parse_semantic_scholar_response(&body.to_string().into_bytes(), &requested).unwrap();

    assert_eq!(record.provider(), MetadataProvider::SemanticScholar);
    assert_eq!(record.provider_record_id(), "paper-123");
    assert_eq!(record.doi().as_str(), "10.1000/test");
    assert_eq!(record.title(), "Semantic result");
    assert_eq!(record.authors(), ["Ada Lovelace"]);
    assert_eq!(record.year(), Some(2024));
    assert_eq!(record.venue(), Some("Semantic Journal"));
    assert_eq!(
        record.open_access_url(),
        Some("https://example.org/paper.pdf")
    );
}

#[test]
fn semantic_scholar_response_rejects_malformed_or_mismatched_data() {
    let requested = Doi::parse("10.1000/test").unwrap();
    for body in [
        json!({ "error": "not found" }),
        json!({
            "paperId": "paper", "title": "Title", "authors": [], "year": null,
            "venue": null, "externalIds": { "DOI": "10.1000/other" },
            "openAccessPdf": null
        }),
        json!({
            "paperId": "paper", "title": " ", "authors": [], "year": null,
            "venue": null, "externalIds": null, "openAccessPdf": null
        }),
    ] {
        assert_eq!(
            parse_semantic_scholar_response(&body.to_string().into_bytes(), &requested),
            Err(MetadataLookupError::InvalidResponse)
        );
    }
}
