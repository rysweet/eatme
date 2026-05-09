# Starter project preflight evidence

This readiness report captures bounded preflight evidence for opening the
bundled starter project before save, reopen, or export review. It describes the
bounded evidence contract for `starter-project-open-save-export-preflight` and
the feature boundary for the persistence work that still needs to be built:
starter-project launch readiness is documented, and workflow completion beyond
that launch boundary remains explicitly unproven.

This report is not implementation proof for Save, reopen, export, first-lesson
completion, or full UI automation. It is the handoff boundary for a later
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
| Starter-project launch readiness evidence | Save implementation or Save completion evidence |
| Inspectable preflight outputs from the existing scenario | Reopen implementation or end-to-end reopen verification |
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
that the reopened state matches the expected learner-world state.

Closing this gap needs a separate persistence feature and evidence lane that
exercises the save path, records the saved artifact, reopens that artifact, and
verifies the reopened project state. Until that feature and evidence exist, this
report should be read only as starter-project open-readiness evidence.

The feature to build is a deterministic save/reopen workflow for the bundled
starter project. Its implementation-ready boundary is:

1. open the bundled starter project;
2. make or identify a deterministic, reviewer-visible save-worthy project state;
3. save the project and record the saved artifact path, size, and run metadata;
4. reopen that saved artifact in a new or explicitly reset Alice session;
5. verify the reopened project state against the expected learner-world state;
6. report the save/reopen evidence in its own manifest or report, separate from
   this preflight evidence.

Only the first item is supported by this preflight report. Items 2 through 6
need their own scenario evidence and should not inherit completion claims from
this document.

Export can be added as a follow-on acceptance path, but it should not be implied
by save/reopen success. If export is included in the future feature, it needs a
separate exported artifact, artifact verification, and evidence boundary.

## Acceptance contract for the feature to build

The future save/reopen lane is ready to trust only when its own evidence proves
each persistence step without borrowing claims from this preflight report.

| Future evidence | Required meaning |
| --- | --- |
| Save action evidence | Shows the workflow invoked Alice's save path after opening the starter project. |
| Saved artifact evidence | Identifies the saved `.a3p` artifact and records that it exists, is non-empty, and belongs to the current run. |
| Reopen action evidence | Shows Alice reopened the saved artifact, not the original bundled starter project. |
| Reopened-state verification | Confirms the reopened project matches the deterministic expected learner-world state. |
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

## Next evidence needed

The next evidence should be a separate save/reopen review that proves the
persistence workflow beyond preflight. That evidence should show the save
action, identify the saved project artifact, reopen that artifact, verify the
reopened state, and state its own evidence boundary without relying on this
preflight report as proof of workflow completion. If export is included in the
same future lane, it should add its own exported artifact verification instead
of treating save/reopen success as export proof.
