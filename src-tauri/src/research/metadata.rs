use std::{error::Error, fmt};

use crate::network::client::NetworkRequestError;

const MAX_DOI_LENGTH: usize = 2_048;
const MAX_CONTACT_EMAIL_LENGTH: usize = 254;

/// Validated lowercase DOI used by metadata providers.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Doi(String);

/// Validated provider contact address used only for request identification.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContactEmail(String);

/// Metadata source represented by one normalized candidate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MetadataProvider {
    Crossref,
    SemanticScholar,
    Unpaywall,
}

/// Normalized, non-persistent candidate metadata from one provider response.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MetadataRecord {
    provider: MetadataProvider,
    provider_record_id: String,
    doi: Doi,
    title: String,
    authors: Vec<String>,
    year: Option<u16>,
    venue: Option<String>,
    open_access_url: Option<String>,
}

/// Bounded failures produced before metadata network work begins.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MetadataInputError {
    InvalidDoi,
    InvalidContactEmail,
}

/// Bounded failures from provider transport or response normalization.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MetadataLookupError {
    NotFound,
    RateLimited { retry_after_millis: u64 },
    Timeout,
    Offline,
    AccessDenied,
    ServiceUnavailable,
    RequestRejected { status: u16 },
    ResponseTooLarge,
    ReadFailed,
    NetworkUnavailable,
    InvalidResponse,
}

pub(crate) struct MetadataRecordParts {
    pub(crate) provider: MetadataProvider,
    pub(crate) provider_record_id: String,
    pub(crate) doi: Doi,
    pub(crate) title: String,
    pub(crate) authors: Vec<String>,
    pub(crate) year: Option<u16>,
    pub(crate) venue: Option<String>,
    pub(crate) open_access_url: Option<String>,
}

impl Doi {
    /// Parses and normalizes one DOI without performing a lookup.
    pub fn parse(value: &str) -> Result<Self, MetadataInputError> {
        let normalized = value.trim().to_ascii_lowercase();
        if is_valid_doi(&normalized) {
            Ok(Self(normalized))
        } else {
            Err(MetadataInputError::InvalidDoi)
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl ContactEmail {
    /// Validates a bounded ASCII email used for provider identification.
    pub fn parse(value: &str) -> Result<Self, MetadataInputError> {
        let normalized = value.trim().to_ascii_lowercase();
        if is_valid_contact_email(&normalized) {
            Ok(Self(normalized))
        } else {
            Err(MetadataInputError::InvalidContactEmail)
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl MetadataRecord {
    pub fn provider(&self) -> MetadataProvider {
        self.provider
    }

    pub fn provider_record_id(&self) -> &str {
        &self.provider_record_id
    }

    pub fn doi(&self) -> &Doi {
        &self.doi
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn authors(&self) -> &[String] {
        &self.authors
    }

    pub fn year(&self) -> Option<u16> {
        self.year
    }

    pub fn venue(&self) -> Option<&str> {
        self.venue.as_deref()
    }

    pub fn open_access_url(&self) -> Option<&str> {
        self.open_access_url.as_deref()
    }
}

impl fmt::Display for MetadataInputError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidDoi => "DOI is invalid",
            Self::InvalidContactEmail => "contact email is invalid",
        })
    }
}

impl Error for MetadataInputError {}

impl fmt::Display for MetadataLookupError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.message())
    }
}

impl Error for MetadataLookupError {}

impl MetadataLookupError {
    fn message(&self) -> &'static str {
        match self {
            Self::NotFound => "metadata record was not found",
            Self::RateLimited { .. } => "metadata provider is rate limited",
            Self::Timeout => "metadata lookup timed out",
            Self::Offline => "metadata provider is unreachable",
            Self::AccessDenied => "metadata provider denied access",
            Self::ServiceUnavailable => "metadata provider is unavailable",
            Self::RequestRejected { .. } => "metadata request was rejected",
            Self::ResponseTooLarge => "metadata response is too large",
            Self::ReadFailed => "metadata response could not be read",
            Self::NetworkUnavailable => "network client state is unavailable",
            Self::InvalidResponse => "metadata response is invalid",
        }
    }
}

