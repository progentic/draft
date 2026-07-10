# DRAFT — Governance

**Status:** Draft v0.4  
**Product name:** D.R.A.F.T — Document Research, Analysis, Formatting & Text-analysis  
**Purpose:** Define how architectural decisions, invariants, ADRs, contract documents, and build enforcement change over time.

This document is about process, not product design. It exists so that future changes do not silently reverse the reason a boundary was created.

---

## 1. Document Roles

- **`ARCHITECTURE.md`:** current agreed system shape. Changes to it are architectural decisions and follow this governance process.
- **`INVARIANTS.md`:** binding mechanical rules the implementation must obey. It defines the invariant, the risk it prevents, and the local/GitHub Actions enforcement path.
- **`GOVERNANCE.md`:** this document. It defines how architecture, invariants, ADRs, and downstream contracts change.
- **`docs/adr/*.md`:** Architecture Decision Records. Immutable history of why a decision changed.
- **`docs/contracts/*.md`:** accepted downstream specifications for APIs, schemas, commands, workers, formatting rules, and data models.
- **`docs/drafts/*.md`:** exploratory documents. These are non-binding until accepted through the contract lifecycle.
- **`.github/workflows/*.yml`:** GitHub Actions enforcement.
- **`justfile` and `scripts/*`:** local development and CI command surface.

Granular contracts do not become binding until the architectural section they depend on is stable and its relevant invariants are accepted. Writing a schema against an undecided boundary just means rewriting the schema later.

---

## 2. Current Stack Decision

DRAFT uses this stack:

- Rust for the trusted core.
- TypeScript for the frontend application code.
- Pure React for the UI layer.
- Tiptap for document editing.
- Tauri 2 for the desktop application shell and IPC bridge.
- Python for allowlisted formatting and text-analysis helper workers.
- Bash for local development and GitHub Actions orchestration.

This stack is an architectural decision. Changing any major stack component requires an ADR.

Examples of changes that require an ADR:

- Replacing Tauri with Electron or a web-only app.
- Replacing Tiptap with another editor framework.
- Moving persistence or network ownership from Rust into TypeScript.
- Allowing Python helpers to call external network services directly.
- Using Bash as a product runtime path.
- Adding a new runtime language.

Examples that do not require an ADR:

- Updating a dependency within the same accepted stack.
- Adding a formatter or linter for an accepted language.
- Refactoring implementation inside an accepted boundary.

---

## 3. Authority and Change Process

This process applies to changes to:

- `ARCHITECTURE.md`
- `GOVERNANCE.md`
- `INVARIANTS.md`
- accepted contract documents
- accepted ADR status links
- build workflows that enforce invariants
- adding, removing, weakening, or rewriting an invariant

### 3.1 While Solo

A solo project is the highest-risk governance mode because the author and approver are the same person.

To reduce impulsive rewrites:

1. All architectural changes must go through a Pull Request. No direct pushes to `main` for governed files.
2. Architecture PRs must remain open for a mandatory 24-hour cooling-off period before merge.
3. Architecture PRs must carry the `architecture` label.
4. Self-review is required. The PR author must leave a review comment stating the alternatives considered and what becomes harder after the change.
5. The PR must include an ADR unless the change is purely editorial and does not alter meaning.

### 3.2 When Team Size Is Greater Than One

When another maintainer exists:

1. Add `CODEOWNERS` entries for governed files.
2. Require at least one approving review from a code owner who is not the author.
3. Keep the 24-hour window for changes labeled `breaking-invariant`.
4. Keep ADR requirements for architectural changes.

Branch protection enforces this mechanically. Process that relies only on memory or discipline does not survive.

---

## 4. How to Change an Architectural Decision

An architectural change must do the following in one PR:

1. **State the current decision and trigger.** Required form: “The constraint protecting X no longer applies because Y” or “New requirement Z conflicts with current decision because...”
2. **Write the ADR first.** Create `docs/adr/NNN-short-title.md` using the template in §5.
3. **Check downstream dependents.** List every contract doc, schema, test suite, script, workflow, and invariant that assumes the current shape.
4. **Update the living docs.** Update `ARCHITECTURE.md`, `INVARIANTS.md`, and relevant contract docs in the same PR.
5. **Show enforcement.** Add or update local and GitHub Actions enforcement for any new or modified invariant.
6. **Pass local verification.** `just verify` and `just check-invariants` must pass before review.
7. **Pass CI verification.** GitHub Actions must pass before merge.

