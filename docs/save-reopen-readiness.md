# [PLANNED - Implementation Pending] Save/reopen readiness

This page describes the intended save/reopen readiness contract for the
persistence feature that will be built. It is not current proof that the
runnable first-lesson readiness flow records `project-reopen/` evidence or a
`reopen-project` UI action.

Use this page when designing or reviewing the planned persistence lane:

> Did the harness save a changed `.a3p`, reopen that saved artifact instead of
> the bundled starter project, and record reopened-state evidence for review?

The answer is only trusted after the planned integration records that evidence
in the normal readiness outputs. Until then, the current runnable readiness
boundary remains save-project evidence plus explicit non-claims.

## Contents

- [Current implemented boundary](#current-implemented-boundary)
- [Planned evidence boundary](#planned-evidence-boundary)
- [Select Project proof vs reopen artifact proof](#select-project-proof-vs-reopen-artifact-proof)
- [Planned integration points](#planned-integration-points)
- [Planned hook API](#planned-hook-api)
- [Planned readiness output](#planned-readiness-output)
- [Review checklist](#review-checklist)
- [Non-claims](#non-claims)

## Current implemented boundary

The existing first-lesson readiness flow can collect and validate
`save-project` proof. It does not yet wire `reopen-project` into the runnable
readiness sequence.

| Current surface | Implemented meaning |
| --- | --- |
| `alice run-first-lesson-readiness` | Runs the first-lesson readiness comparison and writes the comparison manifest for the selected run. |
| `ui-action-contract.json` | Includes required user-like actions through `save-project`; it does not include `reopen-project`. |
| Required action validation | Requires `save-project` proof or a save no-go after run-world proof; it does not require reopen proof. |
| Readiness progress | Reports missing proof through "Save Project proof artifact" and "Select Project proof artifact"; it does not report a reopened-state proof artifact. |
| `probe_project_reopen_hook` | Exists as an internal probe/test contract for the future reopen lane, but is not emitted by the normal readiness flow. |

Run commands from the repository root when checking the current implemented
boundary:

```bash
cargo run -q -p eatme-cli -- assets validate --json
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
```

When Alice targets are available, the current first-lesson readiness command is:

```bash
export NODE_OPTIONS=--max-old-space-size=32768
export ALICE_BASELINE_HOME=/opt/alice/original-alice
export ALICE_MODERNIZED_HOME=/opt/alice/rabbithole-alice

EATME_REAL_ALICE=1 cargo run -q -p eatme-cli -- alice run-first-lesson-readiness \
  --run-id local-first-lesson-readiness \
  --json \
  --no-memory \
  --offline-package \
  --execute
```

Do not treat that command as save/reopen completion proof until the planned
`reopen-project` integration points below are implemented.

## Planned evidence boundary

The planned save/reopen lane is layered on top of the starter-project and
first-lesson evidence contracts.

| Layer | What it should prove | What it must not imply |
| --- | --- | --- |
| Starter-project preflight | The bundled starter project can be opened and inspected with launch evidence. | Save, reopen, export, full UI automation, or lesson completion. |
| Save artifact proof | A deterministic save affordance produced a non-empty saved `.a3p` and save evidence for the current run. | Full Save completion, grading, creative assessment, or first-lesson completion. |
| Reopen artifact proof | A deterministic reopen affordance reopened the saved `.a3p`, not the bundled starter project, and produced non-empty reopen evidence. | Visible rendering correctness, broad Alice compatibility, or creative quality. |
| Reopened-state proof | The reopen affordance produced state evidence and marked state verification as passed for the bounded selector. | Learner-world grading, instructor assessment, or complete lesson success. |

The planned sequence is:

1. Open the bundled starter project through the existing Alice launch path.
2. Reach the first-lesson run proof needed before persistence review.
3. Save the edited project through a deterministic save affordance.
4. Record a non-empty saved `.a3p` and save evidence under `project-save/`.
5. Reopen that saved artifact in a new or explicitly reset Alice session.
6. Record non-empty reopen evidence and reopened-state evidence under
   `project-reopen/`.
7. Report unsupported or missing affordances as `blocked`, not as success.

Each step depends on the previous step. Reopen proof is blocked until accepted
save artifact proof exists. Save proof is blocked until run-world proof exists.

## Select Project proof vs reopen artifact proof

Existing readiness language includes "Select Project proof artifact". That is
not the same boundary as planned reopen proof.

| Evidence name | Boundary |
| --- | --- |
| Select Project proof artifact | Shows that a project selection/open path produced evidence for the current Alice readiness flow. It does not prove the source project came from a saved artifact. |
| Save Project proof artifact | Shows the save affordance produced a non-empty `.a3p` and save evidence for the current run. |
| Reopen artifact proof | Planned evidence that Alice opened the saved `.a3p` from `project-save/`, not the bundled starter project or another source. |
| Reopened-state proof | Planned state evidence from the reopened saved project, with bounded verification for the selected learner-world state. |

Reviewers should not collapse these boundaries. A select/open proof can be a
precondition for readiness, but it does not replace `source_saved_project_artifact`
or `reopened_state_artifact` evidence.

## Planned integration points

The save/reopen feature is ready only after the implementation wires the reopen
contract into the same surfaces that already enforce save proof.

| Surface to update | Required planned behavior |
| --- | --- |
| First-lesson readiness sequence | Invoke the reopen probe after accepted `save-project` proof. |
| `ui-action-contract.json` | Add a `reopen-project` required action with ready/no-go evidence. |
| Required action validation | Require passed reopen proof or an explicit reopen no-go once save proof passes. |
| Readiness progress | Report missing reopen proof after save proof, without claiming first-lesson completion. |
| No-go reporting | Explain missing reopen affordance, missing saved artifact, and failed state verification distinctly. |
| Contract tests | Cover passed reopen evidence, blocked reopen preconditions, unsafe artifact paths, and missing or empty reopened-state artifacts. |

The existing internal reopen probe contract can be reused, but the feature is
not complete until normal readiness manifests and validation consume its output.

## Planned hook API

The persistence hooks are Alice-side contracts. Eatme invokes them and validates
their JSON and artifacts. Hook paths are relative to the Alice checkout.

### Save hook

Command shape:

```bash
tools/eatme-save-project \
  --project runs/first-lessons-real-ui-actions/local-save-reopen-readiness/procedure-edit/edited-project.a3p \
  --save-selector scene.eatmeFirstLessonStep \
  --evidence-dir runs/first-lessons-real-ui-actions/local-save-reopen-readiness/project-save \
  --json
```

The hook prints:

```json
{
  "schema_version": "eatme.alice-project-save-result/v1",
  "status": "saved",
  "save_selector": "scene.eatmeFirstLessonStep",
  "saved_project_artifact": "saved-project.a3p",
  "save_artifact": "project-save.json"
}
```

Validation rules:

| Field | Rule |
| --- | --- |
| `schema_version` | Must be `eatme.alice-project-save-result/v1`. |
| `status` | Must be `saved`. |
| `save_selector` | Must be `scene.eatmeFirstLessonStep`. |
| `saved_project_artifact` | Must be a simple relative path under the `project-save/` evidence directory and must point to a non-empty file. |
| `save_artifact` | Must be a simple relative path under the `project-save/` evidence directory and must point to a non-empty file. |

### Reopen hook

Command shape:

```bash
tools/eatme-reopen-project \
  --saved-project runs/first-lessons-real-ui-actions/local-save-reopen-readiness/project-save/saved-project.a3p \
  --reopen-selector scene.eatmeFirstLessonStep \
  --evidence-dir runs/first-lessons-real-ui-actions/local-save-reopen-readiness/project-reopen \
  --json
```

The hook prints:

```json
{
  "schema_version": "eatme.alice-project-reopen-result/v1",
  "status": "reopened",
  "source_saved_project_artifact": "project-save/saved-project.a3p",
  "reopen_selector": "scene.eatmeFirstLessonStep",
  "reopened_project_artifact": "reopened-project.a3p",
  "reopen_artifact": "project-reopen.json",
  "reopened_state_artifact": "reopened-state.json",
  "state_verification": "passed"
}
```

Validation rules:

| Field | Rule |
| --- | --- |
| `schema_version` | Must be `eatme.alice-project-reopen-result/v1`. |
| `status` | Must be `reopened`. |
| `source_saved_project_artifact` | Must be a simple relative path under `project-save/` and must point to the saved artifact from the same run. It must not point to the bundled starter project. |
| `reopen_selector` | Must be `scene.eatmeFirstLessonStep`. |
| `reopened_project_artifact` | Must be a simple relative path under the `project-reopen/` evidence directory and must point to a non-empty file. |
| `reopen_artifact` | Must be a simple relative path under the `project-reopen/` evidence directory and must point to a non-empty file. |
| `reopened_state_artifact` | Must be a simple relative path under the `project-reopen/` evidence directory and must point to a non-empty file. |
| `state_verification` | Must be `passed`. |

Absolute paths, parent traversal, symlink escapes, empty files, malformed JSON,
wrong schema versions, and artifacts outside the expected run evidence
directories are not accepted as proof.

## Planned readiness output

Save/reopen readiness should use explicit states:

| State | Meaning |
| --- | --- |
| `passed` | The named hook ran, returned accepted JSON, and produced all required non-empty artifacts. |
| `blocked` | A required earlier proof or deterministic Alice affordance is missing. The run is honest but not ready for that boundary. |
| `failed` | A hook ran or returned data, but the command, JSON, artifact, or validation result did not satisfy the contract. |

The planned UI action contract records persistence facts with stable action ids:

| Action id | Evidence |
| --- | --- |
| `save-project` | Save artifact proof or a blocked save precondition. |
| `reopen-project` | Reopen artifact proof or a blocked reopen precondition. |

Safe user-facing wording after the planned implementation:

```text
Save artifact proof is shown for this run.
Reopen artifact proof is shown for the saved artifact from this run.
Save completion is not claimed as full UI completion.
First-lesson completion is not proven.
```

## Review checklist

Use this checklist when implementing or reviewing the planned lane:

1. Validate assets with `cargo run -q -p eatme-cli -- assets validate --json`.
2. Check generated adapters with
   `cargo run -q -p eatme-cli -- assets generate-gadugi --check --json`.
3. Confirm the normal first-lesson readiness manifest records `save-project`.
4. Confirm the normal first-lesson readiness manifest records `reopen-project`
   only after accepted save proof.
5. Confirm `project-reopen/` contains non-empty reopen evidence and
   `reopened-state.json`.
6. Confirm `source_saved_project_artifact` starts with `project-save/`.
7. Confirm the report keeps full UI automation, visible rendering correctness,
   grading, creative assessment, full Save completion, and first-lesson
   completion unproven unless exact separate evidence exists.

## Non-claims

This readiness contract does not claim:

- full Save completion
- full reopen completion beyond the bounded artifact/state contract
- export completion
- full UI automation
- first-lesson completion
- visible rendering correctness
- creative assessment, learner-world grading, or complete Alice coverage

Unsafe wording:

```text
The lesson is complete.
Rendering is correct.
The saved world was graded.
Alice UI automation is complete.
```

Do not fix missing reopen proof by editing generated Gadugi adapters or by
changing documentation wording. Fix the canonical evidence source, rerun
validation, and regenerate adapters only from canonical assets when asset
changes require it.
