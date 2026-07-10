use serde::Deserialize;

use super::protocol::{MAX_TEXT_ANALYSIS_TEXT_BYTES, PythonHelperLocale, PythonHelperRequestError};

/// Maximum findings accepted from one deterministic helper response.
pub const MAX_TEXT_ANALYSIS_FINDINGS: usize = 100;

/// Validated immutable text snapshot submitted for deterministic checks.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextAnalysisInput {
    pub(crate) text: String,
    pub(crate) locale: PythonHelperLocale,
}

/// Closed deterministic issue codes returned by the helper.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
#[serde(rename_all = "snake_case")]
pub enum TextAnalysisFindingCode {
    RepeatedWord,
    LongSentence,
    AllCapsEmphasis,
    RepeatedSentenceOpener,
    MixedFirstPerson,
}

/// User-facing review category owned by Rust rather than Python prose.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TextAnalysisCategory {
    Grammar,
    Clarity,
    Tone,
    Cohesion,
    Voice,
}

/// Review urgency without claiming that a heuristic proves an error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TextAnalysisSeverity {
    Warning,
    Advice,
}

/// One explainable non-destructive review finding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextAnalysisFinding {
    code: TextAnalysisFindingCode,
    category: TextAnalysisCategory,
    severity: TextAnalysisSeverity,
    start_byte: usize,
    end_byte: usize,
    title: &'static str,
    explanation: &'static str,
}

/// Bounded deterministic findings for one immutable input snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextAnalysisResult {
    findings: Vec<TextAnalysisFinding>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(crate) struct TextAnalysisWireResult {
    findings: Vec<TextAnalysisWireFinding>,
}

#[derive(Clone, Copy, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct TextAnalysisWireFinding {
    code: TextAnalysisFindingCode,
    start_byte: usize,
    end_byte: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct InvalidTextAnalysisResult;

struct FindingPolicy {
    category: TextAnalysisCategory,
    severity: TextAnalysisSeverity,
    title: &'static str,
    explanation: &'static str,
}

impl TextAnalysisInput {
    pub fn new(text: impl Into<String>) -> Result<Self, PythonHelperRequestError> {
        let text = text.into();
        if text.trim().is_empty() {
            return Err(PythonHelperRequestError::EmptyText);
        }
        if text.len() > MAX_TEXT_ANALYSIS_TEXT_BYTES {
            return Err(PythonHelperRequestError::TextTooLong);
        }
        Ok(Self {
            text,
            locale: PythonHelperLocale::EnUs,
        })
    }

    pub fn text(&self) -> &str {
        &self.text
    }
}

impl TextAnalysisFinding {
    pub fn code(&self) -> TextAnalysisFindingCode {
        self.code
    }

    pub fn category(&self) -> TextAnalysisCategory {
        self.category
    }

    pub fn severity(&self) -> TextAnalysisSeverity {
        self.severity
    }

    pub fn start_byte(&self) -> usize {
        self.start_byte
    }

    pub fn end_byte(&self) -> usize {
        self.end_byte
    }

    pub fn title(&self) -> &str {
        self.title
    }

    pub fn explanation(&self) -> &str {
        self.explanation
    }
}

impl TextAnalysisResult {
    pub fn findings(&self) -> &[TextAnalysisFinding] {
        &self.findings
    }
}

pub(crate) fn validate_text_analysis_result(
    text: &str,
    result: TextAnalysisWireResult,
) -> Result<TextAnalysisResult, InvalidTextAnalysisResult> {
    if result.findings.len() > MAX_TEXT_ANALYSIS_FINDINGS {
        return Err(InvalidTextAnalysisResult);
    }
    require_strict_finding_order(&result.findings)?;
    let findings = result
        .findings
        .into_iter()
        .map(|finding| validated_finding(text, finding))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(TextAnalysisResult { findings })
}

fn require_strict_finding_order(
    findings: &[TextAnalysisWireFinding],
) -> Result<(), InvalidTextAnalysisResult> {
    let mut previous = None;
    for finding in findings {
        let current = (
            finding.start_byte,
            finding.end_byte,
            finding_code_order(finding.code),
        );
        if previous.is_some_and(|previous| previous >= current) {
            return Err(InvalidTextAnalysisResult);
        }
        previous = Some(current);
    }
    Ok(())
}

fn finding_code_order(code: TextAnalysisFindingCode) -> u8 {
    match code {
        TextAnalysisFindingCode::AllCapsEmphasis => 0,
        TextAnalysisFindingCode::LongSentence => 1,
        TextAnalysisFindingCode::MixedFirstPerson => 2,
        TextAnalysisFindingCode::RepeatedSentenceOpener => 3,
        TextAnalysisFindingCode::RepeatedWord => 4,
    }
}

fn validated_finding(
    text: &str,
    finding: TextAnalysisWireFinding,
) -> Result<TextAnalysisFinding, InvalidTextAnalysisResult> {
    require_valid_range(text, finding.start_byte, finding.end_byte)?;
    let policy = finding_policy(finding.code);
    Ok(TextAnalysisFinding {
        code: finding.code,
        category: policy.category,
        severity: policy.severity,
        start_byte: finding.start_byte,
        end_byte: finding.end_byte,
        title: policy.title,
        explanation: policy.explanation,
    })
}

fn require_valid_range(
    text: &str,
    start_byte: usize,
    end_byte: usize,
) -> Result<(), InvalidTextAnalysisResult> {
    if start_byte < end_byte
        && end_byte <= text.len()
        && text.is_char_boundary(start_byte)
        && text.is_char_boundary(end_byte)
    {
        Ok(())
    } else {
        Err(InvalidTextAnalysisResult)
    }
}

fn finding_policy(code: TextAnalysisFindingCode) -> FindingPolicy {
    match code {
        TextAnalysisFindingCode::RepeatedWord => FindingPolicy {
            category: TextAnalysisCategory::Grammar,
            severity: TextAnalysisSeverity::Warning,
            title: "Repeated word",
            explanation: "An adjacent word may have been duplicated. Review the repetition before editing.",
        },
        TextAnalysisFindingCode::LongSentence => FindingPolicy {
            category: TextAnalysisCategory::Clarity,
            severity: TextAnalysisSeverity::Advice,
            title: "Long sentence",
            explanation: "This sentence contains more than 30 words. Consider whether smaller parts would be easier to follow.",
        },
        TextAnalysisFindingCode::AllCapsEmphasis => FindingPolicy {
            category: TextAnalysisCategory::Tone,
            severity: TextAnalysisSeverity::Advice,
            title: "Extended capital emphasis",
            explanation: "A word of five or more letters uses all capitals and may read as unusually forceful.",
        },
        TextAnalysisFindingCode::RepeatedSentenceOpener => FindingPolicy {
            category: TextAnalysisCategory::Cohesion,
            severity: TextAnalysisSeverity::Advice,
            title: "Repeated sentence opening",
            explanation: "Consecutive sentences begin with the same substantial word and may feel repetitive.",
        },
        TextAnalysisFindingCode::MixedFirstPerson => FindingPolicy {
            category: TextAnalysisCategory::Voice,
            severity: TextAnalysisSeverity::Advice,
            title: "First-person perspective shift",
            explanation: "Both singular and plural first-person pronouns appear. Review whether the perspective shift is intentional.",
        },
    }
}

#[cfg(test)]
#[path = "text_analysis_tests.rs"]
mod tests;
