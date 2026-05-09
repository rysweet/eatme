# Save/reopen readiness

Save/reopen readiness is the bounded evidence contract for recording that a
save hook produced a saved artifact from the edited project path, a reopen hook
used that saved artifact, and reopened-state evidence was captured for review.
It does not treat those artifacts as semantic project-change proof, full UI
automation, or lesson completion.

Use this page to review one question:

> Did the harness save an artifact from the edited project path, reopen that
> saved artifact instead of the bundled starter project, and record
> reopened-state evidence for review?

The answer is trusted only when the run evidence records the save proof, reopen
proof, and reopened-state proof described below. A starter-project launch smoke
or opened-project preflight does not prove save/reopen readiness by itself.

## Contents

- [Evidence boundary](#evidence-boundary)
- [Usage](#usage)
- [Configuration](#configuration)
- [Evidence directories](#evidence-directories)
- [Integration boundary](#integration-boundary)
- [Hook API](#hook-api)
- [Path validation](#path-validation)
- [Rust API](#rust-api)
- [Contract tests](#contract-tests)
- [Readiness states](#readiness-states)
- [Review checklist](#review-checklist)
- [PR review evidence](#pr-review-evidence)
- [Non-claims](#non-claims)

## Evidence boundary

Save/reopen readiness is layered on top of the starter-project and first-lesson
evidence contracts.

| Layer | What it proves | What it does not imply |
| --- | --- | --- |
| Starter-project preflight | The bundled starter project can be opened and inspected with launch evidence. | Save, reopen, export, full UI automation, or lesson completion. |
| Save artifact proof | A deterministic save affordance produced a non-empty saved `.a3p` and save evidence for the current run. | Semantic project change, full Save completion, grading, creative assessment, or first-lesson completion. |
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

Read the resulting manifest and `ui-action-contract.json` as prerequisite and
save-action review inputs. Read `project-reopen/` only when the run or a
dedicated persistence report explicitly produced reopen evidence. Treat
`save-project` and `reopen-project` as ready only when their own proof artifacts
are present, non-empty, and accepted by validation. Treat a missing hook,
missing report surface, or missing precondition as a bounded `blocked` result.

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

A complete save/reopen evidence bundle uses this layout under the selected run
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
that points back to the bundled starter project, a different saved artifact, or
an artifact outside the run evidence directories is rejected.

## Integration boundary

`ui-action-contract.json` is the action-readiness report for the first-lesson
flow. It can include action probes through `save-project`, including blocked or
passed save proof. Do not infer reopen proof from `ui-action-contract.json`
unless that file explicitly contains a dedicated `reopen-project` probe.

`project-reopen/` is the separate persistence evidence lane for the full
save/reopen feature. A reviewer should accept reopen readiness only from an
explicit `reopen-project` probe or report that:

1. depends on accepted `save-project` proof from the same run;
2. passes the saved `.a3p` as `--saved-project`;
3. records non-empty `reopened-project.a3p`, `project-reopen.json`, and
   `reopened-state.json` artifacts under `project-reopen/`;
4. reports `source_saved_project_artifact` as the same canonical artifact that
   `save-project` produced under `project-save/`.

This keeps the feature boundary clear: save proof may appear in the UI action
contract, while reopen proof requires its own explicit persistence evidence.

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
| `source_saved_project_artifact` | Must be a simple relative path starting with `project-save/`, must resolve under the run's `project-save/` evidence directory to the same saved artifact from the same run, and must not point to the bundled starter project. |
| `reopen_selector` | Must be `scene.eatmeFirstLessonStep`. |
| `reopened_project_artifact` | Must be a simple relative path under `project-reopen/` and must point to a non-empty file. |
| `reopen_artifact` | Must be a simple relative path under `project-reopen/` and must point to a non-empty file. |
| `reopened_state_artifact` | Must be a simple relative path under `project-reopen/` and must point to a non-empty file. |
| `state_verification` | Must be `passed` as reported by the reopen hook for the bounded selector. |

Absolute paths, parent traversal, symlink escapes, empty files, malformed JSON,
wrong schema versions, and artifacts outside the expected run evidence
directories are not accepted as proof.

## Path validation

The `launch_path_validation` module defends every artifact resolution against
path traversal, symlink escape, and absolute-path injection. All save and reopen
artifact paths pass through this validation before they are accepted as proof.

| Defense | Implementation |
| --- | --- |
| Absolute path rejection | `normal_components` rejects any path whose components include root, prefix, or current/parent-directory markers. Only `Component::Normal` values are accepted. |
| Parent traversal rejection | A path containing `..` produces a `None` from `normal_components` and is rejected before file I/O occurs. |
| Symlink escape rejection | `canonical_artifact_under` calls `canonicalize()` on both the root directory and the artifact path, then checks that the canonical artifact starts with the canonical root. A symlink that escapes the evidence directory fails this check. |
| Non-readable artifact rejection | `artifact_info_under` combines the containment check with `artifact_info` to confirm the file exists, is readable, and is non-empty. |

The two primary entry points are:

```rust
pub(crate) fn artifact_info_under(
    root_dir: &Path,
    relative_path: &str,
    field: &str,
    root_label: &str,
) -> Result<ArtifactInfo, String>

pub(crate) fn canonical_artifact_under(
    root_dir: &Path,
    artifact_path: &Path,
    field: &str,
    root_label: &str,
) -> Result<PathBuf, String>
```

`artifact_info_under` is the standard path for save and reopen artifact
validation: it validates containment, reads the artifact, and returns size and
path metadata. `canonical_artifact_under` is used by the reopen probe to verify
that `source_saved_project_artifact` resolves to the same canonical file as the
save probe's `saved_project_artifact`.

## Rust API

The save/reopen harness exposes two probe types for evidence consumers. Both
are constructed by the corresponding `probe_project_*_hook` functions, which
invoke the Alice-side hook, parse its JSON output, validate every artifact, and
return a typed probe.

### UiActionSaveProjectProbe

Constructed by `probe_project_save_hook`. Fields:

| Field | Type | Purpose |
| --- | --- | --- |
| `id` | `String` | Always `alice-side-project-save-command-hook`. |
| `action_id` | `String` | Always `save-project`. |
| `status` | `String` | `passed`, `blocked`, or `failed`. |
| `detail` | `String` | Human-readable status explanation. |
| `save_selector` | `String` | Always `scene.eatmeFirstLessonStep`. |
| `candidate_hook_path` | `String` | Resolved path to the save hook in the Alice checkout. |
| `command` | `Option<String>` | Full command line when the hook ran. |
| `exit_status` | `Option<i32>` | Exit code when the hook ran. |
| `stdout` | `String` | Hook stdout (expected JSON on success). |
| `stderr` | `String` | Hook stderr. |
| `saved_project_artifact` | `Option<ArtifactInfo>` | Validated saved `.a3p` artifact info under `project-save/`. |
| `save_artifact` | `Option<ArtifactInfo>` | Validated save evidence JSON artifact info under `project-save/`. |
| `validation_errors` | `Vec<String>` | All validation failures, empty when `status` is `passed`. |
| `missing_affordance` | `Option<UiActionMissingAffordance>` | Present when `status` is `blocked` due to a missing hook or precondition. |

The `proves_save()` method returns `true` only when `status` is `passed`, both
artifact fields are `Some`, and `validation_errors` is empty.

### UiActionReopenProjectProbe

Constructed by `probe_project_reopen_hook`. Fields:

| Field | Type | Purpose |
| --- | --- | --- |
| `id` | `String` | Always `alice-side-project-reopen-command-hook`. |
| `action_id` | `String` | Always `reopen-project`. |
| `status` | `String` | `passed`, `blocked`, or `failed`. |
| `detail` | `String` | Human-readable status explanation. |
| `reopen_selector` | `String` | Always `scene.eatmeFirstLessonStep`. |
| `candidate_hook_path` | `String` | Resolved path to the reopen hook in the Alice checkout. |
| `command` | `Option<String>` | Full command line when the hook ran. |
| `exit_status` | `Option<i32>` | Exit code when the hook ran. |
| `stdout` | `String` | Hook stdout (expected JSON on success). |
| `stderr` | `String` | Hook stderr. |
| `source_saved_project_artifact` | `String` | Relative path starting with `project-save/` that must resolve to the same canonical artifact the save probe produced. |
| `reopened_project_artifact` | `Option<ArtifactInfo>` | Validated reopened `.a3p` artifact info under `project-reopen/`. |
| `reopen_artifact` | `Option<ArtifactInfo>` | Validated reopen evidence JSON artifact info under `project-reopen/`. |
| `reopened_state_artifact` | `Option<ArtifactInfo>` | Validated reopened-state JSON artifact info under `project-reopen/`. |
| `validation_errors` | `Vec<String>` | All validation failures, empty when `status` is `passed`. |
| `missing_affordance` | `Option<UiActionMissingAffordance>` | Present when `status` is `blocked` due to a missing hook or precondition. |

The `proves_reopen()` method returns `true` only when `status` is `passed`,
`source_saved_project_artifact` is non-empty, all three artifact fields are
`Some`, and `validation_errors` is empty.

### UI action contract integration

Save proof can appear in `ui-action-contract.json` through the action contract
inspector in `compare/ui_action_contract/save.rs`. The inspector checks for:

- A `project-save-precondition` no-go probe with `run-world` passed and
  `deterministic-alice-project-save-affordance` not passed.
- A `alice-side-project-save-command-hook` candidate affordance probe with
  `status: passed`, a valid `save_selector`, non-empty artifacts, and no
  validation errors.

Reopen proof is not inferred from `ui-action-contract.json`. It requires an
explicit `reopen-project` probe in the `project-reopen/` evidence directory.

## Contract tests

The save/reopen contract tests are in `crates/eatme-alice/src/` and verify the
full save→reopen flow using `FakeCommandRunner` without real Alice execution.

| File | Tests | Boundary |
| --- | --- | --- |
| `launch_save_project/tests.rs` | Save hook invocation, JSON parsing, artifact validation, precondition gating, and blocked/failed state construction. | Save proof only; does not test reopen. |
| `launch_save_reopen_contract_tests.rs` | End-to-end save→reopen flow: reopen blocks until save proof exists, reopen passes only with the saved artifact reopened and state verified, reopen rejects the bundled starter project, reopen rejects different saved artifacts, reopen rejects symlink escapes, reopen fails when reopened-state artifact is missing or empty. | Full save→reopen chain including cross-probe dependency. |
| `compare/ui_action_contract/tests.rs` | UI action contract inspection: save-project no-go probe after run-world proof, save-project candidate with validation errors rejected as unproven. | Action contract boundary for save proof only. |

Run the save/reopen contract tests directly:

```bash
cargo test -p eatme-alice launch_save
cargo test -p eatme-alice launch_save_reopen_contract
```

Run the action contract tests:

```bash
cargo test -p eatme-alice ui_action_contract
```

Run all `eatme-alice` tests:

```bash
cargo test -p eatme-alice
```

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
4. Confirm any `reopen-project` probe or evidence appears only in an explicit
   persistence report after accepted save proof; do not infer reopen proof from
   `ui-action-contract.json` unless it contains a dedicated reopen probe.
5. Confirm `project-save/` contains a non-empty saved project and save evidence.
6. Confirm `project-reopen/` contains non-empty reopen evidence and
   `reopened-state.json`.
7. Confirm `source_saved_project_artifact` resolves to the same canonical
   artifact as `project-save/saved-project.a3p` from the same run and does not
   escape the run evidence directories through absolute paths, parent
   traversal, or symlinks.
8. Confirm the report keeps full UI automation, visible rendering correctness,
   grading, creative assessment, full Save completion, deployed sharing/platform
   success, and first-lesson completion unproven unless exact separate evidence
   exists.

After the checklist passes, describe the result as ready for
continuation/review based on available bounded evidence. Do not describe it as
end-to-end user success. If the same run lacks accepted save proof, any reopen
claim remains blocked no matter how much starter-project or launch evidence is
present.

## PR review evidence

Save/reopen PR finalization uses a bounded evidence record. The record is useful
only when it names the exact code under review, the files changed by the
finalization, the evidence inspected, the checks actually run, and the explicit
claims that remain out of scope.

| Field | Required content |
| --- | --- |
| PR and branch | Pull request number, branch name, and exact head SHA from GitHub metadata or the fetched PR ref. |
| Working tree | Clean working tree, or a file list limited to the documentation, asset, or code changes made for the finalization. |
| Files modified or no-op justification | A real `Files modified` list when files changed; otherwise an explicit `No-op justification` tied to existing committed files and evidence. |
| Save evidence inspected | `save-project` proof or no-go state, including status, required artifacts, and validation errors. |
| Reopen evidence inspected | `reopen-project` proof or no-go state, including source saved artifact, reopened artifacts, state verification, and validation errors. |
| UI action contract boundary | Whether `ui-action-contract.json` includes only save proof or also an explicit `reopen-project` probe. Reopen proof is not inferred from save proof. |
| Commands run | Only commands actually executed for the finalization, such as docs build, asset validation, Gadugi freshness, or targeted Rust tests. |
| Limitations | Explicit non-claims for full Alice UI automation, grading, creative assessment, full Save completion, first-lesson completion, export completion, and broad product readiness. |

Use this reusable shape for save/reopen recovery evidence:

```text
Default-workflow save/reopen recovery evidence for PR #<number>.

Branch: <branch-name>
HEAD: <exact-head-sha>
Evidence source: fetched PR head and GitHub PR metadata for the same head.

Files modified:
- docs/save-reopen-readiness.md - Documents the bounded save/reopen PR review
  evidence shape and explicit non-claims.
- docs/default-workflow-pr-readiness.md - Documents the save/reopen recovery output
  shape after rate-limit or no-op guard failure.

Save evidence: `proves_save()` is accepted only when status is `passed`, both
required save artifacts are present, and validation errors are empty.

Reopen evidence: `proves_reopen()` is accepted only when status is `passed`, the
source saved artifact is present, reopened project evidence, reopen evidence,
and reopened-state evidence are present, and validation errors are empty.

Validation: name only commands actually run for this finalization.

Limitations: This does not claim full Alice UI automation, grading validation,
creative-assessment validation, full Save completion, first-lesson completion,
export completion, or broad product readiness.
```

If the finalization changes no files, replace `Files modified` with:

```text
No-op justification: Evidence-only recovery for PR #<number>. The exact PR head
<exact-head-sha> was verified against branch <branch-name>, the fetched PR ref
matched GitHub `headRefOid`, current check status was reviewed for that same
head, and the committed save/reopen docs and tests already expressed the bounded
starter/save-reopen readiness boundary. No files were changed because no stale,
missing, or overbroad documentation, asset, generated output, or source artifact
was found.
```

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
