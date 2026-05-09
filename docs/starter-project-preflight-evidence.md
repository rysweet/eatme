# Starter project preflight evidence

This readiness report captures bounded preflight evidence for opening the
bundled starter project before save, reopen, or export review. It describes the
bounded evidence contract for `starter-project-open-save-export-preflight`.
Starter-project launch readiness is documented here, while the separate
save/reopen artifact and reopened-state contract is documented in
[Save/reopen Readiness](save-reopen-readiness.md). Workflow completion beyond
the launch boundary remains explicitly unproven unless that separate evidence
exists.

This report is not implementation proof for Save, reopen, export, first-lesson
completion, or full UI automation. It is the handoff boundary for a separate
save/reopen evidence lane.

## Evidence source and scope

The supporting evidence source is the existing scenario:

```text
assets/scenarios/eatme/starter-project-open-save-export-preflight.yaml
```

The scenario is referenced here as context only. This report does not require
editing the scenario, regenerating the Gadugi adapter, or changing Rust or UI
automation code.

| In scope for this report | Out of scope for this report |
| --- | --- |
| Starter-project launch readiness evidence | Save artifact proof or full Save completion evidence |
| Inspectable preflight outputs from the existing scenario | Reopen artifact proof or reopened-state verification |
| Boundary wording for later save/reopen/export review | Export implementation or export completion evidence |
| Documentation-only clarification of the evidence contract | Scenario, adapter, Rust, or UI automation changes |

## What this readiness report demonstrates

The preflight evidence demonstrates that the eatme harness can launch real Alice
with the bundled starter project and record inspectable evidence for the
opened-project state. The demonstrated state is "opened and inspectable"; it is
not "saved", "reopened", "exported", or "lesson-complete".

| Evidence | Readiness meaning |
| --- | --- |
| Launch manifest | Identifies `starter-project-open-save-export-preflight` as the selected scenario. |
| Launch command | Shows Alice was started with the bundled starter project, such as `africa.a3p`. |
| Assertions | Records deterministic harness assertions, including real Alice execution evidence. |
| Window or screenshot evidence | Shows a smoke-ready Alice desktop session was observed. |
| Logs | Preserve Alice launch output for review and troubleshooting. |
| Inspectable launch-smoke outputs | Provide setup evidence for later save, reopen, export, or action-contract review. |

This evidence boundary is intentionally narrow. It supports readiness to review
the starter-project lane and prepare later persistence checks, but it does not
prove that any save, reopen, or export workflow has completed successfully.

## Save/reopen readiness gap

The remaining save/reopen gap is an unproven workflow boundary, not a failed
implementation. The available preflight evidence shows that the bundled starter
project can be opened and inspected before further review. It does not prove
that a changed project can be saved, that the saved file can be reopened, or
that reopened-state verification evidence exists.

Closing this gap uses a separate persistence evidence lane that exercises the
save path, records the saved artifact, reopens that artifact, and records
reopened-state evidence reported by the reopen step. Until that separate
evidence exists for a run, this report should be read only as starter-project
open-readiness evidence.

The deterministic save/reopen boundary is:

1. open the bundled starter project;
2. make or identify a deterministic, reviewer-visible save-worthy project state;
3. save the project and record the saved artifact path, size, and run metadata;
4. reopen that saved artifact in a new or explicitly reset Alice session;
5. record reopened-state verification and non-empty state evidence from the reopen step;
6. report the save/reopen evidence in its own manifest or report, separate from
   this preflight evidence.

Only the first item is supported by this preflight report. Items 2 through 6 are
specified by [Save/reopen Readiness](save-reopen-readiness.md) and should not
inherit completion claims from this document.

Export can be added as a follow-on acceptance path, but it should not be implied
by save/reopen success. If export is included in the same lane, it needs a
separate exported artifact, artifact verification, and evidence boundary.

## Acceptance contract for the save/reopen lane

The save/reopen lane is ready to trust only when its own evidence proves
each persistence step without borrowing claims from this preflight report.

| Save/reopen evidence | Required meaning |
| --- | --- |
| Save proof | Shows the workflow invoked Alice's save path after opening the starter project. |
| Saved artifact evidence | Identifies the saved `.a3p` artifact and records that it exists, is non-empty, and belongs to the current run. |
| Reopen proof | Shows Alice reopened the saved artifact, not the original bundled starter project. |
| Reopened-state verification | Records `state_verification: passed` from the reopen hook and preserves non-empty reopened-state evidence for review. |
| Separate manifest or report | Keeps persistence evidence distinct from starter-project launch preflight evidence. |
| Optional export evidence | If export is part of the scenario, identifies and verifies the exported artifact separately from save/reopen. |

## Explicit non-claims

This readiness report does not claim:

- full Save completion
- end-to-end reopen verification
- export completion
- full UI automation
- first-lesson completion
- creative assessment, learner-world grading, or complete Alice coverage
- `ui-action-contract.json` generation
- persistence durability across sessions

`ui-action-contract.json` belongs to scenarios that explicitly exercise or
specify user-like UI actions, such as `first-lessons-real-ui-actions`. The
starter-project preflight lane remains bounded to opening the bundled starter
project and preserving inspectable readiness evidence.

References to existing scenarios are supporting context only. They do not expand
this report into full UI automation, Save/reopen execution, export verification,
or first-lesson completion evidence.

## Evidence beyond preflight

Evidence beyond preflight belongs in a separate save/reopen review that proves
the persistence workflow beyond this launch boundary. That evidence should show
the save action, identify the saved project artifact, reopen that artifact,
record reopened-state verification evidence, and state its own evidence boundary
without relying on this preflight report as proof of workflow completion. If
export is included in the same lane, it should add its own exported artifact
verification instead of treating save/reopen success as export proof.
