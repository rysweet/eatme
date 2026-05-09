# Save/reopen readiness

Save/reopen readiness is the bounded evidence contract for recording that a
save hook produced a changed Alice project artifact, a reopen hook used that
saved artifact, and reopened-state evidence was captured for review. It does
not treat those artifacts as full UI automation or lesson completion.

Use this page to review one question:

> Did the harness save a changed `.a3p`, reopen that saved artifact instead of
> the bundled starter project, and record reopened-state evidence for review?

The answer is trusted only when the run evidence records the save proof, reopen
proof, and reopened-state proof described below. A starter-project launch smoke
or opened-project preflight does not prove save/reopen readiness by itself.

## Contents

- [Evidence boundary](#evidence-boundary)
- [Usage](#usage)
- [Configuration](#configuration)
- [Evidence directories](#evidence-directories)
- [Hook API](#hook-api)
- [Readiness states](#readiness-states)
- [Review checklist](#review-checklist)
- [Non-claims](#non-claims)

## Evidence boundary

Save/reopen readiness is layered on top of the starter-project and first-lesson
evidence contracts.

| Layer | What it proves | What it does not imply |
| --- | --- | --- |
| Starter-project preflight | The bundled starter project can be opened and inspected with launch evidence. | Save, reopen, export, full UI automation, or lesson completion. |
| Save artifact proof | A deterministic save affordance produced a non-empty saved `.a3p` and save evidence for the current run. | Full Save completion, grading, creative assessment, or first-lesson completion. |
| Reopen artifact proof | A deterministic reopen affordance reopened the saved `.a3p`, not the bundled starter project, and produced non-empty reopen evidence. | Visible rendering correctness, broad Alice compatibility, or creative quality. |
| Reopened-state proof | The reopen affordance produced state evidence and marked state verification as passed for the bounded selector. | Learner-world grading, instructor assessment, or complete lesson success. |

The readiness sequence is:

1. Open the bundled starter project through the Alice launch path.
2. Reach the run-world proof needed before persistence review.
3. Save the edited project through a deterministic save affordance.
4. Record a non-empty saved `.a3p` and save evidence under `project-save/`.
5. Reopen that saved artifact in a new or explicitly reset Alice session.
6. Record non-empty reopen evidence and reopened-state evidence under
   `project-reopen/`.
7. Report unsupported or missing affordances as `blocked`, not as success.

Each step depends on the previous step. Reopen proof is blocked until accepted
save artifact proof exists. Save proof is blocked until run-world proof exists.

## Usage

Run commands from the repository root.

Validate the repository assets before trusting readiness evidence:

```bash
cargo run -q -p eatme-cli -- assets validate --json
```

Check generated Gadugi adapters when scenario assets changed:

```bash
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
```

Run first-lesson readiness only when Alice targets are available:

```bash
export NODE_OPTIONS=--max-old-space-size=32768
export ALICE_BASELINE_HOME=/opt/alice/original-alice
export ALICE_MODERNIZED_HOME=/opt/alice/rabbithole-alice

EATME_REAL_ALICE=1 cargo run -q -p eatme-cli -- alice run-first-lesson-readiness \
  --run-id local-save-reopen-readiness \
  --json \
  --no-memory \
  --offline-package \
  --execute
```

Read the resulting manifest, `ui-action-contract.json`, and any `project-save/`
or `project-reopen/` evidence as review inputs. Treat `save-project` and
`reopen-project` as ready only when their proof artifacts are present,
non-empty, and accepted by validation. Treat a missing hook or missing
precondition as a bounded `blocked` result.

## Configuration

The save/reopen lane uses deterministic Alice-side hooks when they are available
in the selected Alice checkout.

| Setting | Value |
| --- | --- |
| Save hook | `tools/eatme-save-project` |
| Reopen hook | `tools/eatme-reopen-project` |
| Selector | `scene.eatmeFirstLessonStep` |
| Save evidence directory | `project-save/` under the run directory |
| Reopen evidence directory | `project-reopen/` under the run directory |
| Hook timeout | 30 seconds |
| Display | The active Alice display passed as `DISPLAY` |

The hook paths are relative to the Alice checkout. Do not pass absolute hook
paths from local machines in documentation, manifests, or comments.

## Evidence directories

A passing bounded save/reopen run records evidence under the selected run
directory:

```text
runs/first-lessons-real-ui-actions/local-save-reopen-readiness/
├── procedure-edit/
│   └── edited-project.a3p
├── project-save/
│   ├── saved-project.a3p
│   └── project-save.json
├── project-reopen/
│   ├── reopened-project.a3p
│   ├── project-reopen.json
│   └── reopened-state.json
└── ui-action-contract.json
```

`project-save/saved-project.a3p` is the source for reopen proof. A reopen result
that points back to the bundled starter project is rejected.

## Hook API

The persistence hooks are Alice-side contracts. Eatme invokes them and validates
their JSON and artifacts.

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
| `saved_project_artifact` | Must be a simple relative path under `project-save/` and must point to a non-empty file. |
| `save_artifact` | Must be a simple relative path under `project-save/` and must point to a non-empty file. |

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
| `source_saved_project_artifact` | Must be a simple relative path under `project-save/`, must point to the saved artifact from the same run, and must not point to the bundled starter project. |
| `reopen_selector` | Must be `scene.eatmeFirstLessonStep`. |
| `reopened_project_artifact` | Must be a simple relative path under `project-reopen/` and must point to a non-empty file. |
| `reopen_artifact` | Must be a simple relative path under `project-reopen/` and must point to a non-empty file. |
| `reopened_state_artifact` | Must be a simple relative path under `project-reopen/` and must point to a non-empty file. |
| `state_verification` | Must be `passed`. |

Absolute paths, parent traversal, symlink escapes, empty files, malformed JSON,
wrong schema versions, and artifacts outside the expected run evidence
directories are not accepted as proof.

## Readiness states

Save/reopen readiness uses explicit states:

| State | Meaning |
| --- | --- |
| `passed` | The named hook ran, returned accepted JSON, and produced all required non-empty artifacts. |
| `blocked` | A required earlier proof or deterministic Alice affordance is missing. The run is honest but not ready for that boundary. |
| `failed` | A hook ran or returned data, but the command, JSON, artifact, or validation result did not satisfy the contract. |

Safe user-facing wording:

```text
Save artifact proof is shown for this run.
Reopen artifact proof is shown for the saved artifact from this run.
Save completion is not claimed as full UI completion.
First-lesson completion is not proven.
```

## Review checklist

Use this checklist when reviewing save/reopen readiness:

1. Validate assets with `cargo run -q -p eatme-cli -- assets validate --json`.
2. Check generated adapters with
   `cargo run -q -p eatme-cli -- assets generate-gadugi --check --json` when
   scenario assets changed.
3. Confirm `save-project` appears only after accepted run-world proof.
4. Confirm any `reopen-project` probe or evidence appears only after accepted
   save proof.
5. Confirm `project-save/` contains a non-empty saved project and save evidence.
6. Confirm `project-reopen/` contains non-empty reopen evidence and
   `reopened-state.json`.
7. Confirm `source_saved_project_artifact` starts with `project-save/`.
8. Confirm the report keeps full UI automation, visible rendering correctness,
   grading, creative assessment, full Save completion, deployed sharing/platform
   success, and first-lesson completion unproven unless exact separate evidence
   exists.

## Non-claims

This readiness contract does not claim:

- full Save completion
- full reopen completion beyond the bounded artifact/state contract
- export completion
- full UI automation
- first-lesson completion
- visible rendering correctness
- deployed sharing or platform success
- creative assessment, learner-world grading, or complete Alice coverage

Avoid wording that turns artifact evidence into a broader product claim. The
report should not say that the lesson, rendering, grading, Save workflow, UI
automation, or platform sharing succeeded unless a separate evidence contract
proves that exact claim.

Do not fix missing reopen proof by editing generated Gadugi adapters or by
changing documentation wording. Fix the canonical evidence source, rerun
validation, and regenerate adapters only from canonical assets when asset
changes require it.
