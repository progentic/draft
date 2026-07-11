# ADR-002: Limit v1 Analysis to Local Text Review

Date: 2026-07-11
Status: Proposed
Deciders: @progentic

## Context

DRAFT has a provider-independent Rust orchestration boundary and a separate
Rust-owned Python helper that produces five deterministic text-analysis finding
types. Neither boundary is currently reachable from the product. Phase 45
assigned the missing visible analysis workflow to release blocker `RC-03`, but
that blocker also assumed a production model provider would be selected before
v1.0.0.

Selecting an external provider now would require decisions about credentials,
privacy disclosures, data transmission, offline exceptions, availability,
cost, rate limits, failure recovery, provider-specific tests, and support. A
packaged local model would instead require a model license, resource budgets,
hardware limits, update policy, and a substantially larger distribution. Those
commitments are not prerequisites for exposing the deterministic local checks
that already exist.

The repository owner therefore authorized a narrower v1 analysis boundary. The
following alternatives were considered:

- **External model provider:** Enables generative and semantic analysis, but
  introduces network, credential, privacy, cost, availability, and provider
  support commitments too late in the release cycle.
- **Packaged local model:** Avoids transmission and provider credentials, but
  introduces model licensing, package-size, performance, hardware, update, and
  deterministic-verification requirements that are not accepted.
- **Local deterministic text analysis:** Exposes the existing bounded grammar,
  clarity, tone, cohesion, and voice heuristics without a network, credential,
  or model runtime.
- **Retain `RC-03` unchanged and delay release:** Avoids narrowing the blocker,
  but leaves an implemented local review capability inaccessible and prevents
  the release from advancing without a provider decision.

## Owner Authorization

> **Owner Decision: v1.0.0 Analysis Boundary**
>
> For DRAFT v1.0.0, production analysis is limited to local, deterministic
> text-analysis capabilities that require no external model provider, network
> transmission, provider credentials, or packaged model runtime.
>
> The existing provider-independent analysis orchestration boundary will remain
> intact for future integrations, but no external or generative AI provider will
> be selected or shipped in v1.0.0.
>
> Revise `RC-03` through the repository's governance process so that the release
> requirement is satisfied by a tested, documented, locally executable
> deterministic analysis path. The revised requirement must not claim
> generative, semantic, or model-backed analysis capabilities that are not
> implemented.
>
> Any future external-model integration requires a separate accepted ADR
> covering provider selection, credential handling, privacy, offline behavior,
> failure policy, packaging, testing, and user disclosure.
>
> Phase 46 may proceed after this decision is recorded and the `RC-03` revision
> is accepted through the required governance gates.

## Decision

For DRAFT v1.0.0, production analysis is limited to local deterministic text
analysis. The supported functions are the existing repeated-word, long-sentence,
extended-capital-emphasis, repeated-sentence-opener, and mixed-first-person
checks. They remain advisory review findings, not correctness judgments.

The v1 path must execute locally through the Rust-owned helper boundary and
produce the same ordered findings for identical input and configuration. It
must not require or imply an external model provider, network transmission,
provider credential, generative output, semantic understanding, or packaged
model runtime.

The provider-independent Rust orchestration boundary remains intact but
internal. DRAFT will not select or ship an external or generative AI provider
in v1.0.0. A future provider integration requires a separate accepted ADR that
covers provider selection, credential handling, privacy and disclosure,
offline behavior, failure policy, packaging, resource use, testing, and user
support.

Phase 46 may revise `RC-03` to close only when the local workflow is visible,
typed, bounded, cancel-safe where applicable, accessible, packaged, and
documented without unsupported AI claims.

## Analysis Layers

The v1 boundary distinguishes three layers. Determinism describes repeatability,
not truth or correctness.

**Deterministic measurement** covers objective, mechanically reproducible
observations such as word and sentence counts, lengths, frequencies, structural
presence, and exact pattern matches. Phase 46 may use those measurements only
to support the five enumerated checks; this ADR does not authorize additional
user-visible analysis classes.

