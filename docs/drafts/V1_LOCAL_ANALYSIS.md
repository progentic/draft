# V1 Local Analysis Contract

**Status:** Accepted implementation contract
**Decision dependency:** Accepted ADR-002
**Target phase:** Phase 46

## Purpose

Define the bounded evidence required to expose the existing deterministic text
analysis in DRAFT v1.0.0 without adding a model provider or changing trusted
ownership.

## Required Behavior

- Rust starts the existing allowlisted `text_analysis` helper for one immutable
  editor snapshot.
- The packaged application resolves only its trusted helper entrypoint and does
  not accept a user-supplied executable, module, path, or command.
- Identical text, locale, protocol version, and helper version produce identical
  ordered findings. Thresholds are explicit, locale-sensitive behavior is
  controlled, and map or set iteration cannot affect output order.
- The visible workflow names grammar, clarity, tone, cohesion, and voice review
  and identifies the five supported checks without claiming comprehensive
  correctness.
- Findings remain review-only. They may focus a current passage but cannot
  persist, replace text, or apply an edit.
- The workflow exposes typed, non-sensitive failures through the existing error
  presentation policy.
- Empty, malformed, unsupported-version, oversized, timeout, cancellation, and
  unavailable-runtime paths fail without changing the document.
- Input remains inside the local Rust-owned helper process boundary and is not
  persisted, logged, transmitted, or retained after the run.
- The visible interaction is keyboard operable, has accessible names and
  announcements, preserves focus, and explains unavailable states.

## Required Evidence

- representative inputs for all five finding codes;
- deterministic repeated-run output;
- stable ordering independent of map and set iteration;
- controlled locale, protocol version, helper version, and thresholds;
- empty and malformed request rejection;
- exact lower and upper text-size boundaries;
- UTF-8 byte-range validation;
- helper timeout, cancellation, and process cleanup;
- packaged helper discovery on the supported macOS Apple Silicon target;
- no network, credential, provider, model-runtime, persistence, or automatic
  document-mutation authority;
- successful execution while the Rust-owned connectivity mode is offline; and
- browser and packaged-app interaction tests for the visible review workflow.

## Analysis Layers

Deterministic measurement is limited to mechanically reproducible counts,
lengths, frequencies, structural presence, and exact patterns used by the named
checks. Deterministic heuristics are repeatable signals and may still be wrong;
they must not be presented as conclusions. Model-backed interpretation includes
argument-quality assessment, synthesis evaluation, intent inference,
substantive critique, conceptual comparison, and generated revision advice and
is outside v1.0.0.

The five permitted user-visible analyses are repeated adjacent word, explicit
long-sentence threshold, extended all-capital emphasis, repeated consecutive
sentence opener, and mixed first-person perspective. Phase 46 must not add a
sixth analysis class under a generic local-analysis label.

## Capability Language

User-visible copy must not imply intelligence, semantic understanding,
reasoning, quality assessment, originality detection, human-likeness detection,
AI detection, AI-powered analysis, semantic analysis, LLM analysis, generative
feedback, or another model-backed capability.

## RC-03 Closure Contract

`RC-03` is the local deterministic analysis path. DRAFT must provide at least
one documented, user-accessible, locally executable text-analysis workflow that
produces stable typed results without network access, provider credentials,
external model services, or packaged model runtimes. Supported analyses must be
explicitly enumerated, bounded, tested, and presented without implying
generative or semantic capabilities.

Closure requires a production Rust path, typed command boundary, frontend
presentation, representative tests, empty and malformed input tests,
size-boundary tests, deterministic output tests, offline execution evidence,
user documentation, packaged-app evidence, and capability-language review.

## Non-Goals

Phase 46 does not add generative writing, summarization, semantic analysis,
argument evaluation, fact checking, source reliability scoring, model-backed
grammar, provider selection, API-key management, document transmission, a
packaged model, automatic edits, or durable finding history.

## Acceptance Boundary

ADR-002 is accepted, so this contract governs Phase 46 implementation. The
production boundary, tests, documentation, packaged helper probe, and visible
workflow now exist. `RC-03` remains open until a stable complete packaged
interaction run supplies the remaining evidence.
