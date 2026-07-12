# Release Candidate Hardening

## Status

This is the implemented Phase 44 release-candidate hardening baseline as
reconciled by Phase 45. It is not a release-candidate declaration and does not
authorize a tag, signed package, or publication.

The accepted `docs/contracts/V1_USABILITY_ACCEPTANCE.md` contract makes user
comprehension and supported-workflow completion executable release gates. It
does not close any row in this ledger or imply that an absent workflow exists.

The realignment baseline is merged Phase 44 commit `37d0228`. Its post-merge
hosted verification passed. At the audit time, GitHub had no open issues or open pull
requests, and no published releases. Repository scans found no production
`TODO`, `FIXME`, or workaround marker affecting data integrity, security, or
the supported document path.

## Classification Policy

- **Release blocker:** current product or distribution behavior that prevents a
  defensible v1 candidate.
- **Roadmap gate:** a mandatory phase boundary, even when it is not itself a
  product defect.
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
| RC-01 | Release blocker | Open | Browser interaction tests pass, but the first exact-artifact manual run found `UX-46-001`: Save produced an unrecoverable macOS beach ball and required force-quit; a stable complete packaged-app lifecycle run is still missing. | Document workflow | Phase 46 | The visible workflow uses existing Rust-owned lifecycle paths, protects dirty state, passes browser and packaged-app tests, and matches user documentation. |
| RC-02 | Release blocker | Open | Manual reference create/list and citation insertion are visible and Rust-authorized; a stable packaged reference/citation workflow run is still missing. Metadata lookup, PDF intake, and bibliography management remain intentionally unavailable. | Research workflow | Phase 46 | The supported research and citation workflow is visible, understandable, Rust-authorized, recoverable, and covered in the packaged app. |
| RC-03 | Release blocker | Open | Accepted ADR-002 is implemented through the typed production command, packaged helper discovery, exact five-check workflow, deterministic/offline/size/error tests, and bounded heuristic wording; a stable complete packaged interaction run is still missing. | Analysis workflow | Phase 46 | A typed production Rust boundary and user-visible workflow expose exactly the five named heuristics; identical input and configuration produce stable bounded output locally without network or credentials; representative, empty, malformed, size-boundary, offline, and packaged tests pass; and UI plus documentation frame findings as signals without generative, semantic, originality, intelligence, or quality-assessment claims. |
| RC-04 | Release blocker | Open | The Rust-owned DOCX target flow and visible export control are implemented with source-preservation tests; a stable packaged success/failure/recovery run is still missing. | Export workflow | Phase 46 | The supported DOCX path is reachable through a clear Rust-owned target flow and passes packaged source-preservation and recovery tests. |
| RC-05 | Release blocker | Open | Tauri CSP is `null`. | Security review | Phase 51 | A restrictive packaged-app CSP is configured, enforced structurally, and verified against every required local asset and IPC flow. |
| RC-06 | Release blocker | Open | The only package is an unsigned Apple Silicon `.app`; no release is published. | Release engineering | Phase 52 | The candidate distribution is signed, notarized when required, installation-tested, checksummed, built from the exact candidate commit, and passes the complete packaged usability workflow without adding updater or upload authority implicitly. |
| RC-07 | Release blocker | Open | Parsed Markdown, DOCX import and safe round-trip save, format lossiness state, and governed ODT/RTF dispositions are absent. | Document interoperability | Phase 47 | Supported-format fixtures, Rust-owned lifecycle state, explicit fidelity classes, no-edit source preservation, typed failures, and lossless or rejected-lossy save behavior pass. |
| RC-08 | Release blocker | Open | Primary workflows lack native macOS menu integration, shared action dispatch, final window branding, and accepted desktop-layout evidence. | Desktop UI and native workflow | Phase 48 | Native menus, visible controls, and shortcuts share one state-aware action path and pass packaged icon, layout, focus, enablement, and keyboard evidence. |
| GATE-45 | Roadmap gate | Closed | Phase 45 reconciles release, roadmap, public, maintainer, Wiki, and enforcement truth without changing product behavior. | Documentation and governance | Phase 45 | The realignment assigns every blocker and binds the v1 usability and interaction-clarity rule without claiming implementation. |
| GATE-46 | Roadmap gate | Open | Automated accessibility, keyboard, focus, announcement, state, recovery, responsive-layout, and visible-workflow checks pass, but `UX-46-001` through `UX-46-015` remain open. Accepted ADR-003 assigns findings `UX-46-008` through `UX-46-015` to later interoperability and desktop-workflow phases without closing this gate. | Frontend accessibility and workflow clarity | Phase 46 | `RC-01` through `RC-04` close and critical flows satisfy accepted discoverability, understanding, state, recovery, keyboard, focus, naming, announcement, terminology, and unavailable-state criteria. |
| GATE-47 | Roadmap gate | Open | The accepted document-interoperability implementation and evidence do not exist. | Document interoperability | Phase 47 | `RC-07` closes with the accepted format matrix, lifecycle, fidelity, lossiness, source-preservation, and fixture evidence. |
| GATE-48 | Roadmap gate | Open | The accepted desktop UI and native workflow integration and packaged evidence do not exist. | Desktop UI and native workflow | Phase 48 | `RC-08` closes with shared dispatch, native-menu parity, state, icon, layout, focus, and keyboard evidence. |
| GATE-49 | Roadmap gate | Open | First-time-user, documentation-comprehension, realistic-workload, and measured/perceived performance validation has not run. | Usability and performance | Phase 49 | Accepted task, terminology, recovery, satisfaction, documentation, and performance thresholds pass without open `UX-0` or `UX-1` findings. |
| GATE-50 | Roadmap gate | Open | The post-interoperability and desktop-workflow documentation and drift realignment has not run. | Documentation and governance | Phase 50 | Repository, user, maintainer, architecture, contract, release, implementation, terminology, onboarding, and cross-link truth agree; `INV-UX-07` remains Proposed until its separate acceptance evidence exists. |
| GATE-51 | Roadmap gate | Open | The final trust-boundary, parser, dependency, packaged CSP, source-safety, and secure-usability review has not run. | Security | Phase 51 | `RC-05` closes and invariant, dependency, capability, CSP, path, archive/XML, secret, network, frontend-authority, security-wording, safe-recovery, and work-preservation review is green. |

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
| MAINT-01 | P2 maintenance backlog | Backlog | Vite reports one JavaScript chunk above its 500 kB advisory threshold. | Frontend performance | Phase 49 triage | Measure startup and interaction impact before deciding whether code splitting is release-relevant. |
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

