use std::{collections::HashSet, error::Error, fmt};

use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{Map, Value};
use uuid::Uuid;

const SCHEMA_VERSION_FIELD: &str = "schema_version";
const REFERENCE_ID_FIELD: &str = "reference_id";
const CITEKEY_FIELD: &str = "citekey";
const KIND_FIELD: &str = "kind";
const TITLE_FIELD: &str = "title";
const CONTRIBUTORS_FIELD: &str = "contributors";
const ISSUED_FIELD: &str = "issued";
const CONTAINER_TITLE_FIELD: &str = "container_title";
const PUBLISHER_FIELD: &str = "publisher";
const VOLUME_FIELD: &str = "volume";
const ISSUE_FIELD: &str = "issue";
const PAGES_FIELD: &str = "pages";
const RESOLUTION_STATE_FIELD: &str = "resolution_state";
const IDENTIFIERS_FIELD: &str = "identifiers";
const PROVENANCE_FIELD: &str = "provenance";
const RECORD_FIELDS: [&str; 15] = [
    SCHEMA_VERSION_FIELD,
    REFERENCE_ID_FIELD,
    CITEKEY_FIELD,
    KIND_FIELD,
    TITLE_FIELD,
    CONTRIBUTORS_FIELD,
    ISSUED_FIELD,
    CONTAINER_TITLE_FIELD,
    PUBLISHER_FIELD,
    VOLUME_FIELD,
    ISSUE_FIELD,
    PAGES_FIELD,
    RESOLUTION_STATE_FIELD,
    IDENTIFIERS_FIELD,
    PROVENANCE_FIELD,
];

/// Current reference-record schema accepted by the Rust core.
///
/// A different version must fail validation until an explicit migration owns it.
pub const REFERENCE_RECORD_SCHEMA_VERSION: u64 = 1;

/// Validated version 1 reference record with no persistence lifecycle.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ReferenceRecord {
    #[serde(flatten)]
    identity: ReferenceIdentity,
    #[serde(flatten)]
    bibliography: BibliographicDetails,
    #[serde(flatten)]
    tracking: ReferenceTracking,
}

/// Stable reference identity parsed and serialized by the Rust core.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize)]
#[serde(transparent)]
pub struct ReferenceId(Uuid);

/// Supported normalized reference kinds for schema version 1.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum ReferenceKind {
    Article,
    Book,
    Chapter,
    Report,
    Thesis,
    Webpage,
    Other,
}

/// Supported contributor roles for schema version 1.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum ContributorRole {
    Author,
    Editor,
    Translator,
}

/// Validated person or organization contributor name.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ContributorName {
    Person {
        given: Option<String>,
        family: Option<String>,
    },
    Organization {
        literal: String,
    },
}

/// One validated contributor and its bibliographic role.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct ReferenceContributor {
    role: ContributorRole,
    name: ContributorName,
}

/// Validated partial publication date.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
struct IssuedDate {
    year: u16,
    month: Option<u8>,
    day: Option<u8>,
}

/// Validated identifier group retained without network resolution.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct ReferenceIdentifiers {
    doi: Option<String>,
    isbn: Vec<String>,
    url: Option<String>,
}

/// Descriptive metadata-resolution state; Phase 16 performs no transitions.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum ResolutionState {
    Unresolved,
    Resolved,
    NeedsReview,
}

/// Declared origin of normalized reference metadata.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum ReferenceSource {
    Manual,
    Crossref,
    SemanticScholar,
    Unpaywall,
    PdfImport,
}

/// Bibliographic field protected from a future metadata merge.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum ManualOverrideField {
    Kind,
    Title,
    Contributors,
    Issued,
    ContainerTitle,
    Publisher,
    Volume,
    Issue,
    Pages,
    Identifiers,
}

/// Validated source provenance without lookup, scoring, or merge behavior.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct ReferenceProvenance {
    source: ReferenceSource,
    source_record_id: Option<String>,
    manual_overrides: Vec<ManualOverrideField>,
}

/// Bounded failures produced while validating an untrusted reference record.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum ReferenceRecordError {
    InvalidReferenceObject,
    UnknownReferenceField { path: String },
    MissingSchemaVersion,
    InvalidSchemaVersion,
    UnsupportedSchemaVersion { found: u64 },
    MissingReferenceId,
    InvalidReferenceId,
    MissingCitekey,
    InvalidCitekey,
    MissingKind,
    UnsupportedReferenceKind,
    MissingTitle,
    InvalidTitle,
    MissingContributors,
    InvalidContributors,
    InvalidContributor { index: usize },
    MissingIssued,
    InvalidIssuedDate,
    MissingContainerTitle,
    InvalidContainerTitle,
    MissingPublisher,
    InvalidPublisher,
    MissingVolume,
    InvalidVolume,
    MissingIssue,
    InvalidIssue,
    MissingPages,
    InvalidPages,
    MissingResolutionState,
    InvalidResolutionState,
    MissingIdentifiers,
    InvalidIdentifiers,
    InvalidIsbn { index: usize },
    MissingProvenance,
    InvalidProvenance,
    InvalidProvenanceSource,
    InvalidManualOverride { index: usize },
}

#[derive(Clone, Debug, PartialEq, Serialize)]
struct ReferenceIdentity {
    schema_version: u64,
    reference_id: ReferenceId,
    citekey: String,
    kind: ReferenceKind,
    title: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
struct BibliographicDetails {
    contributors: Vec<ReferenceContributor>,
    issued: Option<IssuedDate>,
    container_title: Option<String>,
    publisher: Option<String>,
    volume: Option<String>,
    issue: Option<String>,
    pages: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
struct ReferenceTracking {
    resolution_state: ResolutionState,
    identifiers: ReferenceIdentifiers,
    provenance: ReferenceProvenance,
}

impl ReferenceRecord {
    /// Validates an untrusted JSON value without persistence, IPC, or network work.
    pub fn from_json_value(value: Value) -> Result<Self, ReferenceRecordError> {
        let mut fields = reference_fields(value)?;
        reject_unknown_fields(&fields, &RECORD_FIELDS, "")?;

        Ok(Self {
            identity: parse_identity(&mut fields)?,
            bibliography: parse_bibliography(&mut fields)?,
            tracking: parse_tracking(&mut fields)?,
        })
    }

    /// Returns the validated schema version.
    pub fn schema_version(&self) -> u64 {
        self.identity.schema_version
    }

