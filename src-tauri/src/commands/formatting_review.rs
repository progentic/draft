use serde::{Deserialize, Serialize};

use crate::formatting::{
    checks::{
        CitationStyleDeclaration, FormattingFindingCode, FormattingInputError, FormattingSeverity,
        FormattingSnapshot, FormattingStyle, FormattingTarget, HeadingEntry,
    },
    review::{
        FormattingReview, FormattingReviewAction, FormattingReviewFinding,
        run_formatting_review as build_formatting_review,
    },
};

/// Bounded immutable formatting snapshot submitted by the current editor.
#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct RunFormattingReviewRequest {
    style: FormattingStyleDto,
    headings: Vec<FormattingHeadingDto>,
    citations: Vec<FormattingCitationDto>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum FormattingStyleDto {
    Apa7,
    Mla9,
    Chicago17AuthorDate,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
struct FormattingHeadingDto {
    level: u8,
    title: String,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct FormattingCitationDto {
    citekey: String,
    render_style: FormattingStyleDto,
}

/// Content-free formatting findings and their closed transient actions.
#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RunFormattingReviewResponse {
    style: FormattingStyleDto,
    findings: Vec<FormattingFindingDto>,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct FormattingFindingDto {
    code: FormattingFindingCodeDto,
    severity: FormattingSeverityDto,
    target: FormattingTargetDto,
    title: &'static str,
    explanation: &'static str,
    actions: Vec<FormattingActionDto>,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum FormattingFindingCodeDto {
    FirstHeadingNotLevelOne,
    HeadingLevelSkipped,
    CitationStyleMismatch,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum FormattingSeverityDto {
    Warning,
    Advice,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum FormattingTargetDto {
    Heading { index: usize },
    Citation { index: usize },
}

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum FormattingActionDto {
    Inspect,
    ApplyHeadingLevel { level: u8 },
    Dismiss,
}

/// Stable content-free errors exposed by `run_formatting_review`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "code", rename_all = "snake_case")]
pub(crate) enum RunFormattingReviewError {
    TooManyHeadings,
    TooManyCitations,
    InvalidHeadingLevel,
    EmptyHeadingTitle,
    HeadingTitleTooLong,
    InvalidCitekey,
}

/// Runs one bounded synchronous formatting review without document authority.
#[tauri::command]
pub(crate) fn run_formatting_review(
    request: RunFormattingReviewRequest,
) -> Result<RunFormattingReviewResponse, RunFormattingReviewError> {
    let style = request.style;
    let snapshot = request.into_snapshot()?;
    let review = build_formatting_review(&snapshot);
    Ok(RunFormattingReviewResponse::new(style, &review))
}

impl RunFormattingReviewRequest {
    fn into_snapshot(self) -> Result<FormattingSnapshot, RunFormattingReviewError> {
        FormattingSnapshot::new(
            self.style.into(),
            heading_entries(self.headings)?,
            citation_entries(self.citations)?,
        )
        .map_err(Into::into)
    }
}

impl RunFormattingReviewResponse {
    fn new(style: FormattingStyleDto, review: &FormattingReview) -> Self {
        Self {
            style,
            findings: review
                .findings()
                .iter()
                .map(FormattingFindingDto::from)
                .collect(),
        }
    }
}

impl From<FormattingReviewFinding> for FormattingFindingDto {
    fn from(item: FormattingReviewFinding) -> Self {
        Self::from(&item)
    }
}

impl From<&FormattingReviewFinding> for FormattingFindingDto {
    fn from(item: &FormattingReviewFinding) -> Self {
        let finding = item.finding();
        Self {
            code: finding.code().into(),
            severity: finding.severity().into(),
            target: finding.target().into(),
            title: finding.title(),
            explanation: finding.explanation(),
            actions: item.actions().iter().copied().map(Into::into).collect(),
        }
    }
}

impl From<FormattingStyleDto> for FormattingStyle {
    fn from(style: FormattingStyleDto) -> Self {
        match style {
            FormattingStyleDto::Apa7 => Self::Apa7,
            FormattingStyleDto::Mla9 => Self::Mla9,
            FormattingStyleDto::Chicago17AuthorDate => Self::Chicago17AuthorDate,
        }
    }
}

impl From<FormattingFindingCode> for FormattingFindingCodeDto {
    fn from(code: FormattingFindingCode) -> Self {
        match code {
            FormattingFindingCode::FirstHeadingNotLevelOne => Self::FirstHeadingNotLevelOne,
            FormattingFindingCode::HeadingLevelSkipped => Self::HeadingLevelSkipped,
            FormattingFindingCode::CitationStyleMismatch => Self::CitationStyleMismatch,
        }
    }
}

impl From<FormattingSeverity> for FormattingSeverityDto {
    fn from(severity: FormattingSeverity) -> Self {
        match severity {
            FormattingSeverity::Warning => Self::Warning,
            FormattingSeverity::Advice => Self::Advice,
        }
    }
}

impl From<FormattingTarget> for FormattingTargetDto {
    fn from(target: FormattingTarget) -> Self {
        match target {
            FormattingTarget::Heading { index } => Self::Heading { index },
            FormattingTarget::Citation { index } => Self::Citation { index },
        }
    }
}

impl From<FormattingReviewAction> for FormattingActionDto {
    fn from(action: FormattingReviewAction) -> Self {
        match action {
            FormattingReviewAction::Inspect => Self::Inspect,
            FormattingReviewAction::ApplyHeadingLevel { level } => {
                Self::ApplyHeadingLevel { level }
            }
            FormattingReviewAction::Dismiss => Self::Dismiss,
        }
    }
}

impl From<FormattingInputError> for RunFormattingReviewError {
    fn from(error: FormattingInputError) -> Self {
        match error {
            FormattingInputError::TooManyHeadings => Self::TooManyHeadings,
            FormattingInputError::TooManyCitations => Self::TooManyCitations,
            FormattingInputError::InvalidHeadingLevel => Self::InvalidHeadingLevel,
            FormattingInputError::EmptyHeadingTitle => Self::EmptyHeadingTitle,
            FormattingInputError::HeadingTitleTooLong => Self::HeadingTitleTooLong,
            FormattingInputError::InvalidCitekey => Self::InvalidCitekey,
        }
    }
}

fn heading_entries(
    headings: Vec<FormattingHeadingDto>,
) -> Result<Vec<HeadingEntry>, RunFormattingReviewError> {
    headings
        .into_iter()
        .map(|heading| HeadingEntry::new(heading.level, heading.title).map_err(Into::into))
        .collect()
}

fn citation_entries(
    citations: Vec<FormattingCitationDto>,
) -> Result<Vec<CitationStyleDeclaration>, RunFormattingReviewError> {
    citations
        .into_iter()
        .map(|citation| {
            CitationStyleDeclaration::new(citation.citekey, citation.render_style.into())
                .map_err(Into::into)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    const TYPED_COMMAND: fn(
        RunFormattingReviewRequest,
    ) -> Result<RunFormattingReviewResponse, RunFormattingReviewError> = run_formatting_review;

    #[test]
    fn command_signature_is_typed() {
        let _ = TYPED_COMMAND;
    }

    #[test]
    fn request_deserialization_is_stable() {
        let request = serde_json::from_value::<RunFormattingReviewRequest>(valid_request());
        let unknown = serde_json::from_value::<RunFormattingReviewRequest>(json!({
            "style": "apa7",
            "headings": [],
            "citations": [],
            "documentPath": "/private/document.draft"
        }));

        assert!(request.is_ok());
        assert!(unknown.is_err());
    }

    #[test]
    fn response_serialization_is_stable() {
        let request = serde_json::from_value(valid_request()).unwrap();
        let response = run_formatting_review(request).unwrap();

        assert_eq!(
            serde_json::to_value(response).unwrap(),
            json!({
                "style": "mla9",
                "findings": [
                    {
                        "code": "first_heading_not_level_one",
                        "severity": "advice",
                        "target": { "type": "heading", "index": 0 },
                        "title": "Outline starts below level 1",
                        "explanation": "The first heading is deeper than level 1. Review whether the outline begins at its top level.",
                        "actions": [
                            { "type": "inspect" },
                            { "type": "apply_heading_level", "level": 1 },
                            { "type": "dismiss" }
                        ]
                    },
                    {
                        "code": "heading_level_skipped",
                        "severity": "warning",
                        "target": { "type": "heading", "index": 1 },
                        "title": "Heading level skipped",
                        "explanation": "This heading is more than one level deeper than the preceding heading. Review the outline hierarchy.",
                        "actions": [
                            { "type": "inspect" },
                            { "type": "apply_heading_level", "level": 3 },
                            { "type": "dismiss" }
                        ]
                    },
                    {
                        "code": "citation_style_mismatch",
                        "severity": "warning",
                        "target": { "type": "citation", "index": 0 },
                        "title": "Citation style differs",
                        "explanation": "This citation declaration differs from the document's selected style. Review the style assignment.",
                        "actions": [
                            { "type": "inspect" },
                            { "type": "dismiss" }
                        ]
                    }
                ]
            })
        );
    }

    #[test]
    fn error_serialization_is_stable() {
        let errors = [
            RunFormattingReviewError::TooManyHeadings,
            RunFormattingReviewError::TooManyCitations,
            RunFormattingReviewError::InvalidHeadingLevel,
            RunFormattingReviewError::EmptyHeadingTitle,
            RunFormattingReviewError::HeadingTitleTooLong,
            RunFormattingReviewError::InvalidCitekey,
        ];

        assert_eq!(
            serde_json::to_value(errors).unwrap(),
            json!([
                { "code": "too_many_headings" },
                { "code": "too_many_citations" },
                { "code": "invalid_heading_level" },
                { "code": "empty_heading_title" },
                { "code": "heading_title_too_long" },
                { "code": "invalid_citekey" }
            ])
        );
    }

    #[test]
    fn rejected_content_is_not_returned_in_errors() {
        let request = serde_json::from_value::<RunFormattingReviewRequest>(json!({
            "style": "apa7",
            "headings": [],
            "citations": [{
                "citekey": "sensitive invalid key",
                "renderStyle": "apa7"
            }]
        }))
        .unwrap();

        let error = run_formatting_review(request).unwrap_err();
        assert_eq!(error, RunFormattingReviewError::InvalidCitekey);
        assert!(!format!("{error:?}").contains("sensitive"));
    }

    fn valid_request() -> serde_json::Value {
        json!({
            "style": "mla9",
            "headings": [
                { "level": 2, "title": "Start" },
                { "level": 4, "title": "Detail" }
            ],
            "citations": [{
                "citekey": "smith2025",
                "renderStyle": "apa7"
            }]
        })
    }
}