**Deterministic heuristics** covers repeatable but interpretive signals. The
permitted v1 findings are exactly:

- repeated adjacent word;
- sentence longer than the explicit word threshold;
- extended all-capital emphasis;
- repeated consecutive sentence opener; and
- mixed singular and plural first-person perspective.

These findings must be presented as possible review concerns, not conclusions,
quality judgments, or proof of an error. Adding another visible deterministic
analysis class requires an explicit accepted contract update with named
behavior, bounds, wording, and tests.

**Model-backed interpretation** covers argument-quality assessment, synthesis
evaluation, intent inference, substantive critique, conceptual comparison, and
generated revision advice. It is outside v1.0.0 along with every external or
packaged model path.

The product, marketing, menus, documentation, and errors must not describe the
v1 workflow as intelligence, semantic understanding, reasoning, quality
assessment, originality detection, human-likeness detection, AI detection,
AI-powered analysis, semantic analysis, LLM analysis, or generative feedback.

## Consequences

The initial release can provide useful language review without transmitting
document text outside the local Rust-owned helper process or asking users for a
service credential. Input travels only through bounded local process pipes and
must not be persisted, logged, or sent over a network. Offline behavior is
simple, repeatable output can be tested exactly, and the existing source-safety
and review-only boundaries remain unchanged.

The analysis label must remain narrow. DRAFT v1.0.0 cannot claim generative,
semantic, model-backed, summarization, argument-evaluation, fact-checking,
reliability-scoring, or source-verification capabilities. The current five
heuristics also do not constitute comprehensive grammar checking.

This decision makes provider-backed analysis unavailable in v1.0.0 and requires
a later architecture cycle before that work can begin. It also requires Phase
46 to package and invoke the deterministic helper through Rust rather than
reimplementing checks in the frontend.

DRAFT therefore has a narrower competitive story for v1.0.0. Marketing cannot
imply generative analysis, semantic analysis, AI-assisted revision, model-based
synthesis, or provider-backed research assistance. Some users may expect those
capabilities from this product category; accurate absence wording is part of
the release contract, not optional positioning polish.

Affected downstream surfaces are `ARCHITECTURE.md`, `INVARIANTS.md`,
`ROADMAP.md`, `PHASEMAP.md`, `docs/maintainers/AI_ORCHESTRATION.md`,
`docs/maintainers/TEXT_ANALYSIS.md`,
`docs/maintainers/RELEASE_CANDIDATE.md`, user limitations, and invariant,
documentation, and release-candidate checks.

## Enforcement

While this ADR is proposed, a named guard keeps the existing model orchestration
boundary internal. It rejects external model SDK dependencies, provider endpoint
constants, provider credential environment variables, packaged model files,
runtime model-download logic, analysis-domain network calls, frontend provider
or secret authority, arbitrary model endpoints, and generative-analysis bridge
surfaces. The documentation check rejects unsupported capability claims,
requires proposal-state language, and keeps `RC-03` open.

Rust remains authoritative for helper discovery, execution, validation, limits,
and typed failure mapping. The frontend may submit one bounded snapshot through
the accepted command boundary and display validated findings; it cannot call a
provider, access secret storage, choose an endpoint, or execute analysis logic.

After acceptance, Phase 46 must replace the text-analysis absence checks with
behavioral evidence for representative, empty, malformed, and size-boundary
inputs; identical repeated output; explicit thresholds; controlled locale and
versions; stable ordering independent of map or set iteration; typed failures;
offline and packaged execution; accessible presentation; and the absence of
network and credential authority. Local verification and GitHub Actions must
run the same enforcement.

## Links

- `ARCHITECTURE.md` §3.2 and §3.4
- `INVARIANTS.md` `INV-10`, `INV-11`, `INV-14`, and `INV-15`
- `docs/drafts/V1_LOCAL_ANALYSIS.md`
- `docs/maintainers/RELEASE_CANDIDATE.md` `RC-03`
