# Release Candidate Hardening

## Status

This is the implemented Phase 44 release-candidate hardening baseline as
reconciled by Phase 45. It is not a release-candidate declaration and does not
authorize a tag, signed package, or publication.

The realignment baseline is merged Phase 44 commit `37d0228`. Its post-merge
hosted verification passed. At the audit time, GitHub had no open issues or open pull
requests, and no published releases. Repository scans found no production
`TODO`, `FIXME`, or workaround marker affecting data integrity, security, or
the supported document path.

## Classification Policy

- **Release blocker:** current product or distribution behavior that prevents a
  defensible v1 candidate.
- **Must close before Phase 49:** a mandatory roadmap review gate, even when it
  is not itself a product defect.
- **Accepted v1 limitation:** bounded behavior that may ship when user wording
  remains accurate and the release notes preserve the limitation.
- **P2 maintenance backlog:** useful work that does not block the active release
  path unless later evidence raises its severity.
- **Post-v1 work:** intentionally excluded from the initial release line.

An open blocker must name evidence, an owner, a closure phase, and an executable
closure condition. Removing a row is not closure.

## Blocker Inventory

| ID | Classification | Status | Evidence | Owner | Closure Phase | Closure Condition |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| RC-01 | Release blocker | Open | The workspace discards edits and exposes no create, open, save, close, or reopen controls. | Document workflow | Phase 46 | The visible workflow uses existing Rust-owned lifecycle paths, protects dirty state, passes browser and packaged-app tests, and matches user documentation. |
| RC-02 | Release blocker | Open | Reference storage, metadata lookup, PDF intake, citation insertion, and bibliography checks are internal-only. | Research workflow | Phase 46 | The supported research and citation workflow is visible, understandable, Rust-authorized, recoverable, and covered in the packaged app. |
| RC-03 | Release blocker | Open | Proposed ADR-002 is under review. The local deterministic analysis path has no production command, packaged helper discovery, or visible review workflow; model-backed interpretation remains internal and outside the proposed v1 boundary. | Analysis workflow | Phase 46 | After the ADR-002 governance gate closes, a typed production Rust boundary and user-visible workflow expose exactly the five named heuristics; identical input and configuration produce stable bounded output locally without network or credentials; representative, empty, malformed, size-boundary, offline, and packaged tests pass; and UI plus documentation frame findings as signals without generative, semantic, originality, intelligence, or quality-assessment claims. |
| RC-04 | Release blocker | Open | DOCX compilation is internal-only and no export control exists. | Export workflow | Phase 46 | The supported DOCX path is reachable through a clear Rust-owned target flow and passes packaged source-preservation and recovery tests. |
| RC-05 | Release blocker | Open | Tauri CSP is `null`. | Security review | Phase 48 | A restrictive packaged-app CSP is configured, enforced structurally, and verified against every required local asset and IPC flow. |
| RC-06 | Release blocker | Open | The only package is an unsigned Apple Silicon `.app`; no release is published. | Release engineering | Phase 49 | The candidate distribution is signed, notarized when required, installation-tested, checksummed, and built from the exact candidate commit without adding updater or upload authority implicitly. |
| GATE-45 | Must close before Phase 49 | Closed | Phase 45 reconciles release, roadmap, public, maintainer, Wiki, and enforcement truth without changing product behavior. | Documentation and governance | Phase 45 | The realignment assigns every blocker and binds the v1 usability and interaction-clarity rule without claiming implementation. |
| GATE-46 | Must close before Phase 49 | Open | The mandatory accessibility and interaction-clarity pass has not run. | Frontend accessibility | Phase 46 | Critical flows pass keyboard, focus, naming, announcement, discoverability, label, terminology, and unavailable-state review. |
| GATE-47 | Must close before Phase 49 | Open | Realistic startup, editor, operation-feedback, and large-document responsiveness have not been measured. | Performance | Phase 47 | Measured limits and perceived-wait behavior meet the accepted release thresholds or remain explicit blockers. |
| GATE-48 | Must close before Phase 49 | Open | The final trust-boundary and packaged CSP review has not run. | Security | Phase 48 | Invariant, dependency, capability, CSP, path, archive/XML, secret, network, and frontend-authority review is green with findings closed or blocking. |

## Accepted v1 Limitations

