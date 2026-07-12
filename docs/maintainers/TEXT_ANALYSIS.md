# Text-Analysis Findings

## Status

This guide records the Phase 29 domain and its Phase 46 production integration.
Accepted ADR-002 and `docs/drafts/V1_LOCAL_ANALYSIS.md` govern the visible v1
boundary.

## Scope

Phase 29 extends the Phase 28 helper protocol with allowlisted
`text_analysis` version 1 and five deterministic review signals for grammar,
clarity, tone, cohesion, and voice.

The domain adds no score, replacement text, apply operation, document mutation,
persistence, event, comprehensive language claim, model call, or third-party
Python dependency. Phase 46 adds one typed command and one visible review panel
without widening those domain capabilities.

## Checks

The standard-library Python helper runs these fixed checks:

| Code | Trigger | Per-check limit |
| :--- | :--- | :--- |
| `repeated_word` | The same case-insensitive lexical word appears twice with only whitespace between it. | 20 |
| `long_sentence` | A simple `.`, `?`, or `!` sentence candidate contains more than 30 lexical words. | 20 |
| `all_caps_emphasis` | A lexical word contains at least five letters and is all uppercase. | 20 |
| `repeated_sentence_opener` | Consecutive sentence candidates begin with the same case-insensitive word of at least four letters. | 20 |
| `mixed_first_person` | Both singular and plural first-person pronoun groups occur. | 1 |

These are review heuristics, not correctness judgments. The checks intentionally
avoid full parsing, abbreviation handling, acronym dictionaries, locale
detection, readability scores, and machine learning.

## Wire Result

Python emits only `code`, `startByte`, and `endByte`. Ranges are half-open UTF-8
byte offsets into the exact submitted text. Results are sorted by start, end,
and code, capped at 20 per check and 100 overall, and deterministic for equal
input and versions.

The wire result contains no excerpt, title, explanation, score, replacement,
document identity, or apply instruction. The Phase 28 request envelope remains
unchanged except for the closed `text_analysis` helper name and helper version
1.

## Rust Validation

Rust rejects unknown fields and codes, more than 100 findings, duplicate or
unsorted entries, empty or reversed ranges, out-of-bounds offsets, and offsets
inside a UTF-8 code point. The response must also repeat the Rust-generated
request ID and exact protocol/helper versions before findings are considered.

After validation, Rust maps each code to a fixed category, severity, title, and
explanation. Python cannot inject user-facing prose.

| Code | Category | Severity | Rust title |
| :--- | :--- | :--- | :--- |
| `repeated_word` | Grammar | Warning | Repeated word |
| `long_sentence` | Clarity | Advice | Long sentence |
| `all_caps_emphasis` | Tone | Advice | Extended capital emphasis |
| `repeated_sentence_opener` | Cohesion | Advice | Repeated sentence opening |
| `mixed_first_person` | Voice | Advice | First-person perspective shift |

Explanations describe why the range deserves review and explicitly avoid
claiming that the writing is wrong. Findings retain no source-text copy.

## Process And Ownership

The `run_text_analysis` command reuses the Phase 28 canonical fixed entrypoint, isolated and
cleared environment, bounded stdin/stdout/stderr, five-second timeout,
cooperative cancellation, kill/reap behavior, typed failures, and
`WorkerRegistration` lifetime. It resolves the packaged helper resource, uses
the fixed Apple system Python path, clears the environment, and restores only
`TMPDIR=/tmp`.

The request carries one immutable bounded text snapshot and closed locale. The
helper cannot read a document, write a file, inspect application state, persist
a finding, call a network service, start a subprocess, or apply an edit. Rust
returns `TextAnalysisResult` through the strict command client. The frontend
presents fixed wording, passage location, and transient state only. No
application state initializes or retains a runner.

## Abstraction Hierarchy

| Layer | Function or type | Responsibility |
| :--- | :--- | :--- |
| High | `run_text_analysis` | Coordinates one immutable review request through validated output. |
| Mid | `validate_text_analysis_result` | Enforces code, order, count, UTF-8 range, and explanation policy. |
| Low | Phase 28 runner | Moves bounded bytes and owns timeout, cancellation, kill, and reap. |
| Python | five check functions | Detect narrowly defined deterministic review signals. |

## Verification

Focused Rust tests cover the text-analysis request allowlist, strict
success shape, unknown codes and fields, all five explanation policies, UTF-8
boundaries, invalid ranges, duplicates, ordering, count limits, lack of
replacement/source fields, overlapping-code ordering, real subprocess round
trips, exact 32 KiB input bounds, offline execution, Apple system Python, and
packaged resource execution.

Five new Python tests cover all finding families, exact thresholds,
false-positive guards, Unicode byte offsets, deterministic ordering, and
per-check/total limits. The complete Phase 28 timeout, cancellation,
environment, process, protocol, and authority suite remains active.

`scripts/check-invariants.sh` requires those tests, helper/version and threshold
constants, fixed Rust explanations, byte-boundary checks, ordering and count
validation, and production-authority denials. It rejects scoring, replacement,
apply, persistence, unowned command/frontend access, document/reference
mutation, networking, credentials, filesystem access, and subprocess authority.

## Current Limits

Sentence segmentation treats `.`, `?`, and `!` as simple boundaries and may
split abbreviations. All-caps detection uses length instead of an acronym
dictionary. Perspective changes can be intentional. Findings therefore remain
advisory review prompts. The visible UI preserves this wording and requires the
user to edit and save explicitly.

Accepted ADR-002 makes these five deterministic checks the complete v1.0.0
production analysis scope. It does not imply generative, semantic, model-backed,
comprehensive grammar, provider, credential, network, or packaged-model
capabilities. Phase 46 closes `RC-03` with the typed local workflow and its
browser/package evidence.

Counts, lengths, and exact patterns are supporting measurements only. The five
findings are deterministic heuristics: repeatable signals that may still be
wrong and are not conclusions. Model-backed interpretation is a separate
excluded class. Another measurement cannot become a sixth visible analysis
capability without a governed contract update.

## Configuration Index

Finding counts, locale, and deterministic heuristic thresholds are indexed in
`docs/maintainers/CONFIGURATION.md`.
