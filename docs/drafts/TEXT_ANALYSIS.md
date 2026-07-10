# Text-Analysis Findings Requirements Draft

## Status

This is a non-binding Phase 29 requirements draft. Implemented behavior must be
recorded separately in `docs/maintainers/TEXT_ANALYSIS.md`. This draft does not
become an accepted contract without the lifecycle in `docs/GOVERNANCE.md`.

## Purpose

DRAFT needs deterministic writing checks that return reviewable, explainable
findings without rewriting text, pretending to prove correctness, or granting a
Python helper authority over documents or durable state.

## Scope

Phase 29 extends the Phase 28 helper protocol with allowlisted `text_analysis`
version 1. It returns five narrowly defined review signals:

- `repeated_word` for the second of two adjacent case-insensitive words;
- `long_sentence` for a sentence containing more than 30 lexical words;
- `all_caps_emphasis` for an all-uppercase lexical word of five or more letters;
- `repeated_sentence_opener` when consecutive sentences begin with the same
  case-insensitive word of four or more letters; and
- `mixed_first_person` where a document changes between first-person singular
  and plural pronouns.

These checks are heuristics. A finding means “review this passage,” not “the
passage is wrong.” Phase 29 adds no score, correction engine, silent rewrite, or
claim that all grammar, clarity, tone, cohesion, or voice problems are detected.

## Request

The request reuses the Phase 28 protocol envelope, Rust-generated UUID identity,
closed `en-US` locale, 32 KiB text bound, 64 KiB request bound, fixed entrypoint,
isolated process, timeout, cancellation, and stream limits. The only new
allowlist entry is `text_analysis` at helper version 1.

The request carries plain text only. It contains no document ID, source path,
document JSON, citation, reference, credential, network, persistence, command,
environment, or mutation field.

## Finding Shape

Python returns only a closed finding code and a half-open UTF-8 byte range:

```json
{
  "code": "repeated_word",
  "startByte": 4,
  "endByte": 7
}
```

Rust rejects more than 100 findings, unknown codes or fields, duplicate or
unsorted findings, empty or reversed ranges, out-of-bounds ranges, and ranges
that do not start and end on UTF-8 character boundaries.

Python does not supply user-facing prose. Rust maps each closed code to one
category, severity, title, and explanation:

| Code | Category | Severity | Review meaning |
| :--- | :--- | :--- | :--- |
| `repeated_word` | Grammar | Warning | An adjacent word may have been duplicated. |
| `long_sentence` | Clarity | Advice | A long sentence may be easier to review in smaller parts. |
| `all_caps_emphasis` | Tone | Advice | Extended capitals may read as unusually forceful. |
| `repeated_sentence_opener` | Cohesion | Advice | Repeated openings may make adjacent sentences feel repetitive. |
| `mixed_first_person` | Voice | Advice | Singular and plural first-person perspectives both appear. |

Messages are fixed, bounded, and contain no source text. Findings contain no
replacement text and no apply operation.

## Determinism

The Python helper uses standard-library Unicode-aware lexical matching and fixed
thresholds. It emits at most 20 findings from each check, then sorts the combined
set by start byte, end byte, and code. The complete result is capped at 100.
Equal input and protocol versions must produce equal findings.

Sentence segmentation is deliberately simple: `.`, `?`, and `!` terminate a
sentence candidate. It does not attempt full linguistic parsing. All-caps checks
ignore words shorter than five letters to reduce acronym noise. Voice findings
point to the first pronoun from the perspective group that appears later.

## Non-Destructive Boundary

Rust validates helper output into immutable `TextAnalysisResult` values. Phase 29
does not persist them, emit them through Tauri, display them in React, attach
them to Tiptap, mutate the document envelope, write files, or apply a suggestion.
Any future accept/apply workflow must use the existing explicit document
mutation and save boundaries.

## Failure Shape

Text-analysis calls reuse the Phase 28 typed configuration, request, process,
timeout, cancellation, output, execution, helper-rejection, and response-mismatch
errors. Errors contain no input text, output JSON, finding range, path, stderr,
or raw process detail.

## Verification

Tests and scans must cover:

- all five finding codes and their fixed Rust-owned explanations;
- threshold boundaries and false-positive guards;
- Unicode UTF-8 byte offsets and Rust character-boundary validation;
- sorted, deterministic, duplicate-free output;
- per-check and total finding limits;
- exact protocol/helper versions and unknown-field rejection;
- malformed, unknown, duplicate, unsorted, excessive, out-of-range, reversed,
  and non-character-boundary findings;
- real isolated Python subprocess round trips;
- timeout, cancellation, environment, stream, and child-cleanup behavior from
  Phase 28 remains active;
- no source text in findings or errors;
- no replacement, apply, document mutation, persistence, Tauri, frontend,
  network, credential, filesystem, or subprocess authority; and
- local/GitHub Actions parity.

## Non-Goals

Phase 29 does not add comprehensive grammar checking, syntax parsing, language
detection, readability or quality scores, machine learning, model calls,
third-party linguistic libraries, custom dictionaries, locale selection,
document extraction, Tiptap position conversion, accepted edits, issue
persistence, events, Tauri commands, visible issue cards, or frontend controls.
