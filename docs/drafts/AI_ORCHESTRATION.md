# AI Orchestration Requirements Draft

## Status

This is a non-binding Phase 27 requirements draft. Implemented behavior must be
recorded separately in `docs/maintainers/AI_ORCHESTRATION.md`. This draft does
not become an accepted contract without the lifecycle in
`docs/GOVERNANCE.md`.

## Purpose

DRAFT needs one Rust-owned boundary that assembles bounded analysis context,
calls a model adapter, streams typed output, and observes cancellation without
allowing generated text to masquerade as verified source evidence.

## Scope

Phase 27 creates an internal Rust orchestration seam. A validated request can be
prepared synchronously to obtain a Rust-generated stream identity, then run
through a caller-supplied model adapter and typed event sink.

The phase defines model-call ownership but does not add a production provider.
OS credential storage is Phase 37, so Phase 27 must not add an API key, secret
field, environment-variable credential, provider endpoint, or external request.
Tests use deterministic in-memory adapters only.

## Request And Context

The request contains:

- one closed analysis task;
- one non-empty instruction of at most 4 KiB;
- at most 64 user-document excerpts;
- at most 64 verified-evidence excerpts; and
- stable evidence IDs and citekeys for evidence blocks.

Each excerpt is non-empty and at most 8 KiB. Evidence IDs are unique. Input
validation completes before worker registration or adapter work.

Context assembly is deterministic and retains whole UTF-8 blocks only. Verified
evidence and user-document text have separate 32 KiB budgets. Input order is
preserved within each class, and omitted block counts are explicit. Instruction
text is validated separately and is never silently truncated.

Every model context block is tagged as either `UserDocument` or
`VerifiedSourceEvidence`. The adapter receives those tags, evidence IDs, and
citekeys. It must not receive a flattened prompt that erases provenance.

## Output Classification

Every streamed chunk and completed result is tagged `GeneratedAnalysis`.
Verified evidence IDs supplied to the model are reported as context scope, not
as proof that the model used or verified a claim. Phase 27 does not parse
model-generated citations or promote generated text into a reference record,
document, fact, or verified-evidence block.

The typed stream sequence is:

```text
Started -> Chunk* -> Completed | Cancelled | Failed
```

The start event reports context counts and omissions. Chunk sequence numbers
start at zero and increase by one. Chunks are non-empty, at most 16 KiB each,
at most 4,096 per stream, and at most 1 MiB cumulatively. Exactly one terminal
event is emitted when the event sink remains available.

## Model Adapter

Rust owns the adapter call. The adapter contract has:

- synchronous stream creation from the typed model request;
- one cancel-safe asynchronous next-chunk operation;
- explicit adapter cancellation; and
- bounded start and stream failure codes.

The orchestration layer validates every returned chunk. Provider-specific
request formats, URLs, authentication, retry policy, and response parsing stay
outside Phase 27.

## Cancellation

Preparation registers one existing `WorkerCancellationRegistry` guard and uses
its Rust-generated worker ID as the stream ID. The prepared generation owns the
guard until terminal exit.

The run loop races each cancel-safe next-chunk future against the existing
cooperative cancellation token. Cancellation drops the pending future, invokes
adapter cancellation, emits `Cancelled`, and returns a typed cancelled outcome.
The loop does not spawn a task. The existing typed `cancel_worker` command is
the only current cancellation IPC.

## Failure Shape

Failures distinguish invalid request, duplicate evidence identity, context
assembly failure, cancellation-registry failure, adapter start failure, adapter
stream failure, invalid or excessive model output, and event-delivery failure.
Errors contain no prompt text, evidence text, document text, model output,
provider response, URL, secret, or raw transport error.

Adapter start or stream failure emits one typed `Failed` event when event
delivery remains available. Event-delivery failure returns directly because the
same failed sink cannot be trusted to accept another terminal event.

## Verification

Tests and scans must cover:

- all request bounds and duplicate evidence IDs;
- deterministic whole-block context assembly and omission counts;
- persistent provenance tags on every context block;
- generated-analysis classification on every output event;
- stable started, chunk, completed, cancelled, and failed serialization;
- ordered chunk sequences and stream-size limits;
- adapter start and mid-stream failure;
- cancellation before and during chunk delivery;
- adapter cancellation and terminal worker registration cleanup;
- no prompt, evidence, model output, or raw failure details in errors;
- no external request in tests;
- no frontend, persistence, document mutation, reference mutation, Python,
  secret, or unmanaged-spawn authority; and
- local/GitHub Actions parity.

## Non-Goals

Phase 27 does not add a production model provider, API key, credential store,
provider selection, custom prompt persistence, Tauri start command, frontend
listener, visible analysis UI, analysis history, automatic document edits,
reference or citation mutation, Python helper, retry loop, usage billing,
tokenizer dependency, tool calling, embeddings, or background task spawning.