    /// Returns the Rust-validated reference identity.
    pub fn reference_id(&self) -> ReferenceId {
        self.identity.reference_id
    }

    /// Returns the validated case-sensitive citation key.
    pub fn citekey(&self) -> &str {
        &self.identity.citekey
    }
}

impl<'de> Deserialize<'de> for ReferenceRecord {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        Self::from_json_value(value).map_err(serde::de::Error::custom)
    }
}

impl fmt::Display for ReferenceId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl fmt::Display for ReferenceRecordError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.message())
    }
}

impl Error for ReferenceRecordError {}

impl ReferenceRecordError {
    fn message(&self) -> &'static str {
        match self {
            Self::InvalidReferenceObject => "reference record must be an object",
            Self::UnknownReferenceField { .. } => "reference record contains an unknown field",
            Self::MissingSchemaVersion => "reference record is missing schema_version",
            Self::InvalidSchemaVersion => "schema_version must be an unsigned integer",
            Self::UnsupportedSchemaVersion { .. } => "schema_version is not supported",
            Self::MissingReferenceId => "reference record is missing reference_id",
            Self::InvalidReferenceId => "reference_id must be a UUID",
            Self::MissingCitekey => "reference record is missing citekey",
            Self::InvalidCitekey => "citekey has an invalid shape",
            Self::MissingKind => "reference record is missing kind",
            Self::UnsupportedReferenceKind => "reference kind is not supported",
            Self::MissingTitle => "reference record is missing title",
            Self::InvalidTitle => "title must be a non-blank string",
            Self::MissingContributors => "reference record is missing contributors",
            Self::InvalidContributors => "contributors must be an array",
            Self::InvalidContributor { .. } => "contributor has an invalid shape",
            Self::MissingIssued => "reference record is missing issued",
            Self::InvalidIssuedDate => "issued date has an invalid shape",
            Self::MissingContainerTitle => "reference record is missing container_title",
            Self::InvalidContainerTitle => "container_title must be null or non-blank text",
            Self::MissingPublisher => "reference record is missing publisher",
            Self::InvalidPublisher => "publisher must be null or non-blank text",
            Self::MissingVolume => "reference record is missing volume",
            Self::InvalidVolume => "volume must be null or non-blank text",
            Self::MissingIssue => "reference record is missing issue",
            Self::InvalidIssue => "issue must be null or non-blank text",
            Self::MissingPages => "reference record is missing pages",
            Self::InvalidPages => "pages must be null or non-blank text",
            Self::MissingResolutionState => "reference record is missing resolution_state",
            Self::InvalidResolutionState => "resolution_state is not supported",
            Self::MissingIdentifiers => "reference record is missing identifiers",
            Self::InvalidIdentifiers => "identifiers have an invalid shape",
            Self::InvalidIsbn { .. } => "ISBN entry must be non-blank text",
            Self::MissingProvenance => "reference record is missing provenance",
            Self::InvalidProvenance => "provenance has an invalid shape",
            Self::InvalidProvenanceSource => "provenance source is not supported",
            Self::InvalidManualOverride { .. } => "manual override field is invalid",
        }
    }
}

fn parse_identity(
    fields: &mut Map<String, Value>,
) -> Result<ReferenceIdentity, ReferenceRecordError> {
    Ok(ReferenceIdentity {
        schema_version: parse_schema_version(take_schema_version(fields)?)?,
        reference_id: parse_reference_id(take_reference_id(fields)?)?,
        citekey: parse_citekey(take_citekey(fields)?)?,
        kind: parse_reference_kind(take_kind(fields)?)?,
        title: parse_title(take_title(fields)?)?,
    })
}

fn parse_bibliography(
    fields: &mut Map<String, Value>,
) -> Result<BibliographicDetails, ReferenceRecordError> {
    Ok(BibliographicDetails {
        contributors: parse_contributors(take_contributors(fields)?)?,
        issued: parse_issued_date(take_issued(fields)?)?,
        container_title: parse_container_title(take_container_title(fields)?)?,
        publisher: parse_publisher(take_publisher(fields)?)?,
        volume: parse_volume(take_volume(fields)?)?,
        issue: parse_issue(take_issue(fields)?)?,
        pages: parse_pages(take_pages(fields)?)?,
    })
}

fn parse_tracking(
    fields: &mut Map<String, Value>,
) -> Result<ReferenceTracking, ReferenceRecordError> {
    Ok(ReferenceTracking {
        resolution_state: parse_resolution_state(take_resolution_state(fields)?)?,
        identifiers: parse_identifiers(take_identifiers(fields)?)?,
        provenance: parse_provenance(take_provenance(fields)?)?,
    })
}

fn reference_fields(value: Value) -> Result<Map<String, Value>, ReferenceRecordError> {
    match value {
        Value::Object(fields) => Ok(fields),
        _ => Err(ReferenceRecordError::InvalidReferenceObject),
    }
}

fn reject_unknown_fields(
    fields: &Map<String, Value>,
    allowed_fields: &[&str],
    parent: &str,
) -> Result<(), ReferenceRecordError> {
    let unknown = fields
        .keys()
        .find(|field| !allowed_fields.contains(&field.as_str()));

    match unknown {
        Some(field) => Err(ReferenceRecordError::UnknownReferenceField {
            path: field_path(parent, field),
        }),
        None => Ok(()),
    }
}

fn field_path(parent: &str, field: &str) -> String {
    if parent.is_empty() {
        field.to_owned()
    } else {
        format!("{parent}.{field}")
    }
}

fn take_schema_version(fields: &mut Map<String, Value>) -> Result<Value, ReferenceRecordError> {
    take_required_field(
        fields,
        SCHEMA_VERSION_FIELD,
        ReferenceRecordError::MissingSchemaVersion,
    )
}

fn take_reference_id(fields: &mut Map<String, Value>) -> Result<Value, ReferenceRecordError> {
    take_required_field(
        fields,
        REFERENCE_ID_FIELD,
        ReferenceRecordError::MissingReferenceId,
    )
}

fn take_citekey(fields: &mut Map<String, Value>) -> Result<Value, ReferenceRecordError> {
    take_required_field(fields, CITEKEY_FIELD, ReferenceRecordError::MissingCitekey)
}

