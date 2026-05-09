# Save/reopen readiness

Save/reopen readiness is the bounded evidence contract for proving that an
Alice project persistence lane is safe to review. It records save-artifact proof,
reopen-artifact proof, and reopened-state proof without turning those facts into
full UI automation, visible rendering correctness, grading, creative assessment,
or first-lesson completion.

Use this page when a run needs to answer:

> Did this evidence lane produce a saved `.a3p`, reopen that saved artifact
> instead of the bundled starter project, and record state evidence for review?

It does not answer whether a learner completed the lesson, whether the saved
world deserves a grade, whether rendering is correct, or whether every Alice UI
step is automated.

## Contents

- [Usage](#usage)
- [Evidence boundary](#evidence-boundary)
- [Configuration](#configuration)
- [Hook API](#hook-api)
- [Readiness output](#readiness-output)
- [Tutorial: review a save/reopen lane](#tutorial-review-a-savereopen-lane)
- [Troubleshooting](#troubleshooting)

## Usage

Run commands from the repository root:

```bash
git rev-parse --show-toplevel
```

Validate the canonical assets before trusting a readiness run:

```bash
cargo run -q -p eatme-cli -- assets validate --json
```

Check that generated Gadugi adapters still match the canonical eatme scenarios:

```bash
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
```

Run the bounded first-lesson readiness sequence when both Alice targets are
available:

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

The sequence writes the comparison manifest under:

```text
runs/comparisons/first-lessons-real-ui-actions/local-save-reopen-readiness/comparison-manifest.json
```

Check an existing manifest without rerunning Alice:

```bash
cargo run -q -p eatme-cli -- alice check-lesson-readiness \
  --manifest runs/comparisons/first-lessons-real-ui-actions/local-save-reopen-readiness/comparison-manifest.json \
  --json
```

Use the result as readiness evidence only for the named boundaries. A passing
save/reopen boundary means the required artifacts and JSON declarations are
present, safe, and current for that run. It is not a full Save completion claim
and is not a first-lesson completion claim.

## Evidence boundary

Save/reopen readiness is layered on top of the starter-project and first-lesson
evidence contracts:

| Layer | What it can prove | What it must not imply |
| --- | --- | --- |
| Starter-project preflight | The bundled starter project can be opened and inspected with launch evidence. | Save, reopen, export, full UI automation, or lesson completion. |
| Save artifact proof | A deterministic save affordance produced a non-empty saved `.a3p` and save evidence for the current run. | Full Save completion, grading, creative assessment, or first-lesson completion. |
| Reopen artifact proof | A deterministic reopen affordance reopened the saved `.a3p`, not the bundled starter project, and produced non-empty reopen evidence. | Visible rendering correctness, broad Alice compatibility, or creative quality. |
| Reopened-state proof | The reopen affordance produced state evidence and marked state verification as passed for the bounded selector. | Learner-world grading, instructor assessment, or complete lesson success. |

The required sequence is:

1. Open the bundled starter project through the existing Alice launch path.
2. Reach the first-lesson run proof needed before persistence review.
3. Save the edited project through a deterministic save affordance.
4. Record a non-empty saved `.a3p` and save evidence under `project-save/`.
5. Reopen that saved artifact in a new or explicitly reset Alice session.
6. Record non-empty reopen evidence and reopened-state evidence under
   `project-reopen/`.
7. Report unsupported or missing affordances as `blocked`, not as success.

Each step depends on the previous step. Reopen proof is blocked until save
artifact proof exists. Save proof is blocked until run-world proof exists.

## Configuration

| Setting | Required for | Description |
| --- | --- | --- |
| `NODE_OPTIONS=--max-old-space-size=32768` | Agentic or wrapper workflows | Large-heap setting used by local workflow wrappers. It is safe to keep exported while running Rust commands. |
| `EATME_REAL_ALICE=1` | Real Alice execution | Explicit opt-in for non-baseline desktop execution. |
| `ALICE_BASELINE_HOME` | First-lesson comparison | Original Alice checkout used by `alice run-first-lesson-readiness --execute`. |
| `ALICE_MODERNIZED_HOME` | First-lesson comparison | RabbitHole or candidate Alice checkout used by `alice run-first-lesson-readiness --execute`. |
| `ALICE_HOME` | Single-target launch smoke | Alice checkout used by `alice launch-smoke`. |

The Alice checkout may expose deterministic persistence hooks:

```text
tools/eatme-save-project
tools/eatme-reopen-project
```

If a hook is absent, the readiness lane reports a `blocked` affordance with the
missing capability. It must not silently skip the step or substitute mocked save
or reopen evidence.

Real desktop evidence also needs the Alice dependency set documented in
[Alice Integration](alice-integration.md), including Java, Maven, Xvfb, window
tools, screenshot tooling, and software OpenGL support.

## Hook API

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

## Readiness output

Save/reopen readiness uses explicit states:

| State | Meaning |
| --- | --- |
| `passed` | The named hook ran, returned accepted JSON, and produced all required non-empty artifacts. |
| `blocked` | A required earlier proof or deterministic Alice affordance is missing. The run is honest but not ready for that boundary. |
| `failed` | A hook ran or returned data, but the command, JSON, artifact, or validation result did not satisfy the contract. |

The UI action contract records persistence facts with stable action ids:

| Action id | Evidence |
| --- | --- |
| `save-project` | Save artifact proof or a blocked save precondition. |
| `reopen-project` | Reopen artifact proof or a blocked reopen precondition. |

Safe user-facing wording:

```text
Save artifact proof is shown for this run.
Reopen artifact proof is shown for the saved artifact from this run.
Save completion is not claimed as full UI completion.
First-lesson completion is not proven.
```

Unsafe wording:

```text
The lesson is complete.
Rendering is correct.
The saved world was graded.
Alice UI automation is complete.
```

## Tutorial: review a save/reopen lane

1. Validate assets with `cargo run -q -p eatme-cli -- assets validate --json`.
2. Check generated adapters with
   `cargo run -q -p eatme-cli -- assets generate-gadugi --check --json`.
3. Run or locate the first-lesson readiness comparison manifest.
4. Open the run evidence directory and confirm `project-save/` contains a
   non-empty saved `.a3p` and save JSON.
5. Confirm `project-reopen/` contains non-empty reopen evidence and
   `reopened-state.json`.
6. Confirm `source_saved_project_artifact` starts with `project-save/`.
7. Confirm the report keeps full UI automation, visible rendering correctness,
   grading, creative assessment, full Save completion, and first-lesson
   completion unproven unless exact separate evidence exists.

## Troubleshooting

| Symptom | Meaning | Fix |
| --- | --- | --- |
| `blocked: run-world proof is required before project save would be safe` | The run has not produced the prerequisite run-world proof. | Collect run-world evidence first, then rerun the save/reopen lane. |
| `blocked: save-project proof is required before project reopen would be safe` | Reopen was evaluated before accepted save artifact proof existed. | Produce valid `project-save/` evidence in the same run. |
| `blocked: Alice checkout does not expose tools/eatme-save-project` | The Alice checkout lacks the save affordance contract. | Add or select an Alice checkout with the deterministic save hook. |
| `blocked: Alice checkout does not expose tools/eatme-reopen-project` | The Alice checkout lacks the reopen affordance contract. | Add or select an Alice checkout with the deterministic reopen hook. |
| `source_saved_project_artifact must reopen the saved artifact, not the bundled starter project` | Reopen evidence pointed at the original starter project or another non-save source. | Reopen the saved `.a3p` from `project-save/` for the same run. |
| `reopened_state_artifact must be non-empty` | The reopened-state proof file is missing or empty. | Write state evidence from the reopened project before reporting readiness. |

Do not fix these conditions by editing generated Gadugi adapters or by changing
documentation wording. Fix the canonical evidence source, rerun validation, and
regenerate adapters only from canonical assets when asset changes require it.
