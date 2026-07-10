use super::*;

#[test]
fn request_validates_and_normalizes_bounded_input() {
    let request = AiAnalysisRequest::new(
        AiAnalysisTask::Summarize,
        "  Summarize this argument.  ",
        vec!["  Document excerpt.  ".to_owned()],
        vec![evidence("source-1", "smith2025", "  Evidence excerpt.  ")],
    )
    .unwrap();

    assert_eq!(request.instruction(), "Summarize this argument.");
    let model = assemble_model_request(&request);
    assert_eq!(model.task(), AiAnalysisTask::Summarize);
    assert_eq!(model.retained_document_count(), 1);
    assert_eq!(model.retained_evidence_count(), 1);
    assert_eq!(model.verified_evidence_ids(), ["source-1"]);
}

#[test]
fn request_rejects_instruction_and_excerpt_bounds() {
    assert_eq!(
        AiAnalysisRequest::new(AiAnalysisTask::Summarize, " ", vec![], vec![]),
        Err(AiRequestError::EmptyInstruction)
    );
    assert_eq!(
        AiAnalysisRequest::new(
            AiAnalysisTask::Summarize,
            "x".repeat(MAX_AI_INSTRUCTION_BYTES + 1),
            vec![],
            vec![],
        ),
        Err(AiRequestError::InstructionTooLong)
    );
    assert_eq!(
        AiAnalysisRequest::new(
            AiAnalysisTask::Summarize,
            "valid",
            vec!["x".repeat(MAX_AI_EXCERPT_BYTES + 1)],
            vec![],
        ),
        Err(AiRequestError::ExcerptTooLong)
    );
}

#[test]
fn request_rejects_count_and_duplicate_evidence_bounds() {
    let documents = vec!["text".to_owned(); MAX_AI_EXCERPTS_PER_CLASS + 1];
    assert_eq!(
        AiAnalysisRequest::new(AiAnalysisTask::Summarize, "valid", documents, vec![]),
        Err(AiRequestError::TooManyDocumentExcerpts)
    );

    let too_much_evidence = (0..=MAX_AI_EXCERPTS_PER_CLASS)
        .map(|index| evidence(&format!("source-{index}"), "source2025", "text"))
        .collect();
    assert_eq!(
        AiAnalysisRequest::new(
            AiAnalysisTask::Summarize,
            "valid",
            vec![],
            too_much_evidence,
        ),
        Err(AiRequestError::TooManyEvidenceExcerpts)
    );

    let duplicated = vec![
        evidence("same", "first2025", "one"),
        evidence("same", "second2025", "two"),
    ];
    assert_eq!(
        AiAnalysisRequest::new(AiAnalysisTask::Summarize, "valid", vec![], duplicated),
        Err(AiRequestError::DuplicateEvidenceId)
    );
}

#[test]
fn evidence_identity_and_citekey_fail_closed() {
    assert_eq!(
        AiEvidenceExcerpt::new("contains space", "smith2025", "text"),
        Err(AiRequestError::InvalidEvidenceId)
    );
    assert_eq!(
        AiEvidenceExcerpt::new("source", " ", "text"),
        Err(AiRequestError::InvalidCitekey)
    );
    assert_eq!(
        AiEvidenceExcerpt::new("source", "contains space", "text"),
        Err(AiRequestError::InvalidCitekey)
    );
}

#[test]
fn context_preserves_provenance_and_class_order() {
    let request = AiAnalysisRequest::new(
        AiAnalysisTask::FactCheckSupport,
        "check",
        vec!["document one".to_owned(), "document two".to_owned()],
        vec![
            evidence("evidence-1", "first2025", "evidence one"),
            evidence("evidence-2", "second2025", "evidence two"),
        ],
    )
    .unwrap();

    let model = assemble_model_request(&request);

    assert!(matches!(
        &model.blocks()[0],
        AiContextBlock::VerifiedSourceEvidence { evidence_id, citekey, .. }
            if evidence_id == "evidence-1" && citekey == "first2025"
    ));
    assert!(matches!(
        &model.blocks()[2],
        AiContextBlock::UserDocument { text } if text == "document one"
    ));
}

#[test]
fn context_omits_whole_blocks_deterministically() {
    let block = "x".repeat(MAX_AI_EXCERPT_BYTES);
    let request = AiAnalysisRequest::new(
        AiAnalysisTask::EvaluateArgument,
        "evaluate",
        vec![block.clone(); 6],
        (0..6)
            .map(|index| evidence(&format!("source-{index}"), "smith2025", &block))
            .collect(),
    )
    .unwrap();

    let first = assemble_model_request(&request);
    let second = assemble_model_request(&request);

    assert_eq!(first, second);
    assert_eq!(first.retained_document_count(), 4);
    assert_eq!(first.omitted_document_count(), 2);
    assert_eq!(first.retained_evidence_count(), 4);
    assert_eq!(first.omitted_evidence_count(), 2);
    assert!(
        first
            .blocks()
            .iter()
            .all(|context| context.text().len() == MAX_AI_EXCERPT_BYTES)
    );
}

#[test]
fn request_errors_do_not_include_user_content() {
    assert_eq!(
        AiRequestError::EmptyInstruction.to_string(),
        "analysis instruction is empty"
    );
    assert_eq!(
        AiRequestError::DuplicateEvidenceId.to_string(),
        "analysis evidence identity is duplicated"
    );
}

fn evidence(id: &str, citekey: &str, text: &str) -> AiEvidenceExcerpt {
    AiEvidenceExcerpt::new(id, citekey, text).unwrap()
}