fn take_kind(fields: &mut Map<String, Value>) -> Result<Value, ReferenceRecordError> {
    take_required_field(fields, KIND_FIELD, ReferenceRecordError::MissingKind)
}

fn take_title(fields: &mut Map<String, Value>) -> Result<Value, ReferenceRecordError> {
    take_required_field(fields, TITLE_FIELD, ReferenceRecordError::MissingTitle)
}

fn take_contributors(fields: &mut Map<String, Value>) -> Result<Value, ReferenceRecordError> {
    take_required_field(
        fields,
        CONTRIBUTORS_FIELD,
        ReferenceRecordError::MissingContributors,
    )
}

fn take_issued(fields: &mut Map<String, Value>) -> Result<Value, ReferenceRecordError> {
    take_required_field(fields, ISSUED_FIELD, ReferenceRecordError::MissingIssued)
}

fn take_container_title(fields: &mut Map<String, Value>) -> Result<Value, ReferenceRecordError> {
    take_required_field(
        fields,
        CONTAINER_TITLE_FIELD,
        ReferenceRecordError::MissingContainerTitle,
    )
}

fn take_publisher(fields: &mut Map<String, Value>) -> Result<Value, ReferenceRecordError> {
    take_required_field(
        fields,
        PUBLISHER_FIELD,
        ReferenceRecordError::MissingPublisher,
    )
}

fn take_volume(fields: &mut Map<String, Value>) -> Result<Value, ReferenceRecordError> {
    take_required_field(fields, VOLUME_FIELD, ReferenceRecordError::MissingVolume)
}

fn take_issue(fields: &mut Map<String, Value>) -> Result<Value, ReferenceRecordError> {
    take_required_field(fields, ISSUE_FIELD, ReferenceRecordError::MissingIssue)
}

fn take_pages(fields: &mut Map<String, Value>) -> Result<Value, ReferenceRecordError> {
    take_required_field(fields, PAGES_FIELD, ReferenceRecordError::MissingPages)
}

fn take_resolution_state(fields: &mut Map<String, Value>) -> Result<Value, ReferenceRecordError> {
    take_required_field(
        fields,
        RESOLUTION_STATE_FIELD,
        ReferenceRecordError::MissingResolutionState,
    )
}

fn take_identifiers(fields: &mut Map<String, Value>) -> Result<Value, ReferenceRecordError> {
    take_required_field(
        fields,
        IDENTIFIERS_FIELD,
        ReferenceRecordError::MissingIdentifiers,
    )
}

fn take_provenance(fields: &mut Map<String, Value>) -> Result<Value, ReferenceRecordError> {
    take_required_field(
        fields,
        PROVENANCE_FIELD,
        ReferenceRecordError::MissingProvenance,
    )
}

fn take_required_field(
    fields: &mut Map<String, Value>,
    field_name: &str,
    missing_error: ReferenceRecordError,
) -> Result<Value, ReferenceRecordError> {
    fields.remove(field_name).ok_or(missing_error)
}

fn parse_schema_version(value: Value) -> Result<u64, ReferenceRecordError> {
    let version = value
        .as_u64()
        .ok_or(ReferenceRecordError::InvalidSchemaVersion)?;

    if version == REFERENCE_RECORD_SCHEMA_VERSION {
        Ok(version)
    } else {
        Err(ReferenceRecordError::UnsupportedSchemaVersion { found: version })
    }
}

fn parse_reference_id(value: Value) -> Result<ReferenceId, ReferenceRecordError> {
    let raw_id = value
        .as_str()
        .ok_or(ReferenceRecordError::InvalidReferenceId)?;
    Uuid::parse_str(raw_id)
        .map(ReferenceId)
        .map_err(|_| ReferenceRecordError::InvalidReferenceId)
}

fn parse_citekey(value: Value) -> Result<String, ReferenceRecordError> {
    let Value::String(citekey) = value else {
        return Err(ReferenceRecordError::InvalidCitekey);
    };

    if is_valid_citekey(&citekey) {
        Ok(citekey)
    } else {
        Err(ReferenceRecordError::InvalidCitekey)
    }
}

pub(crate) fn is_valid_citekey(citekey: &str) -> bool {
    let mut characters = citekey.chars();
    characters
        .next()
        .is_some_and(|first| first.is_ascii_alphanumeric())
        && characters.all(is_citekey_character)
}

fn is_citekey_character(character: char) -> bool {
    character.is_ascii_alphanumeric() || matches!(character, ':' | '_' | '-')
}

fn parse_reference_kind(value: Value) -> Result<ReferenceKind, ReferenceRecordError> {
    match value.as_str() {
        Some("article") => Ok(ReferenceKind::Article),
        Some("book") => Ok(ReferenceKind::Book),
        Some("chapter") => Ok(ReferenceKind::Chapter),
        Some("report") => Ok(ReferenceKind::Report),
        Some("thesis") => Ok(ReferenceKind::Thesis),
        Some("webpage") => Ok(ReferenceKind::Webpage),
        Some("other") => Ok(ReferenceKind::Other),
        _ => Err(ReferenceRecordError::UnsupportedReferenceKind),
    }
}

fn parse_title(value: Value) -> Result<String, ReferenceRecordError> {
    parse_non_blank_text(value).ok_or(ReferenceRecordError::InvalidTitle)
}

fn parse_contributors(value: Value) -> Result<Vec<ReferenceContributor>, ReferenceRecordError> {
    let Value::Array(contributors) = value else {
        return Err(ReferenceRecordError::InvalidContributors);
    };

    contributors
        .into_iter()
        .enumerate()
        .map(|(index, contributor)| parse_contributor(contributor, index))
        .collect()
}

fn parse_contributor(
    value: Value,
    index: usize,
) -> Result<ReferenceContributor, ReferenceRecordError> {
    let parent = format!("contributors[{index}]");
    let mut fields = contributor_fields(value, index)?;
    reject_unknown_fields(&fields, &["role", "name"], &parent)?;

    Ok(ReferenceContributor {
        role: parse_contributor_role(take_contributor_field(&mut fields, "role", index)?, index)?,
        name: parse_contributor_name(
            take_contributor_field(&mut fields, "name", index)?,
            index,
            &field_path(&parent, "name"),
        )?,
    })
}

fn contributor_fields(
    value: Value,
    index: usize,
) -> Result<Map<String, Value>, ReferenceRecordError> {
    match value {
        Value::Object(fields) => Ok(fields),
        _ => Err(ReferenceRecordError::InvalidContributor { index }),
    }
}

