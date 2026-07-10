use std::{error::Error, fmt};

use crate::references::record::is_valid_citekey;

/// Maximum headings accepted in one immutable formatting snapshot.
pub const MAX_FORMATTING_HEADINGS: usize = 512;
/// Maximum citation declarations accepted in one immutable formatting snapshot.
pub const MAX_FORMATTING_CITATIONS: usize = 512;
/// Maximum UTF-8 bytes accepted in one heading title.
pub const MAX_HEADING_TITLE_BYTES: usize = 512;
const MIN_HEADING_LEVEL: u8 = 1;
const MAX_HEADING_LEVEL: u8 = 6;

/// Closed document styles supported by the initial consistency boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FormattingStyle {
    Apa7,
    Mla9,
    Chicago17AuthorDate,
}

/// One validated heading in source order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HeadingEntry {
    level: u8,
    title: String,
}

/// One validated citation's declared style in source order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CitationStyleDeclaration {
    citekey: String,
    style: FormattingStyle,
}

/// Bounded immutable input for deterministic formatting checks.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FormattingSnapshot {
    style: FormattingStyle,
    headings: Vec<HeadingEntry>,
    citations: Vec<CitationStyleDeclaration>,
}

/// Closed review-only formatting finding codes.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum FormattingFindingCode {
    FirstHeadingNotLevelOne,
    HeadingLevelSkipped,
    CitationStyleMismatch,
}

/// Review urgency without claiming complete style-manual conformance.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FormattingSeverity {
    Warning,
    Advice,
}

/// Content-free location of one formatting finding.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum FormattingTarget {
    Heading { index: usize },
    Citation { index: usize },
}

/// One explainable non-destructive formatting finding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FormattingFinding {
    code: FormattingFindingCode,
    severity: FormattingSeverity,
    target: FormattingTarget,
    title: &'static str,
    explanation: &'static str,
}

/// Deterministic review findings for one immutable snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FormattingResult {
    findings: Vec<FormattingFinding>,
}

/// Bounded failures produced while constructing formatting inputs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FormattingInputError {
    TooManyHeadings,
    TooManyCitations,
    InvalidHeadingLevel,
    EmptyHeadingTitle,
    HeadingTitleTooLong,
    InvalidCitekey,
}

struct FindingPolicy {
    severity: FormattingSeverity,
    title: &'static str,
    explanation: &'static str,
}

/// Runs pure style-consistency and outline checks without mutating the snapshot.
pub fn run_formatting_checks(snapshot: &FormattingSnapshot) -> FormattingResult {
    let mut findings = heading_findings(snapshot.headings());
    findings.extend(citation_findings(snapshot.style(), snapshot.citations()));
    FormattingResult { findings }
}

impl FormattingStyle {
    /// Returns the stable identifier used by the formatting contract.
    pub fn identifier(self) -> &'static str {
        match self {
            Self::Apa7 => "apa7",
            Self::Mla9 => "mla9",
            Self::Chicago17AuthorDate => "chicago17_author_date",
        }
    }
}

impl HeadingEntry {
    /// Validates one heading without parsing document JSON.
    pub fn new(level: u8, title: impl Into<String>) -> Result<Self, FormattingInputError> {
        let title = title.into();
        require_valid_heading_level(level)?;
        require_valid_heading_title(&title)?;
        Ok(Self { level, title })
    }

    pub fn level(&self) -> u8 {
        self.level
    }

    pub fn title(&self) -> &str {
        &self.title
    }
}

impl CitationStyleDeclaration {
    /// Validates one citekey and its declared formatting style.
    pub fn new(
        citekey: impl Into<String>,
        style: FormattingStyle,
    ) -> Result<Self, FormattingInputError> {
        let citekey = citekey.into();
        require_valid_citekey(&citekey)?;
        Ok(Self { citekey, style })
    }

    pub fn citekey(&self) -> &str {
        &self.citekey
    }

    pub fn style(&self) -> FormattingStyle {
        self.style
    }
}

impl FormattingSnapshot {
    /// Creates one bounded snapshot from already validated domain entries.
    pub fn new(
        style: FormattingStyle,
        headings: Vec<HeadingEntry>,
        citations: Vec<CitationStyleDeclaration>,
    ) -> Result<Self, FormattingInputError> {
        require_collection_bounds(&headings, &citations)?;
        Ok(Self {
            style,
            headings,
            citations,
        })
    }

    pub fn style(&self) -> FormattingStyle {
        self.style
    }

    pub fn headings(&self) -> &[HeadingEntry] {
        &self.headings
    }

    pub fn citations(&self) -> &[CitationStyleDeclaration] {
        &self.citations
    }
}

impl FormattingFinding {
    pub fn code(&self) -> FormattingFindingCode {
        self.code
    }

    pub fn severity(&self) -> FormattingSeverity {
        self.severity
    }

    pub fn target(&self) -> FormattingTarget {
        self.target
    }

    pub fn title(&self) -> &'static str {
        self.title
    }

    pub fn explanation(&self) -> &'static str {
        self.explanation
    }
}

impl FormattingResult {
    pub fn findings(&self) -> &[FormattingFinding] {
        &self.findings
    }