This is a release condition, not informal polish. Phase 46 owns the clarity of
its supported workflows, Phase 48 owns native desktop action integration, and
Phase 49 owns complete terminology, comprehension, human-task, and measured and
perceived responsiveness validation. The complete workflow and thresholds are
defined in `docs/contracts/V1_USABILITY_ACCEPTANCE.md`; interoperability and
desktop evidence are defined in
`docs/contracts/V1_INTEROPERABILITY_AND_DESKTOP_WORKFLOWS.md`.

## Usability Finding Taxonomy

- **UX-0:** data loss, inaccessible critical action, or inability to complete
  the primary workflow. Any open `UX-0` blocks Phase 52.
- **UX-1:** misleading label, hidden primary control, unrecoverable confusion,
  or unsupported capability claim. Any open `UX-1` blocks Phase 52.
- **UX-2:** meaningful friction that should be fixed before release. Every
  `UX-2` requires a recorded fix, accepted limitation, or owner-approved
  deferral with rationale.
- **UX-3:** an enhancement suitable for later maintenance.

Findings and dispositions belong in
`docs/maintainers/V1_USABILITY_EVIDENCE.md`. The evidence ledger is created
when Phase 46 produces real workflow evidence; this baseline does not create
empty or fabricated results.

## Closure Sequence

1. Phase 46 must close `RC-01` through `RC-04` before it can close `GATE-46`.
2. Phase 47 closes `RC-07` and `GATE-47` through accepted interoperability evidence.
3. Phase 48 closes `RC-08` and `GATE-48` through accepted desktop workflow evidence.
4. Phase 49 closes `GATE-49` only after the complete human usability,
   documentation-comprehension, and performance thresholds pass.
5. Phase 50 closes `GATE-50` through mandatory documentation and drift realignment.
6. Phase 51 closes `RC-05` and `GATE-51` through final security and secure-usability review.
7. Phase 52 may close `RC-06` and cut a candidate only when every prior row is
   closed, the exact package passes the complete workflow, no `UX-0` or `UX-1`
   remains open, and every `UX-2` has a recorded disposition.
8. Phase 53 may tag and publish only after first-run, Start Here, shortcut,
   troubleshooting, user release-note, download, and launch evidence is green.

Phase 45 closes only `GATE-45`. It does not close a product, security,
performance, accessibility, or distribution blocker.

## Accepted ADR-003 Gate Chain

ADR-003 and its successor contract are accepted and binding. This remap changes
phase ownership without treating renumbering as closure. Every new or remapped
row remains open until its named evidence exists.

| Row | Owner | Closure basis |
| :--- | :--- | :--- |
| `RC-01` through `RC-04`, `GATE-46` | Phase 46 | Existing supported-workflow and accessibility evidence. |
| `RC-07`, `GATE-47` | Phase 47 | Document interoperability, format ownership, lossiness, fidelity, and source-preservation evidence. |
| `RC-08`, `GATE-48` | Phase 48 | Native-menu and visible-control parity, shared dispatch, state, keyboard, icon, and desktop-layout evidence. |
| `GATE-49` | Phase 49 | First-time-user usability, maintainer-documentation comprehension, and measured and perceived performance evidence. |
| `GATE-50` | Phase 50 | Mandatory plain-language, maintainer-onboarding, terminology, cross-link, and drift realignment evidence. |
| `RC-05`, `GATE-51` | Phase 51 | Security, parser, CSP, dependency, source-safety, and secure-usability evidence. |
| `RC-06` | Phase 52 | Exact final-candidate distribution and complete packaged workflow evidence. |
| Release | Phase 53 | Every prior row is closed and v1.0.0 publication checks pass. |

The former `GATE-47` usability and `GATE-48` security meanings are superseded.
Structural checks reject closure against that obsolete numbering.

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

Phase 52 entry is stricter: every prior-phase `RC-*` and `GATE-*` row must be
closed with the named evidence, and the usability ledger must contain no open
`UX-0` or `UX-1`. Phase 44 hardening and Phase 45 realignment do not satisfy
that future gate.

## Explicit Exclusions

Phases 44 and 45 add no product workflow, UI control, signing identity, credential,
notarization command, upload logic, updater, broad package target, release tag,
changelog release entry, or Phase 52 candidate execution. Phase 46 is the next
implementation and interaction-clarity boundary.