Merge means the decision is accepted.

Squash merge commit messages must preserve the ADR number:

```text
feat(arch): accept ADR-007 [closes #123]
```

---

## 5. Architecture Decision Records

Commit messages get squashed, rebased, and forgotten. ADRs do not.

**Location:** `docs/adr/`  
**Naming:** `NNN-kebab-case-title.md`, zero-padded and monotonic. Example: `001-use-sqlite-for-document-index.md`  
**Status lifecycle:** `Proposed` -> `Accepted` -> `Superseded by ADR-XXX` or `Deprecated`

Accepted ADR bodies are immutable except for status links and post-mortems.

### 5.1 ADR Template

```markdown
# ADR-NNN: Title

Date: YYYY-MM-DD
Status: Proposed | Accepted | Superseded by ADR-XXX | Deprecated
Deciders: @iangordon

## Context

What is the problem? What constraint forces a decision? What happens if we do nothing?

## Decision

What we will do, stated imperatively.

## Consequences

Positive outcomes, negative outcomes, and risks. List downstream docs, scripts, workflows, and invariants affected. Explicitly state what becomes harder.

## Enforcement

How will we know this decision is being followed? Link to lint rule, test name, workflow, or checklist item.

## Links

ARCHITECTURE.md §X, INVARIANTS.md INV-0Y, related ADRs, issues, and PRs.
```

### 5.2 Post-Mortem Rule

If an accepted invariant is violated in `main` or a released build, append a post-mortem section to the ADR that created or last changed the violated boundary.

Allowed post-accepted mutation:

```markdown
## Post-mortem YYYY-MM-DD

What happened?
What boundary was breached?
Why did enforcement miss it?
What enforcement was added?
What regression test now prevents recurrence?
```

---

## 6. Invariant Governance

An invariant is a property that must hold regardless of implementation detail. Violating an accepted invariant is a bug by definition, not a style disagreement.

Rules:

1. Invariant definitions live in `INVARIANTS.md`, not in this document.
2. An invariant does not exist as binding policy unless it has documented local and GitHub Actions enforcement.
3. No invariant may be marked `Accepted` unless the enforcement exists or is added in the same PR.
4. Weakening, deleting, or rewriting an invariant requires an ADR.
5. Every invariant must name the architecture section it protects.
6. Every invariant must name what it prevents in plain language.

### 6.1 Invariant Status

- **Proposed:** intended protection, not binding yet. Code must not rely on it as enforced.
- **Accepted:** binding. Must have local and GitHub Actions enforcement.
- **Retired:** no longer binding. Must link to the ADR or invariant that replaced it.

### 6.2 Invariant Violation Response

Any accepted invariant violation in `main` or a release is a P0 critical bug.

Response policy:

1. Halt feature merges touching the affected boundary.
2. Fix forward in a hotfix PR.
3. Add or repair the missing enforcement.
4. Add a regression test that fails if the violation recurs.
5. Add a post-mortem to the relevant ADR within 48 hours.

A violation caught in an unmerged PR is normal review feedback, not a P0. If enforcement was missing, the enforcement must be added before merge.

---

## 7. Contract Document Lifecycle

Downstream contract docs define exact behavior after the architecture stabilizes.

Examples:

- command contracts
- error enum contracts
- citation node schema
- reference record schema
- document envelope schema
- Python helper worker contract
- formatting rule contract
- text-analysis issue model
- network client contract
- export contract

### 7.1 States

1. **Draft:** stored in `docs/drafts/` or PR description. Free-form and non-binding. No invariant may reference it.
2. **Proposed:** PR opened with label `contract-doc`. Must include purpose, scope, normative requirements, non-goals, invariants upheld, and enforcement.
3. **Accepted:** merged to `docs/contracts/*.md`. Binding. Code may rely on it.
4. **Superseded or Retired:** kept with a banner link to successor. Never delete accepted contracts because they preserve decision history.

### 7.2 Stability Definition

A section of `ARCHITECTURE.md` is stable when:

1. Its open questions are resolved by an accepted ADR or accepted contract.
2. Its relevant invariants have local and GitHub Actions enforcement.
3. No breaking changes have been proposed against it for two weeks.

Only then may a contract doc for that area be proposed as binding.

### 7.3 Required Frontmatter

Every accepted contract doc must start with frontmatter:

```yaml
---
status: Accepted
adr: ADR-005
upholds: [INV-03, INV-10]
owners: [core, frontend]
---
```