    pub fn is_consistent(&self) -> bool {
        self.findings.is_empty()
    }
}

impl fmt::Display for FormattingInputError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.message())
    }
}

impl Error for FormattingInputError {}

impl FormattingInputError {
    fn message(self) -> &'static str {
        match self {
            Self::TooManyHeadings => "formatting snapshot contains too many headings",
            Self::TooManyCitations => "formatting snapshot contains too many citations",
            Self::InvalidHeadingLevel => "heading level must be between 1 and 6",
            Self::EmptyHeadingTitle => "heading title must not be blank",
            Self::HeadingTitleTooLong => "heading title is too long",
            Self::InvalidCitekey => "citation declaration contains an invalid citekey",
        }
    }
}

fn heading_findings(headings: &[HeadingEntry]) -> Vec<FormattingFinding> {
    let mut findings: Vec<FormattingFinding> =
        first_heading_finding(headings).into_iter().collect();
    findings.extend(skipped_heading_findings(headings));
    findings
}

fn first_heading_finding(headings: &[HeadingEntry]) -> Option<FormattingFinding> {
    headings
        .first()
        .filter(|heading| heading.level() != MIN_HEADING_LEVEL)
        .map(|_| {
            finding(
                FormattingFindingCode::FirstHeadingNotLevelOne,
                heading_target(0),
            )
        })
}

fn skipped_heading_findings(headings: &[HeadingEntry]) -> Vec<FormattingFinding> {
    headings
        .windows(2)
        .enumerate()
        .filter(|(_, pair)| pair[1].level() > pair[0].level() + 1)
        .map(|(index, _)| {
            finding(
                FormattingFindingCode::HeadingLevelSkipped,
                heading_target(index + 1),
            )
        })
        .collect()
}

fn citation_findings(
    selected_style: FormattingStyle,
    citations: &[CitationStyleDeclaration],
) -> Vec<FormattingFinding> {
    citations
        .iter()
        .enumerate()
        .filter(|(_, citation)| citation.style() != selected_style)
        .map(|(index, _)| {
            finding(
                FormattingFindingCode::CitationStyleMismatch,
                citation_target(index),
            )
        })
        .collect()
}

fn heading_target(index: usize) -> FormattingTarget {
    FormattingTarget::Heading { index }
}

fn citation_target(index: usize) -> FormattingTarget {
    FormattingTarget::Citation { index }
}

fn finding(code: FormattingFindingCode, target: FormattingTarget) -> FormattingFinding {
    let policy = finding_policy(code);
    FormattingFinding {
        code,
        severity: policy.severity,
        target,
        title: policy.title,
        explanation: policy.explanation,
    }
}

fn finding_policy(code: FormattingFindingCode) -> FindingPolicy {
    match code {
        FormattingFindingCode::FirstHeadingNotLevelOne => first_heading_policy(),
        FormattingFindingCode::HeadingLevelSkipped => skipped_heading_policy(),
        FormattingFindingCode::CitationStyleMismatch => citation_style_policy(),
    }
}

fn first_heading_policy() -> FindingPolicy {
    FindingPolicy {
        severity: FormattingSeverity::Advice,
        title: "Outline starts below level 1",
        explanation: "The first heading is deeper than level 1. Review whether the outline begins at its top level.",
    }
}

fn skipped_heading_policy() -> FindingPolicy {
    FindingPolicy {
        severity: FormattingSeverity::Warning,
        title: "Heading level skipped",
        explanation: "This heading is more than one level deeper than the preceding heading. Review the outline hierarchy.",
    }
}

fn citation_style_policy() -> FindingPolicy {
    FindingPolicy {
        severity: FormattingSeverity::Warning,
        title: "Citation style differs",
        explanation: "This citation declaration differs from the document's selected style. Review the style assignment.",
    }
}

fn require_valid_heading_level(level: u8) -> Result<(), FormattingInputError> {
    if (MIN_HEADING_LEVEL..=MAX_HEADING_LEVEL).contains(&level) {
        Ok(())
    } else {
        Err(FormattingInputError::InvalidHeadingLevel)
    }
}

fn require_valid_heading_title(title: &str) -> Result<(), FormattingInputError> {
    if title.trim().is_empty() {
        return Err(FormattingInputError::EmptyHeadingTitle);
    }
    if title.len() > MAX_HEADING_TITLE_BYTES {
        return Err(FormattingInputError::HeadingTitleTooLong);
    }
    Ok(())
}

fn require_valid_citekey(citekey: &str) -> Result<(), FormattingInputError> {
    if is_valid_citekey(citekey) {
        Ok(())
    } else {
        Err(FormattingInputError::InvalidCitekey)
    }
}

fn require_collection_bounds(
    headings: &[HeadingEntry],
    citations: &[CitationStyleDeclaration],
) -> Result<(), FormattingInputError> {
    if headings.len() > MAX_FORMATTING_HEADINGS {
        return Err(FormattingInputError::TooManyHeadings);
    }
    if citations.len() > MAX_FORMATTING_CITATIONS {
        return Err(FormattingInputError::TooManyCitations);
    }
    Ok(())
}

#[cfg(test)]
#[path = "checks_tests.rs"]
mod tests;