pub(crate) fn metadata_record(
    parts: MetadataRecordParts,
) -> Result<MetadataRecord, MetadataLookupError> {
    Ok(MetadataRecord {
        provider: parts.provider,
        provider_record_id: required_text(parts.provider_record_id)?,
        doi: parts.doi,
        title: required_text(parts.title)?,
        authors: normalized_authors(parts.authors),
        year: valid_year(parts.year)?,
        venue: optional_text(parts.venue),
        open_access_url: valid_open_access_url(parts.open_access_url)?,
    })
}

pub(crate) fn map_network_error(error: NetworkRequestError) -> MetadataLookupError {
    match error {
        NetworkRequestError::InvalidUrl => MetadataLookupError::InvalidResponse,
        NetworkRequestError::RateLimited { retry_after_millis } => {
            MetadataLookupError::RateLimited { retry_after_millis }
        }
        NetworkRequestError::Timeout => MetadataLookupError::Timeout,
        NetworkRequestError::Offline => MetadataLookupError::Offline,
        NetworkRequestError::NotFound => MetadataLookupError::NotFound,
        NetworkRequestError::AccessDenied => MetadataLookupError::AccessDenied,
        NetworkRequestError::ServiceUnavailable => MetadataLookupError::ServiceUnavailable,
        NetworkRequestError::RequestRejected { status } => {
            MetadataLookupError::RequestRejected { status }
        }
        NetworkRequestError::ResponseTooLarge => MetadataLookupError::ResponseTooLarge,
        NetworkRequestError::ReadFailed => MetadataLookupError::ReadFailed,
        NetworkRequestError::ClientUnavailable => MetadataLookupError::NetworkUnavailable,
    }
}

fn is_valid_doi(value: &str) -> bool {
    if value.is_empty() || value.len() > MAX_DOI_LENGTH || !value.starts_with("10.") {
        return false;
    }
    let Some((registrant, suffix)) = value.split_once('/') else {
        return false;
    };
    registrant.len() > 3
        && registrant[3..]
            .chars()
            .all(|character| character.is_ascii_digit())
        && !suffix.is_empty()
        && suffix.chars().all(is_printable_doi_character)
}

fn is_printable_doi_character(character: char) -> bool {
    character.is_ascii_graphic() && character != '#'
}

fn is_valid_contact_email(value: &str) -> bool {
    if value.is_empty() || value.len() > MAX_CONTACT_EMAIL_LENGTH || !value.is_ascii() {
        return false;
    }
    let Some((local, domain)) = value.split_once('@') else {
        return false;
    };
    !local.is_empty()
        && !domain.is_empty()
        && !domain.contains('@')
        && domain.contains('.')
        && value.chars().all(is_email_character)
}

fn is_email_character(character: char) -> bool {
    character.is_ascii_alphanumeric() || matches!(character, '@' | '.' | '_' | '%' | '+' | '-')
}

fn required_text(value: String) -> Result<String, MetadataLookupError> {
    optional_text(Some(value)).ok_or(MetadataLookupError::InvalidResponse)
}

fn optional_text(value: Option<String>) -> Option<String> {
    value.and_then(|text| {
        let normalized = text.trim();
        (!normalized.is_empty()).then(|| normalized.to_owned())
    })
}

fn normalized_authors(authors: Vec<String>) -> Vec<String> {
    authors
        .into_iter()
        .filter_map(|author| optional_text(Some(author)))
        .collect()
}

fn valid_year(year: Option<u16>) -> Result<Option<u16>, MetadataLookupError> {
    match year {
        Some(0) => Err(MetadataLookupError::InvalidResponse),
        value => Ok(value),
    }
}

fn valid_open_access_url(url: Option<String>) -> Result<Option<String>, MetadataLookupError> {
    let Some(url) = optional_text(url) else {
        return Ok(None);
    };
    let parsed = url::Url::parse(&url).map_err(|_| MetadataLookupError::InvalidResponse)?;
    if parsed.scheme() != "https" {
        return Err(MetadataLookupError::InvalidResponse);
    }
    Ok(Some(url))
}

#[cfg(test)]
#[path = "metadata_tests.rs"]
mod tests;
