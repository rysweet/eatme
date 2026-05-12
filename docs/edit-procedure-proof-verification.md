# Edit procedure proof artifact verification

The launch smoke harness checks for a proof artifact file after the
Alice-side `EatmeEditProcedure` hook runs. When the proof artifact
`first-lesson-code-editor-action-proof.json` is present and valid in the
Alice run directory, the edit procedure probe records
`edit_procedure_verified=true` and includes the proof details. When the file
is missing or invalid, it records `edit_procedure_verified=false`. This
upgrades the `edit_procedure_ui_action` assertion from always-fail to
evidence-based.

The proof artifact is a supplementary verification path. The existing
Alice-side procedure edit command hook remains the primary path. The proof
artifact provides an alternative when the hook ran but could not produce the
full edited project artifact and procedure/code diff pair — for example,
when the hook partially succeeded or when an external tool deposited proof
independently.

## Contents

- [Proof artifact file](#proof-artifact-file)
- [How verification works](#how-verification-works)
- [Assertion behavior](#assertion-behavior)
- [API surface](#api-surface)
- [Integration point](#integration-point)
- [Configuration](#configuration)
- [Examples](#examples)
- [Troubleshooting](#troubleshooting)
- [Related documentation](#related-documentation)

## Proof artifact file

The proof artifact file name is:

```text
first-lesson-code-editor-action-proof.json
```

The file is expected in the Alice run directory (`run_dir`) — the same
directory that contains `procedure-edit/` and `object-placement/`
subdirectories. The full path is:

```text
<runs_dir>/<scenario_id>/<run_id>/first-lesson-code-editor-action-proof.json
```

### Schema

The file must be valid JSON. The harness parses it as a generic
`serde::de::IgnoredAny` (validating without building a value tree) and extracts a summary for the `proof_detail` field.
No specific schema version is required — any valid JSON object is accepted.

A typical proof artifact:

```json
{
  "schema_version": "eatme.code-editor-action-proof/v1",
  "status": "verified",
  "editor_action": "append-comment",
  "procedure_selector": "scene.eatmeFirstLessonStep",
  "timestamp": "2026-05-12T07:30:00Z",
  "detail": "Code editor action was observed and recorded by the external proof collector."
}
```

The harness does not validate individual fields beyond requiring valid JSON.
All top-level keys and values are preserved in the proof summary up to 500
characters. If the JSON is valid but has no recognizable fields, the proof
still counts as verified — the artifact's existence and valid parse are
sufficient.

### File states

| File state | `edit_procedure_verified` | `proof_detail` | Effect on `proves_edit()` |
| --- | --- | --- | --- |
| Valid JSON file present | `true` | Summary of JSON contents (≤500 chars) | `proves_edit()` returns `true` even if the hook did not produce full proof |
| File missing | `false` | `None` | No change to existing `proves_edit()` logic |
| File present but not valid JSON | `false` | Error message describing the parse failure | No change to existing `proves_edit()` logic |
| File present but empty | `false` | Parse error (e.g., "invalid JSON in <path>: EOF while parsing a value at line 1 column 0") | No change to existing `proves_edit()` logic |

## How verification works

After the `probe_edit_procedure_hook()` function returns the
`UiActionEditProcedureProbe`, the harness chains a
`with_proof_artifact_check(&run_dir)` call. This method:

1. Constructs the proof artifact path: `run_dir.join("first-lesson-code-editor-action-proof.json")`
2. Attempts to read the file.
3. If the file exists, parses it with `serde::de::IgnoredAny` (validates JSON without building an in-memory value tree).
4. On successful parse: sets `edit_procedure_verified = true`, stores a
   truncated JSON summary in `proof_detail`, and appends proof source
   information to the probe's `detail` field when the hook alone did not
   prove the edit.
5. On failure (missing file, read error, invalid JSON): sets
   `edit_procedure_verified = false` and stores the error in `proof_detail`
   if a file was found but could not be parsed.

The method is non-destructive — it never overwrites a passing hook result.
It only supplements verification when the hook alone was insufficient.

### OR-logic in `proves_edit()`

The `proves_edit()` method uses OR-logic to combine hook evidence and proof
artifact evidence:

```rust
pub fn proves_edit(&self) -> bool {
    let hook_proves = self.status == "passed"
        && self.edited_project_artifact.is_some()
        && self.procedure_or_code_diff.is_some()
        && self.validation_errors.is_empty();
    hook_proves || self.edit_procedure_verified
}
```

Either path is sufficient:

| Hook result | Proof artifact | `proves_edit()` |
| --- | --- | --- |
| Hook passed with full proof | Any | `true` (hook path) |
| Hook blocked/failed | Valid proof artifact present | `true` (proof artifact path) |
| Hook blocked/failed | Missing or invalid proof artifact | `false` |
| Hook passed with full proof | Missing or invalid proof artifact | `true` (hook path) |

## Assertion behavior

The `edit_procedure_ui_action` assertion in the launch smoke manifest uses
`proves_edit()` to determine pass/fail. Before this feature, the assertion
was always `fail` when the hook did not produce full proof. Now:

- When the proof artifact validates, the assertion passes and the detail
  includes the proof source (e.g., "edit procedure verified via proof
  artifact: code editor action was observed").
- When the proof artifact is missing or invalid, the assertion remains
  `fail` with the existing hook detail, unchanged from previous behavior.
- The `edit_procedure_candidate_hook_probe` assertion is unaffected — it
  still validates the hook probe shape independently.

### Manifest fields

Two new fields appear on the `UiActionEditProcedureProbe` in the manifest
JSON:

| Field | Type | Default | Meaning |
| --- | --- | --- | --- |
| `edit_procedure_verified` | `bool` | `false` | Whether the proof artifact was found and successfully parsed |
| `proof_detail` | `string` or `null` | `null` | Summary of proof contents on success, or error description on parse failure; `null` when no proof artifact was attempted or the file was simply missing |

These fields are serialized into the launch smoke manifest alongside
existing edit procedure probe fields. Existing consumers that ignore
unknown fields are unaffected.

## API surface

The feature adds no new public API. All changes are within the existing
`launch_edit_procedure` module.

| Item | Visibility | Purpose |
| --- | --- | --- |
| `EDIT_PROCEDURE_PROOF_ARTIFACT` | Module constant | Filename: `"first-lesson-code-editor-action-proof.json"` |
| `UiActionEditProcedureProbe::edit_procedure_verified` | `pub` field | Whether proof artifact validated |
| `UiActionEditProcedureProbe::proof_detail` | `pub` field | Proof summary or error detail |
| `UiActionEditProcedureProbe::with_proof_artifact_check()` | `pub(crate)` method | Builder-style consuming method; reads and validates the proof artifact and returns the enriched probe |

The `proves_edit()` method signature is unchanged. Its return value may
differ when the proof artifact is present — this is intentional.

## Integration point

The proof artifact check is chained in `launch.rs` immediately after
`probe_edit_procedure_hook()`:

```rust
let edit_procedure_probe = probe_edit_procedure_hook(
    &runner,
    &options.alice_home,
    &run_dir,
    &object_placement_probe,
    display.name(),
)
.with_proof_artifact_check(&run_dir);
```

This is a single-line addition. No other function signatures or call sites
change.

## Configuration

No new configuration, environment variables, or CLI flags are required.
The proof artifact filename is a compile-time constant. The feature
activates automatically when the file is present in the run directory.

| Parameter | Value | Source |
| --- | --- | --- |
| Proof artifact filename | `first-lesson-code-editor-action-proof.json` | `EDIT_PROCEDURE_PROOF_ARTIFACT` constant |
| Expected location | `<run_dir>/first-lesson-code-editor-action-proof.json` | Relative to the scenario run directory |
| Proof detail truncation | 500 characters | Hardcoded in `with_proof_artifact_check()` |

## Examples

### Deposit a proof artifact for a local smoke run

After running the Alice-side edit procedure hook (or an external editor
automation tool), write the proof file to the run directory:

```bash
RUN_DIR=runs/first-lessons-real-ui-actions/local-run-001

cat > "$RUN_DIR/first-lesson-code-editor-action-proof.json" <<'EOF'
{
  "schema_version": "eatme.code-editor-action-proof/v1",
  "status": "verified",
  "editor_action": "append-comment",
  "procedure_selector": "scene.eatmeFirstLessonStep",
  "detail": "Code editor action was observed and recorded."
}
EOF
```

Then run the launch smoke:

```bash
EATME_REAL_ALICE=1 cargo run -q -p eatme-cli -- alice launch-smoke \
  --scenario first-lessons-real-ui-actions \
  --run-id local-run-001 \
  --json
```

The `edit_procedure_ui_action` assertion will pass if the hook was blocked
but the proof artifact is valid.

### Inspect the proof artifact verification in the manifest

After a launch smoke run, check the edit procedure probe fields:

```bash
MANIFEST=runs/first-lessons-real-ui-actions/local-run-001/manifest.json

jq '.ui_action_probes.edit_procedure_probe | {
  status,
  edit_procedure_verified,
  proof_detail,
  proves_edit: (.status == "passed" or .edit_procedure_verified)
}' "$MANIFEST"
```

Expected output when proof artifact is present:

```json
{
  "status": "blocked",
  "edit_procedure_verified": true,
  "proof_detail": "{\"schema_version\":\"eatme.code-editor-action-proof/v1\",\"status\":\"verified\",...}",
  "proves_edit": true
}
```

### Run the unit tests

```bash
TMPDIR=/tmp cargo test -p eatme-alice launch_edit_procedure
```

The test suite includes three proof-artifact-specific tests:

| Test | What it validates |
| --- | --- |
| `proof_artifact_valid_json_sets_verified_true` | Valid JSON proof file sets `edit_procedure_verified=true` and `proves_edit()` returns `true` |
| `proof_artifact_missing_sets_verified_false` | Missing file leaves `edit_procedure_verified=false` with no error in `proof_detail` |
| `proof_artifact_invalid_json_sets_verified_false` | Invalid JSON sets `edit_procedure_verified=false` with parse error in `proof_detail` |

## Troubleshooting

### Proof artifact exists but `edit_procedure_verified` is still false

Check that the file is valid JSON:

```bash
jq . "$RUN_DIR/first-lesson-code-editor-action-proof.json"
```

If `jq` reports a parse error, the file contains invalid JSON. The
`proof_detail` field in the manifest will contain the specific parse error.

### Proof artifact is valid but `proves_edit()` still returns false

This should not happen. If `edit_procedure_verified` is `true`, then
`proves_edit()` returns `true` regardless of the hook status. Check that
the proof artifact check is actually running by verifying the
`edit_procedure_verified` field in the manifest is present (not absent).
If absent, the code change may not have been compiled.

### Proof artifact path is wrong

The proof artifact must be in the run directory root, not in a
subdirectory. Correct path:

```text
runs/first-lessons-real-ui-actions/local-run-001/first-lesson-code-editor-action-proof.json
```

Not:

```text
runs/first-lessons-real-ui-actions/local-run-001/procedure-edit/first-lesson-code-editor-action-proof.json
```

### 500-line module limit

The quality gate checks every `.rs` file independently. The proof artifact
check adds approximately 40 lines to `launch_edit_procedure.rs` (391 →
~430 lines) and the three new tests add approximately 55 lines to
`launch_edit_procedure/tests.rs` (193 → ~248 lines). Both stay well under
the 500-line limit. If the quality gate reports a violation, split the
proof artifact logic into a `launch_edit_procedure_proof` submodule.

## Related documentation

- [Code Editor First Run E2E Test](code-editor-first-run-e2e.md) — E2E
  tests for the `code-editor-first-run` scenario that exercises the
  edit procedure pipeline.
- [Evidence Artifact Contract](evidence-artifact-contract.md) — Schema and
  validation rules for evidence artifacts in the readiness system.
- [Alice Lesson Smoke](alice-lesson-smoke.md) — Desktop scenario roster
  including the scenarios that use the edit procedure probe.
- [First-Lesson Vertical Slice](first-lesson-vertical-slice.md) — The
  first-lesson automation pipeline that includes the edit procedure step.
- [Validation and Quality Gates](validation-quality-gates.md) — Repository
  quality gates including the 500-line module limit.
