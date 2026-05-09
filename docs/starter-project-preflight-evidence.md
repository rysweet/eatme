# Starter project preflight evidence

This readiness report captures bounded preflight evidence for opening the
bundled starter project before save, reopen, or export review. It clarifies the
evidence boundary for `starter-project-open-save-export-preflight`; it is not
implementation proof for Save, reopen, export, first-lesson completion, or full
UI automation.

## Evidence source

The supporting evidence source is the existing scenario:

```text
assets/scenarios/eatme/starter-project-open-save-export-preflight.yaml
```

The scenario is referenced here as context only. This report does not require
editing the scenario, regenerating the Gadugi adapter, or changing Rust or UI
automation code.

## What this readiness report demonstrates

The preflight evidence demonstrates that the eatme harness can launch real Alice
with the bundled starter project and record inspectable evidence for the
opened-project state.

| Evidence | Readiness meaning |
| --- | --- |
| Launch manifest | Identifies `starter-project-open-save-export-preflight` as the selected scenario. |
| Launch command | Shows Alice was started with the bundled starter project, such as `africa.a3p`. |
| Assertions | Records deterministic harness assertions, including real Alice execution evidence. |
| Window or screenshot evidence | Shows a smoke-ready Alice desktop session was observed. |
| Logs | Preserve Alice launch output for review and troubleshooting. |
| Inspectable launch-smoke outputs | Provide setup evidence for later save, reopen, export, or action-contract review. |

This evidence boundary is intentionally narrow. It supports readiness to review
the starter-project lane, but it does not prove that any save, reopen, or export
workflow has completed successfully.

## Save/reopen readiness gap

The remaining save/reopen gap is an unproven workflow boundary, not a failed
implementation. The available preflight evidence shows that the bundled starter
project can be opened and inspected before further review. It does not prove
that a changed project can be saved, the saved file can be reopened, or the
reopened state matches the expected learner-world state.

Closing this gap needs separate evidence that exercises the save path, records
the saved artifact, reopens that artifact, and verifies the reopened project
state. Until that evidence exists, this report should be read only as
starter-project open-readiness evidence.

## Explicit non-claims

This readiness report does not claim:

- full Save completion
- end-to-end reopen verification
- export completion
- full UI automation
- first-lesson completion
- creative assessment, learner-world grading, or complete Alice coverage
- `ui-action-contract.json` generation

`ui-action-contract.json` belongs to scenarios that explicitly exercise or
specify user-like UI actions, such as `first-lessons-real-ui-actions`. The
starter-project preflight lane remains bounded to opening the bundled starter
project and preserving inspectable readiness evidence.

## Next evidence needed

The next evidence should be a separate save/reopen/export review that proves the
workflow beyond preflight. That evidence should show the save action, identify
the saved project artifact, reopen that artifact, verify the reopened state, and
state its own evidence boundary without relying on this preflight report as
proof of workflow completion.