fn take_contributor_field(
    fields: &mut Map<String, Value>,
    field_name: &str,
    index: usize,
) -> Result<Value, ReferenceRecordError> {
    fields
        .remove(field_name)
        .ok_or(ReferenceRecordError::InvalidContributor { index })
}

fn parse_contributor_role(
    value: Value,
    index: usize,
) -> Result<ContributorRole, ReferenceRecordError> {
    match value.as_str() {
        Some("author") => Ok(ContributorRole::Author),
        Some("editor") => Ok(ContributorRole::Editor),
        Some("translator") => Ok(ContributorRole::Translator),
        _ => Err(ReferenceRecordError::InvalidContributor { index }),
    }
}

fn parse_contributor_name(
    value: Value,
    index: usize,
    parent: &str,
) -> Result<ContributorName, ReferenceRecordError> {
    let mut fields = contributor_fields(value, index)?;
    let name_type = take_contributor_field(&mut fields, "type", index)?;

    match name_type.as_str() {
        Some("person") => parse_person_name(fields, index, parent),
        Some("organization") => parse_organization_name(fields, index, parent),
        _ => Err(ReferenceRecordError::InvalidContributor { index }),
    }
}

fn parse_person_name(
    mut fields: Map<String, Value>,
    index: usize,
    parent: &str,
) -> Result<ContributorName, ReferenceRecordError> {
    reject_unknown_fields(&fields, &["given", "family"], parent)?;
    let given = parse_name_part(take_contributor_field(&mut fields, "given", index)?, index)?;
    let family = parse_name_part(take_contributor_field(&mut fields, "family", index)?, index)?;

    if given.is_none() && family.is_none() {
        return Err(ReferenceRecordError::InvalidContributor { index });
    }
    Ok(ContributorName::Person { given, family })
}

fn parse_organization_name(
    mut fields: Map<String, Value>,
    index: usize,
    parent: &str,
) -> Result<ContributorName, ReferenceRecordError> {
    reject_unknown_fields(&fields, &["literal"], parent)?;
    let literal = take_contributor_field(&mut fields, "literal", index)?;
    let literal =
        parse_non_blank_text(literal).ok_or(ReferenceRecordError::InvalidContributor { index })?;
    Ok(ContributorName::Organization { literal })
}

fn parse_name_part(value: Value, index: usize) -> Result<Option<String>, ReferenceRecordError> {
    parse_nullable_text(value).map_err(|_| ReferenceRecordError::InvalidContributor { index })
}

fn parse_issued_date(value: Value) -> Result<Option<IssuedDate>, ReferenceRecordError> {
    if value.is_null() {
        return Ok(None);
    }
    let mut fields = issued_fields(value)?;
    reject_unknown_fields(&fields, &["year", "month", "day"], ISSUED_FIELD)?;
    let year = parse_year(take_issued_field(&mut fields, "year")?)?;
    let month = parse_optional_number(take_issued_field(&mut fields, "month")?, 12)?;
    let day = parse_optional_number(take_issued_field(&mut fields, "day")?, 31)?;
    if day.is_some() && month.is_none() {
        return Err(ReferenceRecordError::InvalidIssuedDate);
    }
    Ok(Some(IssuedDate { year, month, day }))
}

fn issued_fields(value: Value) -> Result<Map<String, Value>, ReferenceRecordError> {
    match value {
        Value::Object(fields) => Ok(fields),
        _ => Err(ReferenceRecordError::InvalidIssuedDate),
    }
}

fn take_issued_field(
    fields: &mut Map<String, Value>,
    field_name: &str,
) -> Result<Value, ReferenceRecordError> {
    fields
        .remove(field_name)
        .ok_or(ReferenceRecordError::InvalidIssuedDate)
}

fn parse_year(value: Value) -> Result<u16, ReferenceRecordError> {
    match value.as_u64() {
        Some(year @ 1..=9999) => Ok(year as u16),
        _ => Err(ReferenceRecordError::InvalidIssuedDate),
    }
}

fn parse_optional_number(value: Value, maximum: u64) -> Result<Option<u8>, ReferenceRecordError> {
    match value {
        Value::Null => Ok(None),
        Value::Number(number) => match number.as_u64() {
            Some(value) if (1..=maximum).contains(&value) => Ok(Some(value as u8)),
            _ => Err(ReferenceRecordError::InvalidIssuedDate),
        },
        _ => Err(ReferenceRecordError::InvalidIssuedDate),
    }
}

fn parse_container_title(value: Value) -> Result<Option<String>, ReferenceRecordError> {
    parse_nullable_text(value).map_err(|_| ReferenceRecordError::InvalidContainerTitle)
}

fn parse_publisher(value: Value) -> Result<Option<String>, ReferenceRecordError> {
    parse_nullable_text(value).map_err(|_| ReferenceRecordError::InvalidPublisher)
}

fn parse_volume(value: Value) -> Result<Option<String>, ReferenceRecordError> {
    parse_nullable_text(value).map_err(|_| ReferenceRecordError::InvalidVolume)
}

fn parse_issue(value: Value) -> Result<Option<String>, ReferenceRecordError> {
    parse_nullable_text(value).map_err(|_| ReferenceRecordError::InvalidIssue)
}

fn parse_pages(value: Value) -> Result<Option<String>, ReferenceRecordError> {
    parse_nullable_text(value).map_err(|_| ReferenceRecordError::InvalidPages)
}

fn parse_non_blank_text(value: Value) -> Option<String> {
    match value {
        Value::String(text) if !text.trim().is_empty() => Some(text),
        _ => None,
    }
}

fn parse_nullable_text(value: Value) -> Result<Option<String>, ()> {
    match value {
        Value::Null => Ok(None),
        Value::String(text) if !text.trim().is_empty() => Ok(Some(text)),
        _ => Err(()),
    }
}

fn parse_resolution_state(value: Value) -> Result<ResolutionState, ReferenceRecordError> {
    match value.as_str() {
        Some("unresolved") => Ok(ResolutionState::Unresolved),
        Some("resolved") => Ok(ResolutionState::Resolved),
        Some("needs_review") => Ok(ResolutionState::NeedsReview),
        _ => Err(ReferenceRecordError::InvalidResolutionState),
    }
}

