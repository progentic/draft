use std::{collections::HashSet, error::Error, fmt};

use crate::references::record::is_valid_citekey;

/// Maximum instruction size accepted for one analysis request.
pub const MAX_AI_INSTRUCTION_BYTES: usize = 4 * 1024;

/// Maximum number of excerpts accepted from either context class.
pub const MAX_AI_EXCERPTS_PER_CLASS: usize = 64;

/// Maximum size of one complete context excerpt.
pub const MAX_AI_EXCERPT_BYTES: usize = 8 * 1024;

/// Separate retained-text budget for document and verified-evidence context.
pub const MAX_AI_CONTEXT_CLASS_BYTES: usize = 32 * 1024;

const MAX_EVIDENCE_ID_BYTES: usize = 128;
const MAX_CITEKEY_BYTES: usize = 256;

/// Closed analysis tasks accepted by the Phase 27 orchestration boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AiAnalysisTask {
    Summarize,
    EvaluateArgument,
    FactCheckSupport,
    ReviewVoiceConsistency,
    AssessSourceReliability,
}

/// One verified source excerpt with stable provenance identifiers.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AiEvidenceExcerpt {
    evidence_id: String,
    citekey: String,
    text: String,
}

/// Validated input for one bounded analysis generation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AiAnalysisRequest {
    task: AiAnalysisTask,
    instruction: String,
    document_excerpts: Vec<String>,
    evidence_excerpts: Vec<AiEvidenceExcerpt>,
}

/// Provenance-preserving block passed to a model adapter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AiContextBlock {
    UserDocument {
        text: String,
    },
    VerifiedSourceEvidence {
        evidence_id: String,
        citekey: String,
        text: String,
    },
}

/// Deterministically assembled model input with explicit omission counts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AiModelRequest {
    task: AiAnalysisTask,
    instruction: String,
    blocks: Vec<AiContextBlock>,
    retained_document_count: usize,
    omitted_document_count: usize,
    retained_evidence_count: usize,
    omitted_evidence_count: usize,
    verified_evidence_ids: Vec<String>,
}

/// Bounded failures produced before worker registration or model work.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AiRequestError {
    EmptyInstruction,
    InstructionTooLong,
    TooManyDocumentExcerpts,
    TooManyEvidenceExcerpts,
    EmptyExcerpt,
    ExcerptTooLong,
    InvalidEvidenceId,
    InvalidCitekey,
    DuplicateEvidenceId,
}

impl AiEvidenceExcerpt {
    pub fn new(
        evidence_id: impl Into<String>,
        citekey: impl Into<String>,
        text: impl Into<String>,
    ) -> Result<Self, AiRequestError> {
        Ok(Self {
            evidence_id: validated_identifier(evidence_id.into(), MAX_EVIDENCE_ID_BYTES)?,
            citekey: validated_citekey(citekey.into())?,
            text: validated_excerpt(text.into())?,
        })
    }

    pub fn evidence_id(&self) -> &str {
        &self.evidence_id
    }

    pub fn citekey(&self) -> &str {
        &self.citekey
    }

    pub fn text(&self) -> &str {
        &self.text
    }
}

impl AiAnalysisRequest {
    pub fn new(
        task: AiAnalysisTask,
        instruction: impl Into<String>,
        document_excerpts: Vec<String>,
        evidence_excerpts: Vec<AiEvidenceExcerpt>,
    ) -> Result<Self, AiRequestError> {
        require_excerpt_counts(&document_excerpts, &evidence_excerpts)?;
        let instruction = validated_instruction(instruction.into())?;
        let document_excerpts = document_excerpts
            .into_iter()
            .map(validated_excerpt)
            .collect::<Result<Vec<_>, _>>()?;
        require_unique_evidence_ids(&evidence_excerpts)?;
        Ok(Self {
            task,
            instruction,
            document_excerpts,
            evidence_excerpts,
        })
    }

    pub fn task(&self) -> AiAnalysisTask {
        self.task
    }

    pub fn instruction(&self) -> &str {
        &self.instruction
    }
}

impl AiContextBlock {
    pub fn text(&self) -> &str {
        match self {
            Self::UserDocument { text } | Self::VerifiedSourceEvidence { text, .. } => text,
        }
    }
}

impl AiModelRequest {
    pub fn task(&self) -> AiAnalysisTask {
        self.task
    }

    pub fn instruction(&self) -> &str {
        &self.instruction
    }

    pub fn blocks(&self) -> &[AiContextBlock] {
        &self.blocks
    }

    pub fn retained_document_count(&self) -> usize {
        self.retained_document_count
    }

    pub fn omitted_document_count(&self) -> usize {
        self.omitted_document_count
    }

    pub fn retained_evidence_count(&self) -> usize {
        self.retained_evidence_count
    }

    pub fn omitted_evidence_count(&self) -> usize {
        self.omitted_evidence_count
    }

    pub fn verified_evidence_ids(&self) -> &[String] {
        &self.verified_evidence_ids
    }
}

