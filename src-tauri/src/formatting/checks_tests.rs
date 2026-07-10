use super::*;

const ALL_STYLES: [FormattingStyle; 3] = [
    FormattingStyle::Apa7,
    FormattingStyle::Mla9,
    FormattingStyle::Chicago17AuthorDate,
];

#[test]
fn style_identifiers_are_stable_and_closed() {
    assert_eq!(
        ALL_STYLES.map(FormattingStyle::identifier),
        ["apa7", "mla9", "chicago17_author_date"]
    );
}

#[test]
fn matching_style_and_valid_outline_are_consistent() {
    for style in ALL_STYLES {
        let snapshot = snapshot(
            style,
            vec![heading(1, "Introduction"), heading(2, "Evidence")],
            vec![citation("smith2025", style)],
        );

        assert!(run_formatting_checks(&snapshot).is_consistent());
    }
}

#[test]
fn first_heading_below_level_one_is_reviewable() {
    let snapshot = snapshot(FormattingStyle::Apa7, vec![heading(2, "Methods")], vec![]);

    assert_eq!(
        finding_pairs(&snapshot),
        [(
            FormattingFindingCode::FirstHeadingNotLevelOne,
            FormattingTarget::Heading { index: 0 }
        )]
    );
}

#[test]
fn skipped_heading_levels_are_reported_in_source_order() {
    let headings = vec![
        heading(1, "One"),
        heading(3, "Three"),
        heading(2, "Two"),
        heading(4, "Four"),
    ];
    let snapshot = snapshot(FormattingStyle::Mla9, headings, vec![]);

    assert_eq!(
        finding_pairs(&snapshot),
        [
            (
                FormattingFindingCode::HeadingLevelSkipped,
                FormattingTarget::Heading { index: 1 }
            ),
            (
                FormattingFindingCode::HeadingLevelSkipped,
                FormattingTarget::Heading { index: 3 }
            )
        ]
    );
}

#[test]
fn siblings_and_ancestor_transitions_do_not_create_findings() {
    let headings = vec![
        heading(1, "One"),
        heading(2, "Two"),
        heading(2, "Two again"),
        heading(1, "One again"),
        heading(2, "Final two"),
    ];

    assert!(
        run_formatting_checks(&snapshot(FormattingStyle::Apa7, headings, vec![])).is_consistent()
    );
}

#[test]
fn citation_style_mismatches_are_reported_for_every_selected_style() {
    for (index, style) in ALL_STYLES.into_iter().enumerate() {
        let other_style = ALL_STYLES[(index + 1) % ALL_STYLES.len()];
        let snapshot = snapshot(
            style,
            vec![],
            vec![
                citation("matching2025", style),
                citation("mismatch2025", other_style),
            ],
        );

        assert_eq!(
            finding_pairs(&snapshot),
            [(
                FormattingFindingCode::CitationStyleMismatch,
                FormattingTarget::Citation { index: 1 }
            )]
        );
    }
}

#[test]
fn heading_and_citation_findings_have_deterministic_target_order() {
    let snapshot = snapshot(
        FormattingStyle::Chicago17AuthorDate,
        vec![heading(2, "Start"), heading(4, "Deep")],
        vec![citation("smith2025", FormattingStyle::Apa7)],
    );
    let first = run_formatting_checks(&snapshot);
    let second = run_formatting_checks(&snapshot);

    assert_eq!(first, second);
    assert_eq!(
        first
            .findings()
            .iter()
            .map(FormattingFinding::target)
            .collect::<Vec<_>>(),
        [
            FormattingTarget::Heading { index: 0 },
            FormattingTarget::Heading { index: 1 },
            FormattingTarget::Citation { index: 0 }
        ]
    );
}

#[test]
fn heading_validation_enforces_level_title_and_utf8_byte_bounds() {
    for level in [0, 7] {
        assert_eq!(
            HeadingEntry::new(level, "Title"),
            Err(FormattingInputError::InvalidHeadingLevel)
        );
    }
    assert_eq!(
        HeadingEntry::new(1, " \n "),
        Err(FormattingInputError::EmptyHeadingTitle)
    );
    assert_eq!(heading(6, &"é".repeat(256)).title().len(), 512);
    assert_eq!(
        HeadingEntry::new(1, format!("{}a", "é".repeat(256))),
        Err(FormattingInputError::HeadingTitleTooLong)
    );
}

#[test]
fn citekey_validation_reuses_the_reference_domain_rule() {
    let declaration = citation("Smith:2025_review", FormattingStyle::Apa7);
    assert_eq!(declaration.citekey(), "Smith:2025_review");
    assert_eq!(
        CitationStyleDeclaration::new("invalid key", FormattingStyle::Apa7),
        Err(FormattingInputError::InvalidCitekey)
    );
}

#[test]
fn snapshot_collection_bounds_fail_before_checks_run() {
    let headings = vec![heading(1, "Heading"); MAX_FORMATTING_HEADINGS + 1];
    assert_eq!(
        FormattingSnapshot::new(FormattingStyle::Apa7, headings, vec![]),
        Err(FormattingInputError::TooManyHeadings)
    );

    let citations =
        vec![citation("smith2025", FormattingStyle::Apa7); MAX_FORMATTING_CITATIONS + 1];
    assert_eq!(
        FormattingSnapshot::new(FormattingStyle::Apa7, vec![], citations),
        Err(FormattingInputError::TooManyCitations)
    );
}

#[test]
fn finding_policy_is_fixed_review_only_and_content_free() {
    let source_title = "Sensitive heading text";
    let source_citekey = "private2025";
    let snapshot = snapshot(
        FormattingStyle::Apa7,
        vec![heading(3, source_title)],
        vec![citation(source_citekey, FormattingStyle::Mla9)],
    );
    let result = run_formatting_checks(&snapshot);

    assert!(result.findings().iter().all(|finding| {
        !finding.title().is_empty()
            && !finding.explanation().contains(source_title)
            && !finding.explanation().contains(source_citekey)
    }));
    assert_eq!(result.findings()[0].severity(), FormattingSeverity::Advice);
    assert_eq!(result.findings()[1].severity(), FormattingSeverity::Warning);
}

#[test]
fn input_errors_are_bounded_and_do_not_include_rejected_content() {
    let rejected = "sensitive key";
    let error = CitationStyleDeclaration::new(rejected, FormattingStyle::Apa7).unwrap_err();

    assert_eq!(
        error.to_string(),
        "citation declaration contains an invalid citekey"
    );
    assert!(!error.to_string().contains(rejected));
}

fn finding_pairs(snapshot: &FormattingSnapshot) -> Vec<(FormattingFindingCode, FormattingTarget)> {
    run_formatting_checks(snapshot)
        .findings()
        .iter()
        .map(|finding| (finding.code(), finding.target()))
        .collect()
}

fn snapshot(
    style: FormattingStyle,
    headings: Vec<HeadingEntry>,
    citations: Vec<CitationStyleDeclaration>,
) -> FormattingSnapshot {
    FormattingSnapshot::new(style, headings, citations).unwrap()
}

fn heading(level: u8, title: &str) -> HeadingEntry {
    HeadingEntry::new(level, title).unwrap()
}

fn citation(citekey: &str, style: FormattingStyle) -> CitationStyleDeclaration {
    CitationStyleDeclaration::new(citekey, style).unwrap()
}
