use serde::Deserialize;
use url::Url;

use crate::{
    network::client::{NetworkClient, NetworkService},
    research::metadata::{
        ContactEmail, Doi, MetadataLookupError, MetadataProvider, MetadataRecord,
        MetadataRecordParts, map_network_error, metadata_record,
    },
};

const UNPAYWALL_BASE_URL: &str = "https://api.unpaywall.org/v2";

/// Retrieves one DOI record from the Unpaywall REST API.
pub async fn lookup_unpaywall(
    client: &NetworkClient,
    doi: &Doi,
    contact_email: &ContactEmail,
) -> Result<MetadataRecord, MetadataLookupError> {
    let url = unpaywall_request_url(doi, contact_email)?;
    let body = client
        .get_metadata(NetworkService::Unpaywall, url.as_str())
        .await
        .map_err(map_network_error)?;
    parse_unpaywall_response(&body, doi)
}

fn unpaywall_request_url(
    doi: &Doi,
    contact_email: &ContactEmail,
) -> Result<Url, MetadataLookupError> {
    let mut url = provider_url(UNPAYWALL_BASE_URL)?;
    url.path_segments_mut()
        .map_err(|_| MetadataLookupError::InvalidResponse)?
        .push(doi.as_str());
    url.query_pairs_mut()
        .append_pair("email", contact_email.as_str());
    Ok(url)
}

fn parse_unpaywall_response(
    body: &[u8],
    requested_doi: &Doi,
) -> Result<MetadataRecord, MetadataLookupError> {
    let work: UnpaywallWork =
        serde_json::from_slice(body).map_err(|_| MetadataLookupError::InvalidResponse)?;
    let doi = matching_doi(&work.doi, requested_doi)?;
    metadata_record(MetadataRecordParts {
        provider: MetadataProvider::Unpaywall,
        provider_record_id: work.doi,
        doi,
        title: work.title,
        authors: work.z_authors.into_iter().filter_map(author_name).collect(),
        year: work.year,
        venue: work.journal_name,
        open_access_url: open_access_url(work.best_oa_location),
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

fn author_name(author: UnpaywallAuthor) -> Option<String> {
    let name = [author.given, author.family]
        .into_iter()
        .flatten()
        .filter_map(non_blank_text)
        .collect::<Vec<_>>()
        .join(" ");
    (!name.is_empty()).then_some(name)
}

fn open_access_url(location: Option<UnpaywallLocation>) -> Option<String> {
    let location = location?;
    location.url_for_pdf.or(location.url_for_landing_page)
}

fn non_blank_text(value: String) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_owned())
}

fn provider_url(base: &str) -> Result<Url, MetadataLookupError> {
    Url::parse(base).map_err(|_| MetadataLookupError::InvalidResponse)
}

#[derive(Deserialize)]
struct UnpaywallWork {
    doi: String,
    title: String,
    year: Option<u16>,
    journal_name: Option<String>,
    #[serde(default)]
    z_authors: Vec<UnpaywallAuthor>,
    best_oa_location: Option<UnpaywallLocation>,
}

#[derive(Deserialize)]
struct UnpaywallAuthor {
    given: Option<String>,
    family: Option<String>,
}

#[derive(Deserialize)]
struct UnpaywallLocation {
    url_for_pdf: Option<String>,
    url_for_landing_page: Option<String>,
}

#[cfg(test)]
#[path = "unpaywall_tests.rs"]
mod tests;
