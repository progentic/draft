# AI Orchestration Boundary

## Status

This guide records implemented Phase 27 behavior. The requirements in
`docs/drafts/AI_ORCHESTRATION.md` remain non-binding until they complete the
contract lifecycle in `docs/GOVERNANCE.md`.

## Scope

Phase 27 adds a provider-independent Rust boundary for bounded analysis
requests, provenance-preserving context assembly, typed streamed output, and
cooperative cancellation.

The phase adds no production model adapter, provider URL, credential, external
request, Tauri start command, frontend listener, visible analysis workflow,
analysis persistence, document or reference mutation, Python helper, retry
loop, or spawned worker.

## Request Boundary

`AiAnalysisRequest` accepts one closed analysis task and a non-empty instruction
of at most 4 KiB. A request may contain at most 64 user-document excerpts and
64 verified-evidence excerpts. Each complete excerpt is at most 8 KiB.

Evidence blocks require a unique bounded evidence ID and a citekey accepted by
the existing reference-domain validator. Validation finishes before worker
registration or adapter work. Typed errors contain no instruction, excerpt, or
evidence content.

## Context Assembly

Context assembly retains whole UTF-8 excerpts deterministically. User-document
and verified-evidence blocks have separate 32 KiB budgets, so one class cannot
consume the other class's allowance. Input order is preserved within each
class, and the resulting request reports retained and omitted counts.

Every block remains tagged as `UserDocument` or `VerifiedSourceEvidence`.
Verified blocks retain their evidence ID and citekey. The adapter receives this
typed structure instead of flattened text that would erase provenance.

## Stream Lifecycle

`prepare_ai_generation` assembles context and registers one existing
`WorkerCancellationRegistry` guard. Its Rust-generated worker ID becomes the
stream ID. `run_ai_generation` then coordinates the caller-supplied adapter and
event sink without spawning work.

The lifecycle is:

```text
Started -> Chunk* -> Completed | Cancelled | Failed
```

Every event carries `GeneratedAnalysis`. Verified evidence IDs in `Started`
describe context scope only; they do not classify generated output as evidence.
Chunks are non-empty, at most 16 KiB each, limited to 4,096 chunks and 1 MiB in
total. Sequence numbers start at zero.

## Cancellation And Failure

Each pending adapter read is raced against the cooperative cancellation token.
Cancellation drops that read, calls the adapter's explicit cancellation hook,
emits `Cancelled`, and releases the worker registration. Pre-run cancellation
prevents adapter startup.

Adapter startup, adapter streaming, and invalid output produce bounded typed
failure codes. Event-delivery failure stops the adapter and returns directly.
No error includes source text, generated text, provider response details, URL,
or transport details.

## Abstraction Hierarchy

| Layer | Function or type | Responsibility |
| :--- | :--- | :--- |
| High | `prepare_ai_generation` | Reserves cancellation ownership and prepares bounded model input. |
| High | `run_ai_generation` | Coordinates one typed stream to a terminal outcome. |
| Mid | context assembly and stream progress | Applies provenance, omission, ordering, and output limits. |
| Low | adapter, sink, and cancellation traits | Isolates model reads, event delivery, and token racing. |

## Verification

Seventeen focused Rust tests cover request limits, duplicate evidence IDs,
citekey validation, deterministic whole-block omission, provenance tags,
generated-output classification, serialization, stream ordering and limits,
adapter failures, cancellation before and during reads, sink failure cleanup,
and bounded errors.

`scripts/check-invariants.sh` requires those tests and the fixed limits,
provenance tags, cancellation race, adapter cancellation, and stream
serialization markers. It rejects provider, network, secret, persistence,
Tauri, frontend, mutation, Python, and spawning authority in this boundary.

## Current Limits

Only deterministic in-memory adapters exist in tests. There is no model call,
user-visible event transport, analysis history, automatic edit, or product
workflow. A later phase must introduce credentials and provider integration
through their accepted boundaries before this orchestration can call an
external model.