---

## 8. Build and Verification Policy

DRAFT supports both local development builds and GitHub Actions builds.

The local and CI paths must use the same underlying scripts where practical.

Required local commands:

```bash
just format
just verify
just check-invariants
just build
```

Required GitHub Actions workflows:

```text
.github/workflows/verify.yml
.github/workflows/invariants.yml
.github/workflows/build.yml
```

Required job names:

```text
verify
invariants
build
```

Phase 3 established this target shape with the aggregate `verify` job in
`.github/workflows/verify.yml`. That job runs all current build, test,
documentation, and invariant checks through `scripts/verify.sh`. Dedicated
`invariants` and `build` workflows remain required as those surfaces mature;
the baseline workflow does not make them complete by implication.

The `justfile` is the stable command surface. It may call Cargo, the JavaScript package manager, Tauri CLI, Python tools, or Bash scripts underneath. Developers should not need to remember every low-level command.

### 8.1 Required Verification Coverage

Local and CI verification must cover:

- Rust format, lint, tests, and security-relevant boundary checks.
- TypeScript format, lint, typecheck, and tests.
- React/Tiptap editor tests for schema-sensitive behavior.
- Tauri 2 build verification.
- Python helper formatting, lint, tests, and dependency checks.
- Bash `shellcheck`, `shfmt`, and script smoke tests.
- Invariant enforcement scripts from `INVARIANTS.md`.

### 8.2 Formatting Policy

Formatting is both a product capability and a repository maintenance concern. Keep them separate.

Repository formatting:

- Rust formatting uses `cargo fmt`.
- TypeScript/React formatting uses the selected JS formatter through `just format`.
- Python formatting uses the selected Python formatter through `just format`.
- Bash formatting uses `shfmt`.

Product formatting:

- Document formatting behavior must be governed by architecture, invariants, and accepted contracts.
- Product formatting must not be implemented as arbitrary Bash execution.
- Python may support product formatting only through the helper-worker boundary described in `ARCHITECTURE.md` and `INVARIANTS.md`.

---

## 9. Standing Policy: External-Service Boundary

DRAFT does not build automated access against services that do not offer a stable, documented API for the access pattern.

Google Scholar is the motivating example. The same rule applies to publisher sites, institutional portals, and research databases.

This is not evaluated case by case per feature. The failure modes are the same: silent breakage, account lockout, IP blocking, terms-of-service exposure, credential exposure, and brittle behavior.

Allowed:

- Querying documented APIs.
- Opening a URL in the user's system browser.
- Letting the user authenticate outside DRAFT.
- Letting the user import a downloaded PDF.

Not allowed:

- Scraping browser sessions.
- Automating institutional login.
- Proxying credentials.
- Intercepting cookies or tokens.
- Storing publisher or institutional credentials.
- Quiet feature-level exceptions.

Violation of this policy requires an ADR that explicitly overturns or narrows this section.

---

## 10. Architecture PR Checklist

Every architecture PR must answer:

```markdown
## Architecture Change Checklist

- [ ] Current decision stated.
- [ ] Trigger for reconsideration stated.
- [ ] ADR added or editorial-only rationale stated.
- [ ] Downstream contracts checked.
- [ ] Affected invariants listed.
- [ ] Local enforcement added or updated.
- [ ] GitHub Actions enforcement added or updated.
- [ ] `just verify` passes locally.
- [ ] `just check-invariants` passes locally.
- [ ] What becomes harder is stated.
- [ ] Rollback or supersession path is stated.
```

---

## 11. Open Items

- Add `.github/pull_request_template.md` with the architecture checklist.
- Add `CODEOWNERS` for governed files once team size is greater than one.
- Add dedicated `invariants.yml` and `build.yml` workflows as their full
  enforcement surfaces mature. Phase 3 provides the aggregate `verify.yml`
  baseline.
- Decide and pin the Python dependency manager before adding a third-party
  helper dependency.
- Keep the implemented document-envelope, reference-record, reference-store,
  citation-node, bibliography-consistency, network-client, metadata-lookup,
  external-browser-handoff, and PDF-import requirements non-binding until the
  contract lifecycle permits acceptance. The implemented Phase 26
  background-job requirements and Phase 27 AI-orchestration requirements are
  also non-binding. The implemented Phase 28 Python-helper requirements remain
  non-binding as well. Create formatting drafts in their owning phases.
