use super::checks::{
    FormattingFinding, FormattingFindingCode, FormattingSnapshot, FormattingTarget,
    run_formatting_checks,
};

/// Closed actions exposed for one transient formatting finding.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FormattingReviewAction {
    Inspect,
    ApplyHeadingLevel { level: u8 },
    Dismiss,
}

/// One formatting finding paired with its Rust-owned action policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FormattingReviewFinding {
    finding: FormattingFinding,
    actions: Vec<FormattingReviewAction>,
}

/// Ordered transient review output for one immutable formatting snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FormattingReview {
    findings: Vec<FormattingReviewFinding>,
}

/// Builds closed review actions without mutating or persisting the snapshot.
pub fn run_formatting_review(snapshot: &FormattingSnapshot) -> FormattingReview {
    let findings = run_formatting_checks(snapshot)
        .findings()
        .iter()
        .cloned()
        .map(|finding| review_finding(snapshot, finding))
        .collect();
    FormattingReview { findings }
}

impl FormattingReviewFinding {
    pub fn finding(&self) -> &FormattingFinding {
        &self.finding
    }

    pub fn actions(&self) -> &[FormattingReviewAction] {
        &self.actions
    }
}

impl FormattingReview {
    pub fn findings(&self) -> &[FormattingReviewFinding] {
        &self.findings
    }
}

fn review_finding(
    snapshot: &FormattingSnapshot,
    finding: FormattingFinding,
) -> FormattingReviewFinding {
    let actions = review_actions(snapshot, &finding);
    FormattingReviewFinding { finding, actions }
}

fn review_actions(
    snapshot: &FormattingSnapshot,
    finding: &FormattingFinding,
) -> Vec<FormattingReviewAction> {
    let mut actions = vec![FormattingReviewAction::Inspect];
    if let Some(level) = suggested_heading_level(snapshot, finding) {
        actions.push(FormattingReviewAction::ApplyHeadingLevel { level });
    }
    actions.push(FormattingReviewAction::Dismiss);
    actions
}

fn suggested_heading_level(
    snapshot: &FormattingSnapshot,
    finding: &FormattingFinding,
) -> Option<u8> {
    match (finding.code(), finding.target()) {
        (
            FormattingFindingCode::FirstHeadingNotLevelOne,
            FormattingTarget::Heading { index: 0 },
        ) => Some(1),
        (FormattingFindingCode::HeadingLevelSkipped, FormattingTarget::Heading { index }) => {
            preceding_heading_level(snapshot, index).map(|level| level + 1)
        }
        _ => None,
    }
}

fn preceding_heading_level(snapshot: &FormattingSnapshot, index: usize) -> Option<u8> {
    index
        .checked_sub(1)
        .and_then(|previous| snapshot.headings().get(previous))
        .map(|heading| heading.level())
}

#[cfg(test)]
#[path = "review_tests.rs"]
mod tests;