fn parse_identifiers(value: Value) -> Result<ReferenceIdentifiers, ReferenceRecordError> {
    let mut fields = identifier_fields(value)?;
    reject_unknown_fields(&fields, &["doi", "isbn", "url"], IDENTIFIERS_FIELD)?;
    let doi = parse_identifier_text(take_identifier_field(&mut fields, "doi")?)?;
    let isbn = parse_isbn_list(take_identifier_field(&mut fields, "isbn")?)?;
    let url = parse_identifier_text(take_identifier_field(&mut fields, "url")?)?;
    Ok(ReferenceIdentifiers { doi, isbn, url })
}

fn identifier_fields(value: Value) -> Result<Map<String, Value>, ReferenceRecordError> {
    match value {
        Value::Object(fields) => Ok(fields),
        _ => Err(ReferenceRecordError::InvalidIdentifiers),
    }
}

fn take_identifier_field(
    fields: &mut Map<String, Value>,
    field_name: &str,
) -> Result<Value, ReferenceRecordError> {
    fields
        .remove(field_name)
        .ok_or(ReferenceRecordError::InvalidIdentifiers)
}

fn parse_identifier_text(value: Value) -> Result<Option<String>, ReferenceRecordError> {
    parse_nullable_text(value).map_err(|_| ReferenceRecordError::InvalidIdentifiers)
}

fn parse_isbn_list(value: Value) -> Result<Vec<String>, ReferenceRecordError> {
    let Value::Array(entries) = value else {
        return Err(ReferenceRecordError::InvalidIdentifiers);
    };
    entries
        .into_iter()
        .enumerate()
        .map(|(index, entry)| {
            parse_non_blank_text(entry).ok_or(ReferenceRecordError::InvalidIsbn { index })
        })
        .collect()
}

fn parse_provenance(value: Value) -> Result<ReferenceProvenance, ReferenceRecordError> {
    let mut fields = provenance_fields(value)?;
    reject_unknown_fields(
        &fields,
        &["source", "source_record_id", "manual_overrides"],
        PROVENANCE_FIELD,
    )?;
    let source = parse_reference_source(take_provenance_field(&mut fields, "source")?)?;
    let source_record_id =
        parse_provenance_text(take_provenance_field(&mut fields, "source_record_id")?)?;
    let manual_overrides =
        parse_manual_overrides(take_provenance_field(&mut fields, "manual_overrides")?)?;
    validate_manual_provenance(source, source_record_id.as_ref(), &manual_overrides)?;
    Ok(ReferenceProvenance {
        source,
        source_record_id,
        manual_overrides,
    })
}

fn provenance_fields(value: Value) -> Result<Map<String, Value>, ReferenceRecordError> {
    match value {
        Value::Object(fields) => Ok(fields),
        _ => Err(ReferenceRecordError::InvalidProvenance),
    }
}

fn take_provenance_field(
    fields: &mut Map<String, Value>,
    field_name: &str,
) -> Result<Value, ReferenceRecordError> {
    fields
        .remove(field_name)
        .ok_or(ReferenceRecordError::InvalidProvenance)
}

fn parse_reference_source(value: Value) -> Result<ReferenceSource, ReferenceRecordError> {
    match value.as_str() {
        Some("manual") => Ok(ReferenceSource::Manual),
        Some("crossref") => Ok(ReferenceSource::Crossref),
        Some("semantic_scholar") => Ok(ReferenceSource::SemanticScholar),
        Some("unpaywall") => Ok(ReferenceSource::Unpaywall),
        Some("pdf_import") => Ok(ReferenceSource::PdfImport),
        _ => Err(ReferenceRecordError::InvalidProvenanceSource),
    }
}

fn parse_provenance_text(value: Value) -> Result<Option<String>, ReferenceRecordError> {
    parse_nullable_text(value).map_err(|_| ReferenceRecordError::InvalidProvenance)
}

fn parse_manual_overrides(value: Value) -> Result<Vec<ManualOverrideField>, ReferenceRecordError> {
    let Value::Array(entries) = value else {
        return Err(ReferenceRecordError::InvalidProvenance);
    };
    let mut seen = HashSet::new();
    entries
        .into_iter()
        .enumerate()
        .map(|(index, entry)| parse_manual_override(entry, index, &mut seen))
        .collect()
}

fn parse_manual_override(
    value: Value,
    index: usize,
    seen: &mut HashSet<ManualOverrideField>,
) -> Result<ManualOverrideField, ReferenceRecordError> {
    let field = match value.as_str() {
        Some("kind") => ManualOverrideField::Kind,
        Some("title") => ManualOverrideField::Title,
        Some("contributors") => ManualOverrideField::Contributors,
        Some("issued") => ManualOverrideField::Issued,
        Some("container_title") => ManualOverrideField::ContainerTitle,
        Some("publisher") => ManualOverrideField::Publisher,
        Some("volume") => ManualOverrideField::Volume,
        Some("issue") => ManualOverrideField::Issue,
        Some("pages") => ManualOverrideField::Pages,
        Some("identifiers") => ManualOverrideField::Identifiers,
        _ => return Err(ReferenceRecordError::InvalidManualOverride { index }),
    };
    if !seen.insert(field) {
        return Err(ReferenceRecordError::InvalidManualOverride { index });
    }
    Ok(field)
}

