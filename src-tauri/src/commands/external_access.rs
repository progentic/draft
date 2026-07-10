use serde::{Deserialize, Serialize};

use crate::{
    research::external_access::{
        ExternalAccessDestination, ExternalAccessError, ExternalAccessInput, open_in_system_browser,
    },
    system_browser::SystemBrowser,
};

/// Tagged external destination submitted for Rust validation and browser handoff.
#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(tag = "destination", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum OpenExternalAccessRequest {
    Publisher { url: String },
    Institutional { url: String },
    Doi { doi: String },
    GoogleScholar { query: String },
}

/// Stable acknowledgement returned after the OS accepts a browser handoff.
#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OpenExternalAccessResponse {
    status: ExternalAccessStatus,
    destination: ExternalAccessDestination,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum ExternalAccessStatus {
    Opened,
}

/// Stable failures exposed by `open_external_access` over Tauri IPC.
#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "code", rename_all = "snake_case")]
pub(crate) enum OpenExternalAccessError {
    InvalidUrl,
    InvalidDoi,
    InvalidSearchQuery,
    BrowserUnavailable,
}

/// Opens one validated research destination in the default system browser.
#[tauri::command]
pub(crate) fn open_external_access(
    request: OpenExternalAccessRequest,
) -> Result<OpenExternalAccessResponse, OpenExternalAccessError> {
    let destination = open_in_system_browser(&SystemBrowser, request.into())?;
    Ok(OpenExternalAccessResponse::opened(destination))
}

impl OpenExternalAccessResponse {
    fn opened(destination: ExternalAccessDestination) -> Self {
        Self {
            status: ExternalAccessStatus::Opened,
            destination,
        }
    }
}

impl From<OpenExternalAccessRequest> for ExternalAccessInput {
    fn from(request: OpenExternalAccessRequest) -> Self {
        match request {
            OpenExternalAccessRequest::Publisher { url } => Self::PublisherUrl(url),
            OpenExternalAccessRequest::Institutional { url } => Self::InstitutionalUrl(url),
            OpenExternalAccessRequest::Doi { doi } => Self::Doi(doi),
            OpenExternalAccessRequest::GoogleScholar { query } => Self::GoogleScholarQuery(query),
        }
    }
}

impl From<ExternalAccessError> for OpenExternalAccessError {
    fn from(error: ExternalAccessError) -> Self {
        match error {
            ExternalAccessError::InvalidUrl => Self::InvalidUrl,
            ExternalAccessError::InvalidDoi => Self::InvalidDoi,
            ExternalAccessError::InvalidSearchQuery => Self::InvalidSearchQuery,
            ExternalAccessError::BrowserUnavailable => Self::BrowserUnavailable,
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    const TYPED_COMMAND: fn(
        OpenExternalAccessRequest,
    ) -> Result<OpenExternalAccessResponse, OpenExternalAccessError> = open_external_access;

    #[test]
    fn command_signature_is_typed() {
        let _ = TYPED_COMMAND;
    }

    #[test]
    fn request_deserialization_is_stable() {
        let requests = [
            json!({ "destination": "publisher", "url": "https://publisher.example" }),
            json!({ "destination": "institutional", "url": "https://library.example" }),
            json!({ "destination": "doi", "doi": "10.1000/example" }),
            json!({ "destination": "google_scholar", "query": "example title" }),
        ];
        for request in requests {
            assert!(serde_json::from_value::<OpenExternalAccessRequest>(request).is_ok());
        }

        for request in [
            json!({ "destination": "publisher", "url": "https://example.org", "extra": true }),
            json!({ "destination": "doi", "url": "https://doi.org/10.1000/example" }),
            json!({ "destination": "google_scholar" }),
            json!({ "destination": "embedded_browser", "url": "https://example.org" }),
        ] {
            assert!(serde_json::from_value::<OpenExternalAccessRequest>(request).is_err());
        }
    }

    #[test]
    fn response_serialization_is_stable() {
        let responses = [
            ExternalAccessDestination::Publisher,
            ExternalAccessDestination::Institutional,
            ExternalAccessDestination::Doi,
            ExternalAccessDestination::GoogleScholar,
        ]
        .map(OpenExternalAccessResponse::opened);

        assert_eq!(
            serde_json::to_value(responses).unwrap(),
            json!([
                { "status": "opened", "destination": "publisher" },
                { "status": "opened", "destination": "institutional" },
                { "status": "opened", "destination": "doi" },
                { "status": "opened", "destination": "google_scholar" }
            ])
        );
    }

    #[test]
    fn error_serialization_is_stable() {
        let errors = [
            OpenExternalAccessError::from(ExternalAccessError::InvalidUrl),
            OpenExternalAccessError::from(ExternalAccessError::InvalidDoi),
            OpenExternalAccessError::from(ExternalAccessError::InvalidSearchQuery),
            OpenExternalAccessError::from(ExternalAccessError::BrowserUnavailable),
        ];

        assert_eq!(
            serde_json::to_value(errors).unwrap(),
            json!([
                { "code": "invalid_url" },
                { "code": "invalid_doi" },
                { "code": "invalid_search_query" },
                { "code": "browser_unavailable" }
            ])
        );
    }
}
