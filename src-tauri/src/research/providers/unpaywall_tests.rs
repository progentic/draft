use serde_json::json;

use super::*;

#[test]
fn unpaywall_request_uses_doi_and_required_contact() {
    let doi = Doi::parse("10.1000/test").unwrap();
    let email = ContactEmail::parse("research@example.org").unwrap();
    let url = unpaywall_request_url(&doi, &email).unwrap();

    assert_eq!(url.scheme(), "https");
    assert_eq!(url.path(), "/v2/10.1000%2Ftest");
    assert_eq!(
        url.query_pairs().collect::<Vec<_>>(),
        [("email".into(), "research@example.org".into())]
    );
}

#[test]
fn unpaywall_response_normalizes_candidate_metadata() {
    let requested = Doi::parse("10.1000/test").unwrap();
    let body = json!({
        "doi": "10.1000/TEST",
        "title": "Open result",
        "year": 2023,
        "journal_name": "Open Journal",
        "z_authors": [{ "given": "Ada", "family": "Lovelace" }],
        "best_oa_location": {
            "url_for_pdf": "https://example.org/open.pdf",
            "url_for_landing_page": "https://example.org/article"
        }
    });

    let record = parse_unpaywall_response(&body.to_string().into_bytes(), &requested).unwrap();

    assert_eq!(record.provider(), MetadataProvider::Unpaywall);
    assert_eq!(record.doi().as_str(), "10.1000/test");
    assert_eq!(record.title(), "Open result");
    assert_eq!(record.authors(), ["Ada Lovelace"]);
    assert_eq!(record.year(), Some(2023));
    assert_eq!(record.venue(), Some("Open Journal"));
    assert_eq!(
        record.open_access_url(),
        Some("https://example.org/open.pdf")
    );
}

#[test]
fn unpaywall_response_rejects_malformed_or_mismatched_data() {
    let requested = Doi::parse("10.1000/test").unwrap();
    for body in [
        json!({ "error": true }),
        json!({
            "doi": "10.1000/other", "title": "Title", "year": null,
            "journal_name": null, "z_authors": [], "best_oa_location": null
        }),
        json!({
            "doi": "10.1000/test", "title": "Title", "year": null,
            "journal_name": null, "z_authors": [],
            "best_oa_location": { "url_for_pdf": "http://example.org/file", "url_for_landing_page": null }
        }),
    ] {
        assert_eq!(
            parse_unpaywall_response(&body.to_string().into_bytes(), &requested),
            Err(MetadataLookupError::InvalidResponse)
        );
    }
}