fn validate_manual_provenance(
    source: ReferenceSource,
    source_record_id: Option<&String>,
    manual_overrides: &[ManualOverrideField],
) -> Result<(), ReferenceRecordError> {
    let manual_is_consistent = source != ReferenceSource::Manual
        || (source_record_id.is_none() && manual_overrides.is_empty());
    if manual_is_consistent {
        Ok(())
    } else {
        Err(ReferenceRecordError::InvalidProvenance)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    const REFERENCE_ID: &str = "00000000-0000-4000-8000-000000000001";

    #[test]
    fn minimal_reference_deserializes() {
        let reference: ReferenceRecord = serde_json::from_value(minimal_reference())
            .expect("minimal reference should deserialize");

        assert_eq!(reference.schema_version(), REFERENCE_RECORD_SCHEMA_VERSION);
        assert_eq!(reference.reference_id().to_string(), REFERENCE_ID);
        assert_eq!(reference.citekey(), "smith2025");
    }

    #[test]
    fn reference_serialization_is_stable() {
        let reference = parse_reference(minimal_reference());

        assert_eq!(
            serde_json::to_value(reference).unwrap(),
            minimal_reference()
        );
    }

    #[test]
    fn reference_round_trip_is_stable() {
        let expected = parse_reference(full_reference());
        let serialized = serde_json::to_value(&expected).unwrap();
        let actual: ReferenceRecord = serde_json::from_value(serialized).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn person_and_organization_contributors_round_trip() {
        let mut value = full_reference();
        value[CONTRIBUTORS_FIELD] = json!([
            person_contributor(),
            {
                "role": "author",
                "name": {
                    "type": "organization",
                    "literal": "Example Research Group"
                }
            }
        ]);

        assert_round_trip(value);
    }

    #[test]
    fn partial_and_absent_issued_dates_round_trip() {
        for issued in [
            Value::Null,
            json!({ "year": 2025, "month": null, "day": null }),
            json!({ "year": 2025, "month": 7, "day": null }),
            json!({ "year": 2025, "month": 7, "day": 9 }),
        ] {
            let mut value = minimal_reference();
            value[ISSUED_FIELD] = issued;
            assert_round_trip(value);
        }
    }

    #[test]
    fn unicode_bibliographic_text_round_trip() {
        let mut value = full_reference();
        value[TITLE_FIELD] = json!("Résumé — 日本語");
        value[CONTAINER_TITLE_FIELD] = json!("Revue naïve");
        value[CONTRIBUTORS_FIELD][0]["name"]["family"] = json!("García");

        assert_round_trip(value);
    }

    #[test]
    fn supported_reference_kinds_round_trip() {
        for kind in [
            "article", "book", "chapter", "report", "thesis", "webpage", "other",
        ] {
            let mut value = minimal_reference();
            value[KIND_FIELD] = json!(kind);
            assert_round_trip(value);
        }
    }

    #[test]
    fn supported_contributor_roles_and_partial_names_round_trip() {
        for (role, given, family) in [
            ("author", Some("Ada"), None),
            ("editor", None, Some("Smith")),
            ("translator", Some("Ada"), Some("Smith")),
        ] {
            let mut value = minimal_reference();
            value[CONTRIBUTORS_FIELD] = json!([{
                "role": role,
                "name": { "type": "person", "given": given, "family": family }
            }]);
            assert_round_trip(value);
        }
    }

    #[test]
    fn supported_resolution_states_round_trip() {
        for state in ["unresolved", "resolved", "needs_review"] {
            let mut value = minimal_reference();
            value[RESOLUTION_STATE_FIELD] = json!(state);
            assert_round_trip(value);
        }
    }

    #[test]
    fn supported_provenance_sources_and_overrides_round_trip() {
        let overrides = json!([
            "kind",
            "title",
            "contributors",
            "issued",
            "container_title",
            "publisher",
            "volume",
            "issue",
            "pages",
            "identifiers"
        ]);
        for source in ["crossref", "semantic_scholar", "unpaywall", "pdf_import"] {
            let mut value = minimal_reference();
            value[PROVENANCE_FIELD] = json!({
                "source": source,
                "source_record_id": "source-record",
                "manual_overrides": overrides
            });
            assert_round_trip(value);
        }
    }

    #[test]
    fn nullable_bibliographic_fields_round_trip() {
        let mut value = minimal_reference();
        value[CONTRIBUTORS_FIELD] = json!([]);
        value[ISSUED_FIELD] = Value::Null;
        for field in [
            CONTAINER_TITLE_FIELD,
            PUBLISHER_FIELD,
            VOLUME_FIELD,
            ISSUE_FIELD,
            PAGES_FIELD,
        ] {
            value[field] = Value::Null;
        }
        value[IDENTIFIERS_FIELD] = json!({ "doi": null, "isbn": [], "url": null });

        assert_round_trip(value);
    }

    #[test]
    fn missing_required_fields_fail_predictably() {
        let missing_fields = [
            (
                SCHEMA_VERSION_FIELD,
                ReferenceRecordError::MissingSchemaVersion,
            ),
            (REFERENCE_ID_FIELD, ReferenceRecordError::MissingReferenceId),
            (CITEKEY_FIELD, ReferenceRecordError::MissingCitekey),
            (KIND_FIELD, ReferenceRecordError::MissingKind),
            (TITLE_FIELD, ReferenceRecordError::MissingTitle),
            (
                CONTRIBUTORS_FIELD,
                ReferenceRecordError::MissingContributors,
            ),
            (ISSUED_FIELD, ReferenceRecordError::MissingIssued),
            (
                CONTAINER_TITLE_FIELD,
                ReferenceRecordError::MissingContainerTitle,
            ),
            (PUBLISHER_FIELD, ReferenceRecordError::MissingPublisher),
            (VOLUME_FIELD, ReferenceRecordError::MissingVolume),
            (ISSUE_FIELD, ReferenceRecordError::MissingIssue),
            (PAGES_FIELD, ReferenceRecordError::MissingPages),
            (
                RESOLUTION_STATE_FIELD,
                ReferenceRecordError::MissingResolutionState,
            ),
            (IDENTIFIERS_FIELD, ReferenceRecordError::MissingIdentifiers),
            (PROVENANCE_FIELD, ReferenceRecordError::MissingProvenance),
        ];

        for (field, error) in missing_fields {
            assert_missing_field(field, error);
        }
    }

    #[test]
    fn non_object_reference_fails() {
        assert_eq!(
            ReferenceRecord::from_json_value(json!([])),
            Err(ReferenceRecordError::InvalidReferenceObject)
        );
    }

    #[test]
    fn unknown_top_level_and_nested_fields_fail() {
        assert_unknown_field("extra", json!(true), "extra");

        let mut identifiers = minimal_reference();
        identifiers[IDENTIFIERS_FIELD]["pmid"] = json!("123");
        assert_eq!(
            ReferenceRecord::from_json_value(identifiers),
            Err(ReferenceRecordError::UnknownReferenceField {
                path: "identifiers.pmid".to_owned()
            })
        );

        let mut contributor = minimal_reference();
        contributor[CONTRIBUTORS_FIELD][0]["name"]["suffix"] = json!("Jr.");
        assert_eq!(
            ReferenceRecord::from_json_value(contributor),
            Err(ReferenceRecordError::UnknownReferenceField {
                path: "contributors[0].name.suffix".to_owned()
            })
        );
    }

    #[test]
    fn malformed_and_unsupported_schema_versions_fail() {
        for malformed in [json!("1"), json!(1.5), json!(-1), json!(true)] {
            assert_field_error(
                SCHEMA_VERSION_FIELD,
                malformed,
                ReferenceRecordError::InvalidSchemaVersion,
            );
        }
        for version in [0, 2] {
            assert_field_error(
                SCHEMA_VERSION_FIELD,
                json!(version),
                ReferenceRecordError::UnsupportedSchemaVersion { found: version },
            );
        }
    }

    #[test]
    fn malformed_identity_and_citekey_fail() {
        for value in [json!(7), json!("not-a-uuid")] {
            assert_field_error(
                REFERENCE_ID_FIELD,
                value,
                ReferenceRecordError::InvalidReferenceId,
            );
        }
        for value in [
            json!(7),
            json!(""),
            json!(" bad"),
            json!("a b"),
            json!("éclair"),
        ] {
            assert_field_error(CITEKEY_FIELD, value, ReferenceRecordError::InvalidCitekey);
        }
    }

    #[test]
    fn unsupported_reference_kinds_fail() {
        for value in [json!(7), json!("journal_article")] {
            assert_field_error(
                KIND_FIELD,
                value,
                ReferenceRecordError::UnsupportedReferenceKind,
            );
        }
    }

    #[test]
    fn blank_titles_fail() {
        for value in [json!(7), json!(""), json!(" \n\t ")] {
            assert_field_error(TITLE_FIELD, value, ReferenceRecordError::InvalidTitle);
        }
    }

    #[test]
    fn malformed_contributors_fail() {
        assert_field_error(
            CONTRIBUTORS_FIELD,
            json!({}),
            ReferenceRecordError::InvalidContributors,
        );
        for contributor in [
            json!("author"),
            json!({ "name": person_contributor()["name"].clone() }),
            json!({ "role": "reviewer", "name": person_contributor()["name"].clone() }),
            json!({
                "role": "author",
                "name": { "type": "person", "given": null, "family": null }
            }),
            json!({
                "role": "author",
                "name": { "type": "organization", "literal": "  " }
            }),
        ] {
            assert_field_error(
                CONTRIBUTORS_FIELD,
                json!([contributor]),
                ReferenceRecordError::InvalidContributor { index: 0 },
            );
        }
    }

    #[test]
    fn malformed_issued_dates_fail() {
        for issued in [
            json!([]),
            json!({ "year": 0, "month": null, "day": null }),
            json!({ "year": 10000, "month": null, "day": null }),
            json!({ "year": 2025, "month": 13, "day": null }),
            json!({ "year": 2025, "month": null, "day": 1 }),
            json!({ "year": 2025, "month": 2, "day": 32 }),
            json!({ "year": 2025, "month": null }),
        ] {
            assert_field_error(
                ISSUED_FIELD,
                issued,
                ReferenceRecordError::InvalidIssuedDate,
            );
        }
    }

    #[test]
    fn malformed_optional_bibliographic_fields_fail() {
        let cases = [
            (
                CONTAINER_TITLE_FIELD,
                ReferenceRecordError::InvalidContainerTitle,
            ),
            (PUBLISHER_FIELD, ReferenceRecordError::InvalidPublisher),
            (VOLUME_FIELD, ReferenceRecordError::InvalidVolume),
            (ISSUE_FIELD, ReferenceRecordError::InvalidIssue),
            (PAGES_FIELD, ReferenceRecordError::InvalidPages),
        ];
        for (field, error) in cases {
            assert_field_error(field, json!("  "), error.clone());
            assert_field_error(field, json!({}), error);
        }
    }

    #[test]
    fn malformed_identifiers_fail() {
        for identifiers in [
            json!([]),
            json!({ "doi": null, "isbn": [] }),
            json!({ "doi": "  ", "isbn": [], "url": null }),
            json!({ "doi": null, "isbn": "123", "url": null }),
        ] {
            assert_field_error(
                IDENTIFIERS_FIELD,
                identifiers,
                ReferenceRecordError::InvalidIdentifiers,
            );
        }
        assert_field_error(
            IDENTIFIERS_FIELD,
            json!({ "doi": null, "isbn": ["978", "  "], "url": null }),
            ReferenceRecordError::InvalidIsbn { index: 1 },
        );
    }

    #[test]
    fn malformed_resolution_and_provenance_fail() {
        assert_field_error(
            RESOLUTION_STATE_FIELD,
            json!("verified"),
            ReferenceRecordError::InvalidResolutionState,
        );
        assert_provenance_error(
            json!({ "source": "unknown", "source_record_id": null, "manual_overrides": [] }),
            ReferenceRecordError::InvalidProvenanceSource,
        );
        assert_provenance_error(
            json!({ "source": "manual", "source_record_id": "remote", "manual_overrides": [] }),
            ReferenceRecordError::InvalidProvenance,
        );
        assert_provenance_error(
            json!({ "source": "manual", "source_record_id": null, "manual_overrides": ["title"] }),
            ReferenceRecordError::InvalidProvenance,
        );
        assert_provenance_error(
            json!({
                "source": "crossref",
                "source_record_id": "remote",
                "manual_overrides": ["title", "title"]
            }),
            ReferenceRecordError::InvalidManualOverride { index: 1 },
        );
        assert_provenance_error(
            json!({
                "source": "crossref",
                "source_record_id": "remote",
                "manual_overrides": ["citekey"]
            }),
            ReferenceRecordError::InvalidManualOverride { index: 0 },
        );
    }

    #[test]
    fn reference_failure_shape_is_stable() {
        let failures = [
            ReferenceRecordError::InvalidReferenceObject,
            ReferenceRecordError::UnknownReferenceField {
                path: "identifiers.pmid".to_owned(),
            },
            ReferenceRecordError::MissingSchemaVersion,
            ReferenceRecordError::InvalidSchemaVersion,
            ReferenceRecordError::UnsupportedSchemaVersion { found: 2 },
            ReferenceRecordError::MissingReferenceId,
            ReferenceRecordError::InvalidReferenceId,
            ReferenceRecordError::MissingCitekey,
            ReferenceRecordError::InvalidCitekey,
            ReferenceRecordError::MissingKind,
            ReferenceRecordError::UnsupportedReferenceKind,
            ReferenceRecordError::MissingTitle,
            ReferenceRecordError::InvalidTitle,
            ReferenceRecordError::MissingContributors,
            ReferenceRecordError::InvalidContributors,
            ReferenceRecordError::InvalidContributor { index: 1 },
            ReferenceRecordError::MissingIssued,
            ReferenceRecordError::InvalidIssuedDate,
            ReferenceRecordError::MissingContainerTitle,
            ReferenceRecordError::InvalidContainerTitle,
            ReferenceRecordError::MissingPublisher,
            ReferenceRecordError::InvalidPublisher,
            ReferenceRecordError::MissingVolume,
            ReferenceRecordError::InvalidVolume,
            ReferenceRecordError::MissingIssue,
            ReferenceRecordError::InvalidIssue,
            ReferenceRecordError::MissingPages,
            ReferenceRecordError::InvalidPages,
            ReferenceRecordError::MissingResolutionState,
            ReferenceRecordError::InvalidResolutionState,
            ReferenceRecordError::MissingIdentifiers,
            ReferenceRecordError::InvalidIdentifiers,
            ReferenceRecordError::InvalidIsbn { index: 2 },
            ReferenceRecordError::MissingProvenance,
            ReferenceRecordError::InvalidProvenance,
            ReferenceRecordError::InvalidProvenanceSource,
            ReferenceRecordError::InvalidManualOverride { index: 3 },
        ];
        let codes = [
            "invalid_reference_object",
            "unknown_reference_field",
            "missing_schema_version",
            "invalid_schema_version",
            "unsupported_schema_version",
            "missing_reference_id",
            "invalid_reference_id",
            "missing_citekey",
            "invalid_citekey",
            "missing_kind",
            "unsupported_reference_kind",
            "missing_title",
            "invalid_title",
            "missing_contributors",
            "invalid_contributors",
            "invalid_contributor",
            "missing_issued",
            "invalid_issued_date",
            "missing_container_title",
            "invalid_container_title",
            "missing_publisher",
            "invalid_publisher",
            "missing_volume",
            "invalid_volume",
            "missing_issue",
            "invalid_issue",
            "missing_pages",
            "invalid_pages",
            "missing_resolution_state",
            "invalid_resolution_state",
            "missing_identifiers",
            "invalid_identifiers",
            "invalid_isbn",
            "missing_provenance",
            "invalid_provenance",
            "invalid_provenance_source",
            "invalid_manual_override",
        ];

        for (failure, code) in failures.into_iter().zip(codes) {
            assert_eq!(serde_json::to_value(failure).unwrap()["code"], code);
        }

        assert_eq!(
            serde_json::to_value(ReferenceRecordError::UnknownReferenceField {
                path: "identifiers.pmid".to_owned(),
            })
            .unwrap(),
            json!({
                "code": "unknown_reference_field",
                "path": "identifiers.pmid"
            })
        );
        assert_eq!(
            serde_json::to_value(ReferenceRecordError::UnsupportedSchemaVersion { found: 2 })
                .unwrap(),
            json!({ "code": "unsupported_schema_version", "found": 2 })
        );
        assert_eq!(
            serde_json::to_value(ReferenceRecordError::InvalidContributor { index: 1 }).unwrap(),
            json!({ "code": "invalid_contributor", "index": 1 })
        );
        assert_eq!(
            serde_json::to_value(ReferenceRecordError::InvalidIsbn { index: 2 }).unwrap(),
            json!({ "code": "invalid_isbn", "index": 2 })
        );
        assert_eq!(
            serde_json::to_value(ReferenceRecordError::InvalidManualOverride { index: 3 }).unwrap(),
            json!({ "code": "invalid_manual_override", "index": 3 })
        );
    }

    fn parse_reference(value: Value) -> ReferenceRecord {
        ReferenceRecord::from_json_value(value).expect("reference should validate")
    }

    fn assert_round_trip(value: Value) {
        let actual = serde_json::to_value(parse_reference(value.clone())).unwrap();
        assert_eq!(actual, value);
    }

    fn assert_missing_field(field: &str, error: ReferenceRecordError) {
        let mut value = minimal_reference();
        value.as_object_mut().unwrap().remove(field);
        assert_eq!(ReferenceRecord::from_json_value(value), Err(error));
    }

    fn assert_field_error(field: &str, invalid: Value, error: ReferenceRecordError) {
        let mut value = minimal_reference();
        value[field] = invalid;
        assert_eq!(ReferenceRecord::from_json_value(value), Err(error));
    }

    fn assert_unknown_field(field: &str, invalid: Value, path: &str) {
        let mut value = minimal_reference();
        value[field] = invalid;
        assert_eq!(
            ReferenceRecord::from_json_value(value),
            Err(ReferenceRecordError::UnknownReferenceField {
                path: path.to_owned()
            })
        );
    }

    fn assert_provenance_error(provenance: Value, error: ReferenceRecordError) {
        assert_field_error(PROVENANCE_FIELD, provenance, error);
    }

    fn minimal_reference() -> Value {
        json!({
            "schema_version": REFERENCE_RECORD_SCHEMA_VERSION,
            "reference_id": REFERENCE_ID,
            "citekey": "smith2025",
            "kind": "article",
            "title": "A bounded reference record",
            "contributors": [person_contributor()],
            "issued": { "year": 2025, "month": null, "day": null },
            "container_title": "Journal of Examples",
            "publisher": null,
            "volume": "12",
            "issue": "3",
            "pages": "1-12",
            "resolution_state": "resolved",
            "identifiers": {
                "doi": "10.1000/example",
                "isbn": [],
                "url": null
            },
            "provenance": {
                "source": "manual",
                "source_record_id": null,
                "manual_overrides": []
            }
        })
    }

    fn full_reference() -> Value {
        let mut value = minimal_reference();
        value["kind"] = json!("chapter");
        value["publisher"] = json!("Example Press");
        value["identifiers"]["isbn"] = json!(["978-0-00-000000-0"]);
        value["identifiers"]["url"] = json!("https://example.test/reference");
        value["resolution_state"] = json!("needs_review");
        value["provenance"] = json!({
            "source": "crossref",
            "source_record_id": "10.1000/example",
            "manual_overrides": ["title", "contributors"]
        });
        value
    }

    fn person_contributor() -> Value {
        json!({
            "role": "author",
            "name": {
                "type": "person",
                "given": "Ada",
                "family": "Smith"
            }
        })
    }
}
