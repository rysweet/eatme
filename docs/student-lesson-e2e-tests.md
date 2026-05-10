# Student lesson E2E tests

The student lesson E2E test suite validates the student-facing contract of the
first-lesson readiness system. It exercises both entry points—full readiness
sequence (`run_first_lesson_readiness_sequence`) and desktop fixture inspection
(`check_lesson_session_readiness`)—against synthetic fixtures that simulate
every stage of the student hook chain without launching real Alice.

The tests enforce three guarantees:

1. **Honest readiness status.** The report never claims pass, completion,
   grading, or creative assessment unless explicit evidence exists.
2. **Student-visible wording.** Evidence summaries, unproven claims, and
   boundary descriptions use language a student or instructor can read.
3. **Hook chain progression.** The four core student actions—place object, edit
   procedure, run world, save project—advance through the evidence progress
   chain in the correct order.

## Contents

- [Usage](#usage)
- [Test inventory](#test-inventory)
- [API entry points under test](#api-entry-points-under-test)
- [Shared test support modules](#shared-test-support-modules)
- [Configuration](#configuration)
- [Examples](#examples)
- [Authoring workflow](#authoring-workflow)
- [Maintenance checklist](#maintenance-checklist)

## Usage

Run the student lesson E2E tests:

```bash
TMPDIR=/tmp cargo test -p eatme-alice --test student_lesson_e2e -- --test-threads=1
```

Run the full `eatme-alice` crate test suite to confirm zero regressions:

```bash
TMPDIR=/tmp cargo test -p eatme-alice -- --test-threads=1
```

Run the repository quality gate when a change touches readiness wording,
evidence boundaries, or student-facing report fields:

```bash
TMPDIR=/tmp ./scripts/quality-gates.sh
```

Use `TMPDIR=/tmp` in deep worktrees to avoid Unix socket path length failures.

## Test inventory

The test file lives at `crates/eatme-alice/tests/student_lesson_e2e.rs` (436
lines, within the 500-line quality gate).

| Test | Entry point | What it validates |
| --- | --- | --- |
| `run_first_lesson_readiness_sequence_reports_student_facing_contract` | `run_first_lesson_readiness_sequence` | Full sequence with fake targets. Validates `readiness_status`, `evidence_progress` consistency, `unproven_claims` wording, `shown_evidence`/`not_yet_shown` item ids, target failure category, and no overclaiming. |
| `check_lesson_session_readiness_validates_student_desktop_fixture` | `check_lesson_session_readiness` | Desktop fixture with all four hooks passed. Validates `required_evidence`, progress items, `unproven_claims`, and student-visible summary wording. |
| `evidence_progress_tracks_student_hook_chain` | `check_lesson_session_readiness` | Four fixture configurations testing the chain: place-object → edit-procedure → run-world → save-project. Validates `next_missing_real_desktop_proof` at each stage. |
| `readiness_boundaries_disallow_completion_grading_creative_assessment` | `check_lesson_session_readiness` | Validates all 7 required boundary IDs. Asserts grading, creative_assessment, and first_lesson_completion are never `"present"` when evidence is absent. |
| `student_readiness_does_not_overclaim_across_fixture_configurations` | `check_lesson_session_readiness` | Three fixture configs (minimal, pixel-blocked, all-hooks-passed) each checked with `assert_no_unsupported_readiness_claims`. |
| `required_evidence_covers_student_session_steps` | `check_lesson_session_readiness` | Action assertions include place-object, edit-procedure, run-world, save-project. Limitations are non-empty. |

## API entry points under test

### `run_first_lesson_readiness_sequence`

The full readiness sequence runs a first-lesson comparison against baseline and
modernized targets, then produces a structured readiness report. The test
creates fake tool binaries and a targets registry YAML to exercise the sequence
without real Alice.

```rust
use eatme_alice::{FirstLessonReadinessOptions, run_first_lesson_readiness_sequence};

let report = run_first_lesson_readiness_sequence(&FirstLessonReadinessOptions {
    registry_path,
    baseline_target: "baseline".into(),
    modernized_target: "modernized".into(),
    baseline_home_override: None,
    modernized_home_override: None,
    run_id: "student-lesson-e2e-sequence".into(),
    runs_dir: fixture.root.join("runs"),
    timeout_seconds: 1,
    json: true,
    no_memory: true,
    offline_package: true,
    execute: true,
    starter_project: None,
})
.unwrap();
```

The report includes:

| Field | Type | Meaning |
| --- | --- | --- |
| `passed` | `bool` | Whether the readiness check passed. Always `false` when evidence is incomplete. |
| `readiness_status` | `String` | `"incomplete"`, `"ready"`, or `"blocked"`. |
| `evidence_progress` | `EvidenceProgress` | Summary string, items list, and `next_missing_real_desktop_proof`. |
| `unproven_claims` | `Vec<String>` | Student-visible sentences naming what is not proven. |
| `shown_evidence` | `Vec<ReadinessEvidenceItem>` | Items with `.id` identifying evidence that has been collected. |
| `not_yet_shown` | `Vec<ReadinessEvidenceItem>` | Items with `.id` identifying evidence still missing. |
| `target_statuses` | `HashMap<String, TargetStatus>` | Per-target launch manifest and failure category. |

### `check_lesson_session_readiness`

The desktop fixture entry point checks a pre-existing comparison manifest
without running a new comparison. Tests use `write_manifest` from the desktop
evidence support module to create synthetic manifests with controlled hook
states.

```rust
let report = eatme_alice::check_lesson_session_readiness(&manifest_path).unwrap();
```

The report includes:

| Field | Type | Meaning |
| --- | --- | --- |
| `required_evidence` | `Vec<String>` | Human-readable list of evidence items a student session must produce. |
| `evidence_progress` | `EvidenceProgress` | Same shape as the sequence report. |
| `unproven_claims` | `Vec<String>` | Always non-empty; the system never claims full automation. |
| `evidence_boundaries` | `Vec<EvidenceBoundary>` | Each boundary has `id` and `status`. The 7 required IDs are `select_project`, `procedure_edit`, `save_project`, `visible_rendering`, `grading`, `creative_assessment`, `first_lesson_completion`. |
| `target_evidence` | `Vec<TargetEvidence>` | Per-role evidence with `action_assertions` covering the four student actions. |
| `limitations` | `Vec<String>` | Always non-empty; describes what the system cannot prove. |

## Shared test support modules

The E2E tests reuse two existing test support modules rather than duplicating
fixture code:

| Module | Import path | Purpose |
| --- | --- | --- |
| `launch_smoke_support` | `mod launch_smoke_support` | `TestFixture` with `write_fake_tools()` and `write_fake_alice_repo()` for the full sequence path. `PathOverride` prepends fake binaries to `$PATH`. |
| `desktop_evidence_support` | `#[path = "first_lesson_desktop_evidence/support.rs"]` | `DesktopFixture`, `write_manifest`, `PixelObservationFixture`, `FirstLessonNextActionFixture` for the desktop fixture path. |

Adding the `#[path = ...]` attribute avoids creating a redundant directory
structure while keeping both support modules importable from the same test file.

## Configuration

The student lesson E2E tests do not require real Alice desktop execution, Node,
or environment variables. They use in-process Rust fixtures.

| Setting | Required | Purpose |
| --- | --- | --- |
| `TMPDIR=/tmp` | Recommended | Avoids Unix socket path length errors in deep worktrees. |
| `--test-threads=1` | Recommended | Prevents `PathOverride` test isolation from conflicting across threads. |
| `NODE_OPTIONS` | No | Not needed; no Node processes are launched. |
| `EATME_REAL_ALICE` | No | Not needed; no real Alice launches. |
| `ALICE_HOME` | No | Not needed; fake Alice home directories are created by fixtures. |

## Examples

### Run a single test by name

```bash
TMPDIR=/tmp cargo test -p eatme-alice --test student_lesson_e2e \
  evidence_progress_tracks_student_hook_chain
```

### Check the hook chain progression at a specific stage

The `evidence_progress_tracks_student_hook_chain` test steps through four
stages. Each stage sets the hooks that have passed and asserts the next expected
hook in `next_missing_real_desktop_proof`:

| Hooks passed | Expected next |
| --- | --- |
| (none) | `place-object` |
| `place_object_ui_action` | `edit-procedure-or-code-block` |
| `place_object_ui_action`, `edit_procedure_ui_action` | `run-world` |
| `place_object_ui_action`, `edit_procedure_ui_action`, `run_world_ui_action` | `save-project` |

Every stage also asserts the automation limit wording: the next proof message
always contains `"does not prove full UI automation"`.

### Verify boundary enforcement

The boundaries test confirms all 7 boundary IDs are present and that three
specific boundaries—`grading`, `creative_assessment`, `first_lesson_completion`—
never report status `"present"` when desktop evidence is absent. This prevents
the readiness system from falsely claiming capabilities it cannot prove.

### Assert no overclaiming across configurations

The overclaim test runs three fixture configurations representing progressively
more evidence (minimal → pixel-blocked → all-hooks-passed) through
`assert_no_unsupported_readiness_claims`. The helper rejects these forbidden
phrases in any report output:

- `save completion evidence`
- `save completed`
- `save project succeeded`
- `lesson completed`
- `ui automation succeeded`
- `grading occurred`
- `creative assessment passed`
- `creative quality assessed`

## Authoring workflow

Use this workflow when adding a student lesson E2E test or modifying evidence
wording.

1. **Pick the right test or add a new one.** The six tests map to distinct
   contract surfaces. Add a new test only when an existing test cannot cover the
   claim.

2. **Reuse the shared fixtures.** Use `TestFixture` for sequence tests and
   `DesktopFixture` with `write_manifest` for desktop fixture tests. Do not
   duplicate fixture setup.

3. **Keep the file under 500 lines.** The file is currently 436 lines. If a new
   test pushes past 500, extract a `student_lesson_e2e/` support submodule
   following the pattern in `first_lesson_desktop_evidence/support.rs`.

4. **Preserve no-overclaim assertions.** The `assert_no_unsupported_readiness_claims`
   helper must remain intact. Do not weaken forbidden phrases to make a wording
   change pass; instead, fix the wording in the production code.

5. **Run the focused tests and the full crate suite:**

   ```bash
   TMPDIR=/tmp cargo test -p eatme-alice --test student_lesson_e2e -- --test-threads=1
   TMPDIR=/tmp cargo test -p eatme-alice -- --test-threads=1
   ```

6. **Run the quality gate** when changing evidence wording or boundary logic:

   ```bash
   TMPDIR=/tmp ./scripts/quality-gates.sh
   ```

## Maintenance checklist

Before merging a change that touches the student lesson E2E tests:

| Check | Command |
| --- | --- |
| Format Rust files | `cargo fmt --check` |
| Run student lesson E2E tests | `TMPDIR=/tmp cargo test -p eatme-alice --test student_lesson_e2e -- --test-threads=1` |
| Run all eatme-alice tests | `TMPDIR=/tmp cargo test -p eatme-alice -- --test-threads=1` |
| Validate assets | `cargo run -q -p eatme-cli -- assets validate --json` |
| Check generated adapters | `cargo run -q -p eatme-cli -- assets generate-gadugi --check --json` |
| Enforce Rust module size | `./scripts/quality-gates.sh` |
| Confirm line count | `wc -l crates/eatme-alice/tests/student_lesson_e2e.rs` (must be ≤ 500) |
