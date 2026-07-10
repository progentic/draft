use super::*;

#[test]
fn doi_is_validated_and_normalized() {
    let doi = Doi::parse(" 10.1000/ABC-123 ").unwrap();

    assert_eq!(doi.as_str(), "10.1000/abc-123");
}

#[test]
fn malformed_dois_fail_before_network_work() {
    for value in ["", "11.1000/test", "10./test", "10.1000/", "10.1000/a b"] {
        assert_eq!(Doi::parse(value), Err(MetadataInputError::InvalidDoi));
    }
}

#[test]
fn contact_email_is_validated_and_normalized() {
    let email = ContactEmail::parse(" Research+Draft@Example.ORG ").unwrap();

    assert_eq!(email.as_str(), "research+draft@example.org");
}

#[test]
fn malformed_contact_emails_fail_before_network_work() {
    for value in [
        "",
        "missing-at.example",
        "@example.org",
        "a@localhost",
        "a b@example.org",
    ] {
        assert_eq!(
            ContactEmail::parse(value),
            Err(MetadataInputError::InvalidContactEmail)
        );
    }
}

#[test]
fn normalized_metadata_rejects_invalid_required_fields() {
    let mut parts = valid_parts();
    parts.title = "  ".to_owned();
    assert_eq!(
        metadata_record(parts),
        Err(MetadataLookupError::InvalidResponse)
    );

    let mut parts = valid_parts();
    parts.open_access_url = Some("http://example.org/paper.pdf".to_owned());
    assert_eq!(
        metadata_record(parts),
        Err(MetadataLookupError::InvalidResponse)
    );
}

#[test]
fn network_failures_map_without_raw_details() {
    assert_eq!(
        map_network_error(NetworkRequestError::RateLimited {
            retry_after_millis: 2_000
        }),
        MetadataLookupError::RateLimited {
            retry_after_millis: 2_000
        }
    );
    assert_eq!(
        map_network_error(NetworkRequestError::Timeout),
        MetadataLookupError::Timeout
    );
    assert_eq!(
        map_network_error(NetworkRequestError::Offline),
        MetadataLookupError::Offline
    );
    assert_eq!(
        map_network_error(NetworkRequestError::ResponseTooLarge),
        MetadataLookupError::ResponseTooLarge
    );
}

fn valid_parts() -> MetadataRecordParts {
    MetadataRecordParts {
        provider: MetadataProvider::Crossref,
        provider_record_id: "10.1000/test".to_owned(),
        doi: Doi::parse("10.1000/test").unwrap(),
        title: "A paper".to_owned(),
        authors: vec!["Ada Lovelace".to_owned()],
        year: Some(2025),
        venue: Some("Journal".to_owned()),
        open_access_url: Some("https://example.org/paper.pdf".to_owned()),
    }
}
