use std::{error::Error, fmt};

use serde::Serialize;
use url::Url;

use crate::network::connectivity::{ConnectivityPolicy, ConnectivityPolicyError};

use super::metadata::Doi;

const MAX_EXTERNAL_URL_LENGTH: usize = 2_048;
const MAX_SCHOLAR_QUERY_LENGTH: usize = 2_048;
const DOI_RESOLVER_BASE_URL: &str = "https://doi.org";
const GOOGLE_SCHOLAR_BASE_URL: &str = "https://scholar.google.com/scholar";

/// Browser destination reported after a successful external handoff.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ExternalAccessDestination {
    Publisher,
    Institutional,
    Doi,
    GoogleScholar,
}

/// Untrusted input accepted by the external-access domain boundary.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum ExternalAccessInput {
    PublisherUrl(String),
    InstitutionalUrl(String),
    Doi(String),
    GoogleScholarQuery(String),
}

/// Bounded failures produced before or during a system-browser handoff.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ExternalAccessError {
    InvalidUrl,
    InvalidDoi,
    InvalidSearchQuery,
    Offline,
    ConnectivityUnavailable,
    BrowserUnavailable,
}

/// Opaque failure returned by a concrete system-browser adapter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ExternalUrlOpenError;

/// Narrow adapter used to keep OS browser execution replaceable in tests.
pub(crate) trait ExternalUrlOpener {
    fn open_external_url(&self, url: &Url) -> Result<(), ExternalUrlOpenError>;
}

/// Validates one target and opens it in the default system browser.
pub(crate) fn open_in_system_browser(
    connectivity: &ConnectivityPolicy,
    opener: &impl ExternalUrlOpener,
    input: ExternalAccessInput,
) -> Result<ExternalAccessDestination, ExternalAccessError> {
    connectivity
        .require_online()
        .map_err(ExternalAccessError::from)?;
    let (destination, url) = external_access_url(input)?;
    opener
        .open_external_url(&url)
        .map_err(|_| ExternalAccessError::BrowserUnavailable)?;
    Ok(destination)
}

impl fmt::Display for ExternalAccessError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidUrl => "external URL is invalid",
            Self::InvalidDoi => "DOI is invalid",
            Self::InvalidSearchQuery => "Google Scholar query is invalid",
            Self::Offline => "external access is paused while DRAFT is offline",
            Self::ConnectivityUnavailable => "connectivity state is unavailable",
            Self::BrowserUnavailable => "system browser could not be opened",
        })
    }
}

impl Error for ExternalAccessError {}

impl From<ConnectivityPolicyError> for ExternalAccessError {
    fn from(error: ConnectivityPolicyError) -> Self {
        match error {
            ConnectivityPolicyError::Offline => Self::Offline,
            ConnectivityPolicyError::Unavailable => Self::ConnectivityUnavailable,
        }
    }
}

fn external_access_url(
    input: ExternalAccessInput,
) -> Result<(ExternalAccessDestination, Url), ExternalAccessError> {
    match input {
        ExternalAccessInput::PublisherUrl(value) => Ok((
            ExternalAccessDestination::Publisher,
            validated_external_url(value)?,
        )),
        ExternalAccessInput::InstitutionalUrl(value) => Ok((
            ExternalAccessDestination::Institutional,
            validated_external_url(value)?,
        )),
        ExternalAccessInput::Doi(value) => {
            Ok((ExternalAccessDestination::Doi, doi_resolver_url(value)?))
        }
        ExternalAccessInput::GoogleScholarQuery(value) => Ok((
            ExternalAccessDestination::GoogleScholar,
            google_scholar_url(value)?,
        )),
    }
}

fn validated_external_url(value: String) -> Result<Url, ExternalAccessError> {
    if value.len() > MAX_EXTERNAL_URL_LENGTH {
        return Err(ExternalAccessError::InvalidUrl);
    }
    let url = Url::parse(value.trim()).map_err(|_| ExternalAccessError::InvalidUrl)?;
    if is_valid_external_url(&url) {
        Ok(url)
    } else {
        Err(ExternalAccessError::InvalidUrl)
    }
}

fn is_valid_external_url(url: &Url) -> bool {
    url.scheme() == "https"
        && url.host_str().is_some()
        && url.username().is_empty()
        && url.password().is_none()
}

fn doi_resolver_url(value: String) -> Result<Url, ExternalAccessError> {
    let doi = Doi::parse(&value).map_err(|_| ExternalAccessError::InvalidDoi)?;
    let mut url = Url::parse(DOI_RESOLVER_BASE_URL).map_err(|_| ExternalAccessError::InvalidDoi)?;
    url.path_segments_mut()
        .map_err(|_| ExternalAccessError::InvalidDoi)?
        .extend(doi.as_str().split('/'));
    Ok(url)
}

fn google_scholar_url(value: String) -> Result<Url, ExternalAccessError> {
    let query = validated_scholar_query(&value)?;
    let mut url =
        Url::parse(GOOGLE_SCHOLAR_BASE_URL).map_err(|_| ExternalAccessError::InvalidSearchQuery)?;
    url.query_pairs_mut().append_pair("q", query);
    Ok(url)
}

fn validated_scholar_query(value: &str) -> Result<&str, ExternalAccessError> {
    let query = value.trim();
    if value.len() > MAX_SCHOLAR_QUERY_LENGTH
        || query.is_empty()
        || query.chars().any(char::is_control)
    {
        return Err(ExternalAccessError::InvalidSearchQuery);
    }
    Ok(query)
}

#[cfg(test)]
#[path = "external_access_tests.rs"]
mod tests;
