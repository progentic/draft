# DRAFT v1 Usability Evidence

## Tested Artifact

- Commit: `154c34c96183ff67d4ecd6acd790b0410403dd58`
- Packaged application: unsigned macOS Apple Silicon `DRAFT.app`
- Executable SHA-256: `e4ab1618764363b066706353149b8bbf700111a6f0a44fbce66ebc02cd4687f0`
- Identity result: the manually tested executable matched the recorded Phase 46 artifact.
- Session date: 2026-07-11

Artifact provenance does not prove workflow usability. Every release-candidate
row remains governed by its own closure evidence.

## Phase 46

### Automated Evidence

- Local `scripts/verify.sh` passed on commit `154c34c` with 370 Rust tests, 252
  frontend tests, and 11 Python tests.
- GitHub Actions Verify passed on commit `154c34c` in PR #36.
- `scripts/package-macos.sh` built the unsigned Apple Silicon application and
  executed the embedded deterministic text-analysis helper successfully.
- Automated evidence does not replace the incomplete packaged human workflow.

### Packaged Application Evidence

| Area | Result | Evidence |
| :--- | :--- | :--- |
| Artifact identity | Pass | Commit and executable hash matched the intended `154c34c` package. |
| Launch | Pass | The packaged application opened for direct manual interaction. |
| Document lifecycle | Fail | Invoking Save made DRAFT unresponsive, displayed the macOS beach ball, and required force-quit. Save did not complete and no recovery action was available. Create and New could not be validated reliably in that session. |
| Formatting controls | Partial | Existing formatting controls were visible, but no font-family or font-size control was available. |
| References and citations | Pending | Not reached after the lifecycle failure. |
| Five local text checks | Pending | Not reached after the lifecycle failure. |
| Finding navigation and focus | Pending | Not reached after the lifecycle failure. |
| DOCX export and source integrity | Pending | Not reached after the lifecycle failure. |
| Recovery and keyboard accessibility | Fail | The unresponsive Save state offered no recovery and required force-quit; remaining keyboard checks are pending. |

No permissions or application-trust cause was established.

The implementation uses blocking native dialog APIs from synchronous Tauri
commands for Open, Save, and Export. The installed dialog API documentation
states that blocking dialog APIs must not run on the main thread. This invalid
execution pattern is recorded separately from the manual observation and is
the defect mechanism to correct.

### Human Task Results

The repository owner directly tested the exact packaged artifact. The session
stopped after Save caused an unrecoverable beach ball and force-quit. No
untested task is counted as passed.

### Findings And Dispositions

| ID | Severity | Status | Evidence | Disposition |
| :--- | :--- | :--- | :--- | :--- |
| UX-46-001 | UX-0 | Open | During manual validation of packaged artifact `154c34c`, invoking Save caused DRAFT to become unresponsive and display the macOS beach ball. Save did not complete, no recovery action was available, and the application required force-quit. | Replace the synchronous blocking dialog path with async Tauri commands and non-blocking Rust-owned dialog callbacks, then rebuild, rehash, and rerun the complete packaged workflow. |
| UX-46-002 | UX-2 | Open | The packaged editor exposed no font-family control. | Add a bounded family control only if the DRAFT envelope persists it, reopen restores it, and DOCX preserves it accurately. Rebuild, rehash, and validate the corrected package before disposition. |
| UX-46-003 | UX-2 | Open | The packaged editor exposed no font-size control. | Add a bounded point-size control only if the DRAFT envelope persists it, reopen restores it, and DOCX preserves it accurately. Rebuild, rehash, and validate the corrected package before disposition. |

`RC-01` through `RC-04` and `GATE-46` remain open.
