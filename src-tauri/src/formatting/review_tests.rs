use super::*;
use crate::formatting::checks::{CitationStyleDeclaration, FormattingStyle, HeadingEntry};

#[test]
fn heading_findings_receive_bounded_apply_levels() {
    let snapshot = FormattingSnapshot::new(
        FormattingStyle::Apa7,
        vec![heading(2, "Start"), heading(4, "Detail")],
        vec![],
    )
    .unwrap();
    let review = run_formatting_review(&snapshot);

    assert_eq!(
        review.findings()[0].actions(),
        [
            FormattingReviewAction::Inspect,
            FormattingReviewAction::ApplyHeadingLevel { level: 1 },
            FormattingReviewAction::Dismiss,
        ]
    );
    assert_eq!(
        review.findings()[1].actions(),
        [
            FormattingReviewAction::Inspect,
            FormattingReviewAction::ApplyHeadingLevel { level: 3 },
            FormattingReviewAction::Dismiss,
        ]
    );
}

#[test]
fn citation_findings_remain_inspect_and_dismiss_only() {
    let snapshot = FormattingSnapshot::new(
        FormattingStyle::Mla9,
        vec![],
        vec![CitationStyleDeclaration::new("smith2025", FormattingStyle::Apa7).unwrap()],
    )
    .unwrap();
    let review = run_formatting_review(&snapshot);

    assert_eq!(
        review.findings()[0].actions(),
        [
            FormattingReviewAction::Inspect,
            FormattingReviewAction::Dismiss,
        ]
    );
}

#[test]
fn consistent_snapshots_return_an_empty_review() {
    let snapshot = FormattingSnapshot::new(
        FormattingStyle::Chicago17AuthorDate,
        vec![heading(1, "Start"), heading(2, "Detail")],
        vec![],
    )
    .unwrap();

    assert!(run_formatting_review(&snapshot).findings().is_empty());
}

#[test]
fn review_preserves_domain_finding_order_and_wording() {
    let snapshot = FormattingSnapshot::new(
        FormattingStyle::Mla9,
        vec![heading(2, "Private heading")],
        vec![CitationStyleDeclaration::new("private2025", FormattingStyle::Apa7).unwrap()],
    )
    .unwrap();
    let review = run_formatting_review(&snapshot);

    assert_eq!(review.findings().len(), 2);
    assert_eq!(
        review.findings()[0].finding().code(),
        FormattingFindingCode::FirstHeadingNotLevelOne
    );
    assert_eq!(
        review.findings()[1].finding().code(),
        FormattingFindingCode::CitationStyleMismatch
    );
    assert!(review.findings().iter().all(|item| {
        !item.finding().explanation().contains("Private heading")
            && !item.finding().explanation().contains("private2025")
    }));
}

fn heading(level: u8, title: &str) -> HeadingEntry {
    HeadingEntry::new(level, title).unwrap()
}
