# V1 Local Analysis Contract

**Status:** Draft, non-binding
**Decision dependency:** Proposed ADR-002
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
  ordered findings.
- The visible workflow names grammar, clarity, tone, cohesion, and voice review
  and identifies the five supported checks without claiming comprehensive
  correctness.
- Findings remain review-only. They may focus a current passage but cannot
  persist, replace text, or apply an edit.
- The workflow exposes typed, non-sensitive failures through the existing error
  presentation policy.
- Empty, malformed, unsupported-version, oversized, timeout, cancellation, and
  unavailable-runtime paths fail without changing the document.
- The visible interaction is keyboard operable, has accessible names and
  announcements, preserves focus, and explains unavailable states.

## Required Evidence

- representative inputs for all five finding codes;
- deterministic repeated-run output;
- empty and malformed request rejection;
- exact lower and upper text-size boundaries;
- UTF-8 byte-range validation;
- helper timeout, cancellation, and process cleanup;
- packaged helper discovery on the supported macOS Apple Silicon target;
- no network, credential, provider, model-runtime, persistence, or automatic
  document-mutation authority; and
- browser and packaged-app interaction tests for the visible review workflow.

## Non-Goals

Phase 46 does not add generative writing, summarization, semantic analysis,
argument evaluation, fact checking, source reliability scoring, model-backed
grammar, provider selection, API-key management, document transmission, a
packaged model, automatic edits, or durable finding history.

## Acceptance Boundary

This draft becomes implementation guidance only after the ADR-002 governance
gate closes.
Until then, `RC-03` remains open and no Phase 46 analysis implementation may
begin.
