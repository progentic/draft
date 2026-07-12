use super::*;

#[test]
fn input_accepts_exact_size_limit_and_rejects_one_byte_more() {
    let exact = "x".repeat(MAX_TEXT_ANALYSIS_TEXT_BYTES);
    let oversized = "x".repeat(MAX_TEXT_ANALYSIS_TEXT_BYTES + 1);

    assert_eq!(
        TextAnalysisInput::new(exact).unwrap().text().len(),
        MAX_TEXT_ANALYSIS_TEXT_BYTES
    );
    assert_eq!(
        TextAnalysisInput::new(oversized),
        Err(PythonHelperRequestError::TextTooLong)
    );
    assert_eq!(
        TextAnalysisInput::new(" \n\t"),
        Err(PythonHelperRequestError::EmptyText)
    );
}

#[test]
fn finding_codes_map_to_fixed_explainable_policies() {
    let text = "alpha beta gamma delta epsilon";
    let codes = [
        TextAnalysisFindingCode::RepeatedWord,
        TextAnalysisFindingCode::LongSentence,
        TextAnalysisFindingCode::AllCapsEmphasis,
        TextAnalysisFindingCode::RepeatedSentenceOpener,
        TextAnalysisFindingCode::MixedFirstPerson,
    ];
    let findings = codes
        .into_iter()
        .enumerate()
        .map(|(index, code)| TextAnalysisWireFinding {
            code,
            start_byte: index * 6,
            end_byte: index * 6 + 5,
        })
        .collect();

    let result = validate_text_analysis_result(text, TextAnalysisWireResult { findings }).unwrap();

    assert_eq!(result.findings().len(), 5);
    assert_eq!(
        result.findings()[0].category(),
        TextAnalysisCategory::Grammar
    );
    assert_eq!(
        result.findings()[0].severity(),
        TextAnalysisSeverity::Warning
    );
    assert_eq!(
        result.findings()[1].category(),
        TextAnalysisCategory::Clarity
    );
    assert_eq!(result.findings()[2].category(), TextAnalysisCategory::Tone);
    assert_eq!(
        result.findings()[3].category(),
        TextAnalysisCategory::Cohesion
    );
    assert_eq!(result.findings()[4].category(), TextAnalysisCategory::Voice);
    assert!(
        result.findings().iter().all(|finding| {
            !finding.title().is_empty() && !finding.explanation().contains(text)
        })
    );
}

#[test]
fn unicode_ranges_must_use_utf8_character_boundaries() {
    let text = "Café café";
    let valid = result(vec![wire(TextAnalysisFindingCode::RepeatedWord, 6, 11)]);
    let finding = validate_text_analysis_result(text, valid).unwrap();
    assert_eq!(finding.findings()[0].start_byte(), 6);
    assert_eq!(finding.findings()[0].end_byte(), 11);

    for (start, end) in [(4, 5), (6, 10)] {
        assert_eq!(
            validate_text_analysis_result(
                text,
                result(vec![wire(
                    TextAnalysisFindingCode::RepeatedWord,
                    start,
                    end
                )]),
            ),
            Err(InvalidTextAnalysisResult)
        );
    }
}

#[test]
fn empty_reversed_and_out_of_bounds_ranges_fail_closed() {
    let text = "review";
    for (start, end) in [(1, 1), (4, 2), (0, 7)] {
        assert_eq!(
            validate_text_analysis_result(
                text,
                result(vec![wire(
                    TextAnalysisFindingCode::LongSentence,
                    start,
                    end
                )]),
            ),
            Err(InvalidTextAnalysisResult)
        );
    }
}

#[test]
fn duplicate_and_unsorted_findings_fail_closed() {
    let text = "alpha beta";
    let first = wire(TextAnalysisFindingCode::RepeatedWord, 0, 5);
    let second = wire(TextAnalysisFindingCode::LongSentence, 6, 10);
    assert_eq!(
        validate_text_analysis_result(text, result(vec![first, first])),
        Err(InvalidTextAnalysisResult)
    );
    assert_eq!(
        validate_text_analysis_result(text, result(vec![second, first])),
        Err(InvalidTextAnalysisResult)
    );
}

#[test]
fn equal_ranges_follow_lexical_wire_code_order() {
    let text = "LOUDER";
    let findings = vec![
        wire(TextAnalysisFindingCode::AllCapsEmphasis, 0, 6),
        wire(TextAnalysisFindingCode::RepeatedWord, 0, 6),
    ];

    assert_eq!(
        validate_text_analysis_result(text, result(findings))
            .unwrap()
            .findings()
            .len(),
        2
    );
}

#[test]
fn excessive_finding_count_fails_closed() {
    let findings = (0..=MAX_TEXT_ANALYSIS_FINDINGS)
        .map(|index| wire(TextAnalysisFindingCode::LongSentence, index, index + 1))
        .collect();
    assert_eq!(
        validate_text_analysis_result(
            &"x".repeat(MAX_TEXT_ANALYSIS_FINDINGS + 1),
            result(findings)
        ),
        Err(InvalidTextAnalysisResult)
    );
}

#[test]
fn finding_model_has_no_replacement_or_source_text_field() {
    let result = validate_text_analysis_result(
        "word word",
        result(vec![wire(TextAnalysisFindingCode::RepeatedWord, 5, 9)]),
    )
    .unwrap();
    let finding = &result.findings()[0];

    assert_eq!(finding.code(), TextAnalysisFindingCode::RepeatedWord);
    assert_eq!(finding.title(), "Repeated word");
    assert!(!finding.explanation().contains("word word"));
}

fn result(findings: Vec<TextAnalysisWireFinding>) -> TextAnalysisWireResult {
    TextAnalysisWireResult { findings }
}

fn wire(
    code: TextAnalysisFindingCode,
    start_byte: usize,
    end_byte: usize,
) -> TextAnalysisWireFinding {
    TextAnalysisWireFinding {
        code,
        start_byte,
        end_byte,
    }
}
