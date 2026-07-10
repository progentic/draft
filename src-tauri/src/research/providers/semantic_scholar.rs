use serde::Deserialize;
use url::Url;

use crate::{
    network::client::{NetworkClient, NetworkService},
    research::metadata::{
        Doi, MetadataLookupError, MetadataProvider, MetadataRecord, MetadataRecordParts,
        map_network_error, metadata_record,
    },
};

const SEMANTIC_SCHOLAR_BASE_URL: &str = "https://api.semanticscholar.org/graph/v1";
const SEMANTIC_SCHOLAR_FIELDS: &str = "paperId,title,authors,year,venue,externalIds,openAccessPdf";

/// Retrieves one DOI record from the Semantic Scholar Academic Graph API.
pub async fn lookup_semantic_scholar(
    client: &NetworkClient,
    doi: &Doi,
) -> Result<MetadataRecord, MetadataLookupError> {
    let url = semantic_scholar_request_url(doi)?;
    let body = client
        .get_metadata(NetworkService::SemanticScholar, url.as_str())
        .await
        .map_err(map_network_error)?;
    parse_semantic_scholar_response(&body, doi)
}

fn semantic_scholar_request_url(doi: &Doi) -> Result<Url, MetadataLookupError> {
    let mut url = provider_url(SEMANTIC_SCHOLAR_BASE_URL)?;
    let paper_id = format!("DOI:{}", doi.as_str());
    url.path_segments_mut()
        .map_err(|_| MetadataLookupError::InvalidResponse)?
        .extend(["paper", paper_id.as_str()]);
    url.query_pairs_mut()
        .append_pair("fields", SEMANTIC_SCHOLAR_FIELDS);
    Ok(url)
}

fn parse_semantic_scholar_response(
    body: &[u8],
    requested_doi: &Doi,
) -> Result<MetadataRecord, MetadataLookupError> {
    let paper: SemanticScholarPaper =
        serde_json::from_slice(body).map_err(|_| MetadataLookupError::InvalidResponse)?;
    let doi = semantic_scholar_doi(paper.external_ids, requested_doi)?;
    metadata_record(MetadataRecordParts {
        provider: MetadataProvider::SemanticScholar,
        provider_record_id: paper.paper_id,
        doi,
        title: paper.title,
        authors: paper
            .authors
            .into_iter()
            .map(|author| author.name)
            .collect(),
        year: paper.year,
        venue: paper.venue,
        open_access_url: paper.open_access_pdf.map(|location| location.url),
    })
}

fn semantic_scholar_doi(
    external_ids: Option<SemanticScholarExternalIds>,
    requested: &Doi,
) -> Result<Doi, MetadataLookupError> {
    let Some(value) = external_ids.and_then(|ids| ids.doi) else {
        return Ok(requested.clone());
    };
    matching_doi(&value, requested)
}

fn matching_doi(value: &str, requested: &Doi) -> Result<Doi, MetadataLookupError> {
    let doi = Doi::parse(value).map_err(|_| MetadataLookupError::InvalidResponse)?;
    if &doi == requested {
        Ok(doi)
    } else {
        Err(MetadataLookupError::InvalidResponse)
    }
}

fn provider_url(base: &str) -> Result<Url, MetadataLookupError> {
    Url::parse(base).map_err(|_| MetadataLookupError::InvalidResponse)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SemanticScholarPaper {
    paper_id: String,
    title: String,
    #[serde(default)]
    authors: Vec<SemanticScholarAuthor>,
    year: Option<u16>,
    venue: Option<String>,
    external_ids: Option<SemanticScholarExternalIds>,
    open_access_pdf: Option<SemanticScholarOpenAccess>,
}

#[derive(Deserialize)]
struct SemanticScholarAuthor {
    name: String,
}

#[derive(Deserialize)]
struct SemanticScholarExternalIds {
    #[serde(rename = "DOI")]
    doi: Option<String>,
}

#[derive(Deserialize)]
struct SemanticScholarOpenAccess {
    url: String,
}

#[cfg(test)]
#[path = "semantic_scholar_tests.rs"]
mod tests;
