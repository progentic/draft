use serde_json::json;

use super::*;

#[test]
fn crossref_request_uses_doi_and_polite_contact() {
    let doi = Doi::parse("10.1000/test").unwrap();
    let email = ContactEmail::parse("research@example.org").unwrap();
    let url = crossref_request_url(&doi, &email).unwrap();

    assert_eq!(url.scheme(), "https");
    assert_eq!(url.path(), "/v1/works/10.1000%2Ftest");
    assert_eq!(
        url.query_pairs().collect::<Vec<_>>(),
        [("mailto".into(), "research@example.org".into())]
    );
}

#[test]
fn crossref_response_normalizes_candidate_metadata() {
    let requested = Doi::parse("10.1000/test").unwrap();
    let body = json!({
        "status": "ok",
        "message": {
            "DOI": "10.1000/TEST",
            "title": ["Research title"],
            "author": [
                { "given": "Ada", "family": "Lovelace" },
                { "name": "Research Group" }
            ],
            "published": { "date-parts": [[2025, 4, 2]] },
            "issued": null,
            "container-title": ["Journal of Tests"]
        }
    });

    let record = parse_crossref_response(&body.to_string().into_bytes(), &requested).unwrap();

    assert_eq!(record.provider(), MetadataProvider::Crossref);
    assert_eq!(record.doi().as_str(), "10.1000/test");
    assert_eq!(record.title(), "Research title");
    assert_eq!(record.authors(), ["Ada Lovelace", "Research Group"]);
    assert_eq!(record.year(), Some(2025));
    assert_eq!(record.venue(), Some("Journal of Tests"));
    assert_eq!(record.open_access_url(), None);
}

#[test]
fn crossref_response_rejects_malformed_or_mismatched_data() {
    let requested = Doi::parse("10.1000/test").unwrap();
    for body in [
        json!({ "status": "error", "message": {} }),
        json!({ "status": "ok", "message": {
            "DOI": "10.1000/other", "title": ["Title"], "author": [],
            "published": null, "issued": null, "container-title": []
        }}),
        json!({ "status": "ok", "message": {
            "DOI": "10.1000/test", "title": [], "author": [],
            "published": null, "issued": null, "container-title": []
        }}),
    ] {
        assert_eq!(
            parse_crossref_response(&body.to_string().into_bytes(), &requested),
            Err(MetadataLookupError::InvalidResponse)
        );
    }
}