impl fmt::Display for AiRequestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::EmptyInstruction => "analysis instruction is empty",
            Self::InstructionTooLong => "analysis instruction is too long",
            Self::TooManyDocumentExcerpts => "too many document excerpts were provided",
            Self::TooManyEvidenceExcerpts => "too many evidence excerpts were provided",
            Self::EmptyExcerpt => "analysis context excerpt is empty",
            Self::ExcerptTooLong => "analysis context excerpt is too long",
            Self::InvalidEvidenceId => "analysis evidence identity is invalid",
            Self::InvalidCitekey => "analysis evidence citekey is invalid",
            Self::DuplicateEvidenceId => "analysis evidence identity is duplicated",
        })
    }
}

impl Error for AiRequestError {}

/// Assembles deterministic whole-block context under separate class budgets.
pub fn assemble_model_request(request: &AiAnalysisRequest) -> AiModelRequest {
    let (evidence_blocks, omitted_evidence_count) = retained_evidence(request);
    let retained_evidence_count = evidence_blocks.len();
    let verified_evidence_ids = retained_evidence_ids(&evidence_blocks);
    let (document_blocks, omitted_document_count) = retained_documents(request);
    let retained_document_count = document_blocks.len();
    let blocks = evidence_blocks.into_iter().chain(document_blocks).collect();
    AiModelRequest {
        task: request.task,
        instruction: request.instruction.clone(),
        blocks,
        retained_document_count,
        omitted_document_count,
        retained_evidence_count,
        omitted_evidence_count,
        verified_evidence_ids,
    }
}

fn retained_evidence(request: &AiAnalysisRequest) -> (Vec<AiContextBlock>, usize) {
    retain_whole_blocks(request.evidence_excerpts.iter().map(|evidence| {
        AiContextBlock::VerifiedSourceEvidence {
            evidence_id: evidence.evidence_id.clone(),
            citekey: evidence.citekey.clone(),
            text: evidence.text.clone(),
        }
    }))
}

fn retained_documents(request: &AiAnalysisRequest) -> (Vec<AiContextBlock>, usize) {
    retain_whole_blocks(
        request
            .document_excerpts
            .iter()
            .cloned()
            .map(|text| AiContextBlock::UserDocument { text }),
    )
}

fn retain_whole_blocks(
    blocks: impl Iterator<Item = AiContextBlock>,
) -> (Vec<AiContextBlock>, usize) {
    let mut retained = Vec::new();
    let mut omitted = 0;
    let mut used_bytes: usize = 0;
    for block in blocks {
        let Some(next_bytes) = used_bytes.checked_add(block.text().len()) else {
            omitted += 1;
            continue;
        };
        if next_bytes <= MAX_AI_CONTEXT_CLASS_BYTES {
            used_bytes = next_bytes;
            retained.push(block);
        } else {
            omitted += 1;
        }
    }
    (retained, omitted)
}

fn retained_evidence_ids(blocks: &[AiContextBlock]) -> Vec<String> {
    blocks
        .iter()
        .filter_map(|block| match block {
            AiContextBlock::VerifiedSourceEvidence { evidence_id, .. } => Some(evidence_id.clone()),
            AiContextBlock::UserDocument { .. } => None,
        })
        .collect()
}

fn require_excerpt_counts(
    documents: &[String],
    evidence: &[AiEvidenceExcerpt],
) -> Result<(), AiRequestError> {
    if documents.len() > MAX_AI_EXCERPTS_PER_CLASS {
        return Err(AiRequestError::TooManyDocumentExcerpts);
    }
    if evidence.len() > MAX_AI_EXCERPTS_PER_CLASS {
        return Err(AiRequestError::TooManyEvidenceExcerpts);
    }
    Ok(())
}

fn require_unique_evidence_ids(evidence: &[AiEvidenceExcerpt]) -> Result<(), AiRequestError> {
    let mut identities = HashSet::with_capacity(evidence.len());
    if evidence
        .iter()
        .all(|item| identities.insert(item.evidence_id()))
    {
        Ok(())
    } else {
        Err(AiRequestError::DuplicateEvidenceId)
    }
}

fn validated_instruction(value: String) -> Result<String, AiRequestError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(AiRequestError::EmptyInstruction);
    }
    if value.len() > MAX_AI_INSTRUCTION_BYTES {
        return Err(AiRequestError::InstructionTooLong);
    }
    Ok(value.to_owned())
}

fn validated_excerpt(value: String) -> Result<String, AiRequestError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(AiRequestError::EmptyExcerpt);
    }
    if value.len() > MAX_AI_EXCERPT_BYTES {
        return Err(AiRequestError::ExcerptTooLong);
    }
    Ok(value.to_owned())
}

fn validated_identifier(value: String, max_bytes: usize) -> Result<String, AiRequestError> {
    let value = value.trim();
    if value.is_empty()
        || value.len() > max_bytes
        || !value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || "-_.:".contains(character))
    {
        return Err(AiRequestError::InvalidEvidenceId);
    }
    Ok(value.to_owned())
}

fn validated_citekey(value: String) -> Result<String, AiRequestError> {
    let value = value.trim();
    if value.len() > MAX_CITEKEY_BYTES || !is_valid_citekey(value) {
        return Err(AiRequestError::InvalidCitekey);
    }
    Ok(value.to_owned())
}

#[cfg(test)]
#[path = "context_tests.rs"]
mod tests;
