use serde::Deserialize;
use url::Url;

use crate::{
    network::client::{NetworkClient, NetworkService},
    research::metadata::{
        ContactEmail, Doi, MetadataLookupError, MetadataProvider, MetadataRecord,
        MetadataRecordParts, map_network_error, metadata_record,
    },
};

const CROSSREF_BASE_URL: &str = "https://api.crossref.org/v1";

/// Retrieves one DOI record from the Crossref REST API.
pub async fn lookup_crossref(
    client: &NetworkClient,
    doi: &Doi,
    contact_email: &ContactEmail,
) -> Result<MetadataRecord, MetadataLookupError> {
    let url = crossref_request_url(doi, contact_email)?;
    let body = client
        .get_metadata(NetworkService::Crossref, url.as_str())
        .await
        .map_err(map_network_error)?;
    parse_crossref_response(&body, doi)
}

fn crossref_request_url(
    doi: &Doi,
    contact_email: &ContactEmail,
) -> Result<Url, MetadataLookupError> {
    let mut url = provider_url(CROSSREF_BASE_URL)?;
    url.path_segments_mut()
        .map_err(|_| MetadataLookupError::InvalidResponse)?
        .extend(["works", doi.as_str()]);
    url.query_pairs_mut()
        .append_pair("mailto", contact_email.as_str());
    Ok(url)
}

fn parse_crossref_response(
    body: &[u8],
    requested_doi: &Doi,
) -> Result<MetadataRecord, MetadataLookupError> {
    let response: CrossrefResponse =
        serde_json::from_slice(body).map_err(|_| MetadataLookupError::InvalidResponse)?;
    if response.status != "ok" {
        return Err(MetadataLookupError::InvalidResponse);
    }
    normalize_crossref_work(response.message, requested_doi)
}

fn normalize_crossref_work(
    work: CrossrefWork,
    requested_doi: &Doi,
) -> Result<MetadataRecord, MetadataLookupError> {
    let doi = matching_doi(&work.doi, requested_doi)?;
    metadata_record(MetadataRecordParts {
        provider: MetadataProvider::Crossref,
        provider_record_id: work.doi,
        doi,
        title: first_text(work.title).unwrap_or_default(),
        authors: work.author.into_iter().filter_map(author_name).collect(),
        year: publication_year(work.published.or(work.issued)),
        venue: first_text(work.container_title),
        open_access_url: None,
    })
}

fn matching_doi(value: &str, requested: &Doi) -> Result<Doi, MetadataLookupError> {
    let doi = Doi::parse(value).map_err(|_| MetadataLookupError::InvalidResponse)?;
    if &doi == requested {
        Ok(doi)
    } else {
        Err(MetadataLookupError::InvalidResponse)
    }
}

fn author_name(author: CrossrefAuthor) -> Option<String> {
    let person = [author.given, author.family]
        .into_iter()
        .flatten()
        .filter_map(non_blank_text)
        .collect::<Vec<_>>()
        .join(" ");
    if person.is_empty() {
        author.name.and_then(non_blank_text)
    } else {
        Some(person)
    }
}

fn publication_year(date: Option<CrossrefDate>) -> Option<u16> {
    date?.date_parts.first()?.first().copied()
}

fn first_text(values: Vec<String>) -> Option<String> {
    values.into_iter().find_map(non_blank_text)
}

fn non_blank_text(value: String) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_owned())
}

fn provider_url(base: &str) -> Result<Url, MetadataLookupError> {
    Url::parse(base).map_err(|_| MetadataLookupError::InvalidResponse)
}

#[derive(Deserialize)]
struct CrossrefResponse {
    status: String,
    message: CrossrefWork,
}

#[derive(Deserialize)]
struct CrossrefWork {
    #[serde(rename = "DOI")]
    doi: String,
    #[serde(default)]
    title: Vec<String>,
    #[serde(default)]
    author: Vec<CrossrefAuthor>,
    published: Option<CrossrefDate>,
    issued: Option<CrossrefDate>,
    #[serde(rename = "container-title", default)]
    container_title: Vec<String>,
}

#[derive(Deserialize)]
struct CrossrefAuthor {
    given: Option<String>,
    family: Option<String>,
    name: Option<String>,
}

#[derive(Deserialize)]
struct CrossrefDate {
    #[serde(rename = "date-parts")]
    date_parts: Vec<Vec<u16>>,
}

#[cfg(test)]
#[path = "crossref_tests.rs"]
mod tests;