| ID | Classification | Status | Evidence | Owner | Closure Phase | Closure Condition |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| LIMIT-01 | Accepted v1 limitation | Accepted | Initial platform support is macOS on Apple Silicon only. | Release engineering | v1 release notes | Package metadata, download wording, and support docs name the exact platform. |
| LIMIT-02 | Accepted v1 limitation | Accepted | Native PDF export is unavailable under accepted ADR-001. | Architecture | Post-v1 governed phase | ADR reopening conditions must be satisfied before any PDF runtime or claim appears. |
| LIMIT-03 | Accepted v1 limitation | Accepted | Offline mode is process-local and does not probe, persist, queue, or retry. | Network boundary | v1 release notes | Workspace and troubleshooting wording continue to describe the exact behavior. |
| LIMIT-04 | Accepted v1 limitation | Accepted | Formatting review covers heading structure and citation-style declarations, not full style-manual compliance. | Formatting | v1 release notes | Findings remain explainable, review-only where required, and accurately documented. |
| LIMIT-05 | Accepted v1 limitation | Accepted | DRAFT includes no product telemetry. | Privacy | v1 release notes | No telemetry dependency, runtime path, or contrary public claim is introduced. |

## P2 Maintenance Backlog

| ID | Classification | Status | Evidence | Owner | Closure Phase | Closure Condition |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| MAINT-01 | P2 maintenance backlog | Backlog | Vite reports one JavaScript chunk above its 500 kB advisory threshold. | Frontend performance | Phase 47 triage | Measure startup and interaction impact before deciding whether code splitting is release-relevant. |
| MAINT-02 | P2 maintenance backlog | Backlog | `rustdoc -D missing_docs` has hundreds of granular findings. | Maintainer documentation | Post-v1 maintenance | Enable the lint only after focused public API documentation work. |
| MAINT-03 | P2 maintenance backlog | Backlog | ShellCheck, shfmt, Ruff, and frontend linting are not all required locally. | Tooling | Post-v1 maintenance | Pin and require tools only through a dedicated parity-preserving tooling change. |

## Post-v1 Work

| ID | Classification | Status | Evidence | Owner | Closure Phase | Closure Condition |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| POST-01 | Post-v1 work | Deferred | Windows, Linux, and macOS Intel packages are not supported. | Release engineering | Post-v1 | Add one governed platform at a time with native package and boundary verification. |
| POST-02 | Post-v1 work | Deferred | PDF export prerequisites remain unresolved under ADR-001. | Architecture and export | Post-v1 | Reopen only after every ADR prerequisite has accepted evidence. |
| POST-03 | Post-v1 work | Deferred | No automatic updater or update channel exists. | Release engineering | Post-v1 | Add only after signed update metadata, rollback, and trust policy are accepted. |
| POST-04 | Post-v1 work | Deferred | Full style-manual conformance and automatic formatting repair are not implemented. | Formatting | Post-v1 | Expand through reviewable, tested rules without silent document mutation. |

## Binding Usability Rule

DRAFT is not ready for v1.0.0 unless a user can identify the primary controls,
understand their labels, predict their effects, recover from visible failures,
and complete the supported document workflow without relying on maintainer
knowledge.

This is a release condition, not informal polish. Phase 46 owns control
discoverability, labels, menu terminology, unavailable states, keyboard and
focus behavior, announcements, and supported-workflow completion. Phase 47
owns feedback during operations, ambiguous waiting states, and measured and
perceived responsiveness.

## Closure Sequence

1. Phase 46 must close `RC-01` through `RC-04` before it can close `GATE-46`.
2. Phase 47 measures realistic workloads and closes `GATE-47` only when slow or
   ambiguous interactions are fixed or remain explicit release blockers.
3. Phase 48 closes `RC-05` and `GATE-48` through the final security review.
4. Phase 49 may close `RC-06` and cut a candidate only when every prior row is
   closed with its named evidence.

Phase 45 closes only `GATE-45`. It does not close a product, security,
performance, accessibility, or distribution blocker.

## Executable Gates

Phase 44 entry requires a green verified `main`, a reproducible unsigned
supported-host package, migration non-mutation evidence, critical-path evidence,
and no unresolved P0 repository failure.

Phase 44 exits when:

- every known release-relevant finding is classified;
- every open blocker has an owner, phase, and closure condition;
- accepted limitations remain bounded and user wording is already accurate;
- tracked output contains no package or generated release artifact;
- no `v1.0.0` tag or changelog release entry exists prematurely;
- `scripts/check-release-candidate.sh` passes locally and in hosted CI; and
- the full verifier passes.

Phase 49 entry is stricter: every `RC-*` and `GATE-*` row must be closed with
the named evidence. Phase 44 hardening and Phase 45 realignment do not satisfy
that future gate.

## Explicit Exclusions

Phases 44 and 45 add no product workflow, UI control, signing identity, credential,
notarization command, upload logic, updater, broad package target, release tag,
changelog release entry, or Phase 49 candidate execution. Phase 46 is the next
implementation and interaction-clarity boundary.
