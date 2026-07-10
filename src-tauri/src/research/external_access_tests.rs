use std::cell::RefCell;

use super::*;

#[test]
fn publisher_and_institutional_urls_open_as_validated_https() {
    let browser = RecordingBrowser::available();

    assert_eq!(
        open_in_system_browser(
            &browser,
            ExternalAccessInput::PublisherUrl(" https://publisher.example/article ".to_owned()),
        ),
        Ok(ExternalAccessDestination::Publisher)
    );
    assert_eq!(
        open_in_system_browser(
            &browser,
            ExternalAccessInput::InstitutionalUrl("https://library.example/item".to_owned()),
        ),
        Ok(ExternalAccessDestination::Institutional)
    );
    assert_eq!(
        browser.opened_urls(),
        [
            "https://publisher.example/article",
            "https://library.example/item"
        ]
    );
}

#[test]
fn non_https_or_credentialed_urls_fail_before_browser_launch() {
    let browser = RecordingBrowser::available();

    for value in [
        "http://publisher.example/article",
        "file:///tmp/article.pdf",
        "javascript:alert(1)",
        "https://user:password@publisher.example/article",
        "https://user@publisher.example/article",
        "not a URL",
        "",
    ] {
        assert_eq!(
            open_in_system_browser(
                &browser,
                ExternalAccessInput::PublisherUrl(value.to_owned()),
            ),
            Err(ExternalAccessError::InvalidUrl)
        );
    }
    assert_eq!(
        open_in_system_browser(
            &browser,
            ExternalAccessInput::PublisherUrl(format!(
                "https://publisher.example/{}",
                "x".repeat(MAX_EXTERNAL_URL_LENGTH)
            )),
        ),
        Err(ExternalAccessError::InvalidUrl)
    );
    assert!(browser.opened_urls().is_empty());
}

#[test]
fn doi_handoff_builds_resolver_url() {
    let browser = RecordingBrowser::available();

    assert_eq!(
        open_in_system_browser(
            &browser,
            ExternalAccessInput::Doi(" 10.1000/Example/Part?A ".to_owned()),
        ),
        Ok(ExternalAccessDestination::Doi)
    );
    assert_eq!(
        browser.opened_urls(),
        ["https://doi.org/10.1000/example/part%3Fa"]
    );
}

#[test]
fn google_scholar_handoff_builds_bounded_search_url() {
    let browser = RecordingBrowser::available();

    assert_eq!(
        open_in_system_browser(
            &browser,
            ExternalAccessInput::GoogleScholarQuery(" source reliability ".to_owned()),
        ),
        Ok(ExternalAccessDestination::GoogleScholar)
    );
    assert_eq!(
        browser.opened_urls(),
        ["https://scholar.google.com/scholar?q=source+reliability"]
    );
}

#[test]
fn malformed_doi_and_query_fail_before_browser_launch() {
    let browser = RecordingBrowser::available();

    for value in ["", "example", "10.x/test"] {
        assert_eq!(
            open_in_system_browser(&browser, ExternalAccessInput::Doi(value.to_owned())),
            Err(ExternalAccessError::InvalidDoi)
        );
    }
    for value in ["", "   ", "line\nbreak"] {
        assert_eq!(
            open_in_system_browser(
                &browser,
                ExternalAccessInput::GoogleScholarQuery(value.to_owned()),
            ),
            Err(ExternalAccessError::InvalidSearchQuery)
        );
    }
    assert_eq!(
        open_in_system_browser(
            &browser,
            ExternalAccessInput::GoogleScholarQuery("x".repeat(MAX_SCHOLAR_QUERY_LENGTH + 1)),
        ),
        Err(ExternalAccessError::InvalidSearchQuery)
    );
    assert!(browser.opened_urls().is_empty());
}

#[test]
fn browser_launch_failures_are_bounded() {
    let browser = RecordingBrowser::unavailable();

    let error = open_in_system_browser(
        &browser,
        ExternalAccessInput::PublisherUrl("https://publisher.example/article".to_owned()),
    )
    .unwrap_err();

    assert_eq!(error, ExternalAccessError::BrowserUnavailable);
    assert_eq!(error.to_string(), "system browser could not be opened");
}

struct RecordingBrowser {
    opened_urls: RefCell<Vec<String>>,
    available: bool,
}

impl RecordingBrowser {
    fn available() -> Self {
        Self {
            opened_urls: RefCell::new(Vec::new()),
            available: true,
        }
    }

    fn unavailable() -> Self {
        Self {
            opened_urls: RefCell::new(Vec::new()),
            available: false,
        }
    }

    fn opened_urls(&self) -> Vec<String> {
        self.opened_urls.borrow().clone()
    }
}

impl ExternalUrlOpener for RecordingBrowser {
    fn open_external_url(&self, url: &Url) -> Result<(), ExternalUrlOpenError> {
        if !self.available {
            return Err(ExternalUrlOpenError);
        }
        self.opened_urls.borrow_mut().push(url.as_str().to_owned());
        Ok(())
    }
}
