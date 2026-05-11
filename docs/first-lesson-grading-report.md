# First-lesson grading report

The first-lesson grading report checks per-step completion status for the
Building a Scene curriculum. It consumes an existing
`LessonSessionReadinessReport` and maps each canonical step to one of three
statuses: `ready`, `blocked`, or `not_yet_tested`. It does not generate new
proof or perform creative assessment.

The report answers one bounded question:

> For each step in the Building a Scene first lesson, can the current
> infrastructure prove that the step was completed?

It does not answer whether a learner completed the full lesson, whether an Alice
world is creatively successful, whether grading was performed, or whether the
lesson met pedagogical goals. Those claims require evidence that does not yet
exist in the automation infrastructure.

## Quick start

Check grading status for an existing comparison manifest:

```bash
cargo run -q -p eatme-cli -- alice first-lesson-grading-report \
  --manifest runs/comparisons/first-lessons-real-ui-actions/local/comparison-manifest.json \
  --json
```

Plain output (human-readable):

```bash
cargo run -q -p eatme-cli -- alice first-lesson-grading-report \
  --manifest runs/comparisons/first-lessons-real-ui-actions/local/comparison-manifest.json
```

## Canonical steps

The grading report defines 11 canonical steps derived from the alice.org
Building a Scene curriculum. Each step maps to existing infrastructure evidence.

### UI action steps

These steps map to entries in `REQUIRED_UI_ACTION_IDS`. A UI action step is
`ready` when the action was observed in the `ui-action-contract.json` evidence,
and `blocked` otherwise.

| Step | Source ID | Curriculum action |
| --- | --- | --- |
| Verify Alice window | `verify-specific-alice-window` | Confirm the Alice application window is visible and responsive. |
| Activate Alice window | `activate-specific-alice-window` | Bring the Alice scene editor window to the foreground. |
| Place object | `place-object` | Place a 3D object into the scene. |
| Edit procedure or code | `edit-procedure-or-code-block` | Edit a procedure or code block attached to the placed object. |
| Run world | `run-world` | Execute the world to observe the object's behavior. |
| Save project | `save-project` | Save the Alice project file. |

### Boundary steps

These steps map to evidence boundary states from `boundary_specs()` in
`first_lesson_boundaries.rs`. A boundary step is `ready` when its boundary
status is `"present"` in the readiness report, and `blocked` otherwise.

| Step | Boundary ID | Evidence source |
| --- | --- | --- |
| Select project | `select_project` | Select Project scenario evidence from the comparison manifest. |
| Visible rendering | `visible_rendering` | Visible rendering scenario evidence observed during the run. |

### Meta-boundary steps

These steps represent curriculum outcomes that current infrastructure cannot
prove. They always report `not_yet_tested`. When infrastructure is extended to
evaluate these boundaries, their status mapping will change.

| Step | Step ID | Why not yet tested |
| --- | --- | --- |
| Grading | `grading` | Automated grading infrastructure does not exist. |
| Creative assessment | `creative-assessment` | Creative quality evaluation requires instructor judgment. |
| First-lesson completion | `first-lesson-completion` | Full lesson completion proof requires all prior steps plus assessment. |

## Status values

The grading report uses a closed set of three status values:

| Status | Meaning |
| --- | --- |
| `ready` | Current infrastructure proves this step was completed. |
| `blocked` | Evidence exists or was expected, but the step cannot be confirmed. |
| `not_yet_tested` | Current infrastructure cannot evaluate this step. |

The status set is intentionally smaller than the readiness report's
`ready`/`not_ready`/`blocked` vocabulary. The grading report collapses
`not_ready` into `blocked` because a grading consumer needs to distinguish
between "infrastructure can check this but it failed" and "infrastructure cannot
check this yet."

## JSON schema

The `--json` output uses schema `eatme.first-lesson-grading-report/v1`:

```json
{
  "schema_version": "eatme.first-lesson-grading-report/v1",
  "scenario_id": "first-lessons-real-ui-actions",
  "steps": [
    {
      "id": "verify-specific-alice-window",
      "name": "Verify specific Alice window",
      "status": "blocked"
    },
    {
      "id": "activate-specific-alice-window",
      "name": "Activate specific Alice window",
      "status": "blocked"
    },
    {
      "id": "select-project",
      "name": "Select project",
      "status": "blocked"
    },
    {
      "id": "place-object",
      "name": "Place object",
      "status": "blocked"
    },
    {
      "id": "edit-procedure-or-code-block",
      "name": "Edit procedure or code block",
      "status": "blocked"
    },
    {
      "id": "run-world",
      "name": "Run world",
      "status": "blocked"
    },
    {
      "id": "save-project",
      "name": "Save project",
      "status": "blocked"
    },
    {
      "id": "visible-rendering",
      "name": "Visible rendering",
      "status": "blocked"
    },
    {
      "id": "grading",
      "name": "Grading",
      "status": "not_yet_tested"
    },
    {
      "id": "creative-assessment",
      "name": "Creative assessment",
      "status": "not_yet_tested"
    },
    {
      "id": "first-lesson-completion",
      "name": "First-lesson completion",
      "status": "not_yet_tested"
    }
  ]
}
```

The example above shows the output when no execution evidence is present
(manifest-only mode). All non-meta steps start as `blocked` because the
infrastructure has no evidence of completion. When the comparison manifest was
produced with `--execute` by `alice compare-launch-smoke` or
`alice run-first-lesson-readiness`, UI action steps move from `blocked` to
`ready` as each action is observed, and boundary steps move from `blocked` to
`ready` as their evidence status becomes `"present"`.

### Schema fields

| Field | Type | Description |
| --- | --- | --- |
| `schema_version` | string | Always `"eatme.first-lesson-grading-report/v1"`. |
| `scenario_id` | string | The scenario used to produce the underlying readiness report. |
| `steps` | array | Exactly 11 elements, one per canonical step, in curriculum order. |
| `steps[].id` | string | Step identifier matching the canonical step ID (hyphen-separated). |
| `steps[].name` | string | Human-readable step name from the curriculum. |
| `steps[].status` | string | One of `"ready"`, `"blocked"`, or `"not_yet_tested"`. |

The `steps` array is always exactly 11 elements. The order follows the
curriculum sequence defined in `CANONICAL_STEPS`, with boundary steps
interspersed among UI action steps and meta-boundary steps at the end.

## Plain output

Without `--json`, the report prints a human-readable summary:

```text
First-lesson grading report
Scenario: first-lessons-real-ui-actions

  verify-specific-alice-window    blocked
  activate-specific-alice-window  blocked
  select-project                  blocked
  place-object                    blocked
  edit-procedure-or-code-block    blocked
  run-world                       blocked
  save-project                    blocked
  visible-rendering               blocked
  grading                         not_yet_tested
  creative-assessment             not_yet_tested
  first-lesson-completion         not_yet_tested
```

## CLI usage

The command reuses the same `--manifest` and `--json` flags as
`alice check-lesson-session` and `alice check-lesson-readiness`:

```bash
cargo run -q -p eatme-cli -- alice first-lesson-grading-report \
  --manifest <path-to-comparison-manifest.json> \
  [--json]
```

| Option | Description |
| --- | --- |
| `--manifest <path>` | Path to a comparison manifest produced by `alice compare-launch-smoke` or `alice run-first-lesson-readiness`. Required. |
| `--json` | Emit structured JSON instead of plain text. |

The command exits 0 on success regardless of step statuses. A non-zero exit
indicates a manifest parse error or missing file, not a grading failure.

## Status mapping rules

The grading report applies these rules in order for each step:

1. **UI action steps**: Look up the action ID in the readiness report's
   `target_evidence` action assertions. If the action was observed and passed,
   status is `ready`. Otherwise, status is `blocked`.

2. **Boundary steps** (`select_project`, `visible_rendering`): Look up the
   boundary ID in the readiness report's `evidence_boundaries`. If the boundary
   status is `"present"`, status is `ready`. Otherwise, status is `blocked`.

3. **Meta-boundary steps** (`grading`, `creative-assessment`,
   `first-lesson-completion`): Always `not_yet_tested`. These boundaries
   represent curriculum outcomes that the current eatme infrastructure does not
   automate.

## Relationship to existing reports

The grading report is a read-only view over the
`LessonSessionReadinessReport` produced by `check_lesson_session_readiness()`.
It does not add, remove, or modify evidence. It exists to answer the per-step
question "can we prove this?" for each curriculum step, which the readiness
report does not answer directly.

| Report | Purpose | Scope |
| --- | --- | --- |
| `check-lesson-session` | Validate that a manifest carries a usable lesson-session contract | Contract structure |
| `check-lesson-readiness` | Full readiness evidence with shown/not-shown/unproven | All evidence surfaces |
| `first-lesson-grading-report` | Per-step completion status for Building a Scene | 11 canonical steps |

For the full readiness evidence contract, see
[First-Lesson Evidence Readiness](first-lesson-evidence-readiness.md).
For the lesson-session contract, see
[Lesson Session Readiness](lesson-session-readiness.md).

## Non-claims

The grading report does not:

- automate grading, creative assessment, or lesson completion;
- replace instructor judgment for pedagogical outcomes;
- prove full Alice UI automation;
- claim visible rendering correctness;
- claim Save completion beyond the bounded Save action evidence;
- produce evidence — it only reads evidence from the readiness report;
- fail the CLI exit code based on step statuses.

## Rust API

The library API is in `crates/eatme-alice/src/compare/grading_report.rs`:

```rust
use eatme_alice::first_lesson_grading_report;

let report = first_lesson_grading_report(&readiness);
assert_eq!(report.steps.len(), 11);
```

The `FirstLessonGradingReport` struct and `GradingStep` struct are re-exported
from `eatme_alice::compare`:

```rust
pub struct FirstLessonGradingReport {
    pub schema_version: String,
    pub scenario_id: String,
    pub steps: Vec<GradingStep>,
}

pub struct GradingStep {
    pub id: String,
    pub name: String,
    pub status: GradingStepStatus,
}
```

`first_lesson_grading_report(readiness: &LessonSessionReadinessReport) -> FirstLessonGradingReport`
maps the readiness report to the 11-step grading view. It does not call
`check_lesson_session_readiness()` internally — the caller supplies the
readiness report.

## Testing

The grading report tests are split into two sibling modules under
`crates/eatme-alice/src/compare/grading_report/` to stay within the 500-line
module-size gate:

| File | Responsibility |
| --- | --- |
| `grading_report/tests.rs` | Core contract tests: step count, schema version, canonical order, status mapping for UI actions and boundaries, empty-evidence defaults. Owns shared test helpers (`empty_readiness_report`, `readiness_with_all_evidence`, `find_step`, `test_boundary`). |
| `grading_report/edge_case_tests.rs` | Serialization and boundary edge-case tests: closed status set validation, JSON snake_case serialization, JSON round-trip fidelity, boundary `"invalid"` and `"blocked"` status mapping. |

Both modules are declared with `#[cfg(test)]` in `grading_report.rs` and share
the same parent imports. The `edge_case_tests` module imports shared helpers from
`super::tests` (e.g., `empty_readiness_report`, `find_step`,
`readiness_with_all_evidence`, `test_boundary`).

### What the tests verify

**Core tests** (`tests.rs`):

- Step count is always exactly 11.
- Schema string is `"eatme.first-lesson-grading-report/v1"`.
- Scenario ID is preserved from the readiness report.
- Step IDs and names match canonical curriculum order.
- Meta-boundary steps (`grading`, `creative-assessment`,
  `first-lesson-completion`) always report `not_yet_tested`.
- UI action steps map observed actions to `ready` and missing actions to
  `blocked`.
- Boundary steps map `"present"` to `ready` and missing to `blocked`.
- All non-meta steps are `blocked` when no evidence is present.

**Edge-case tests** (`edge_case_tests.rs`):

- Every step status is in the closed set (`Ready`, `Blocked`, `NotYetTested`).
- JSON serialization produces snake_case status values (`"ready"`,
  `"blocked"`, `"not_yet_tested"`).
- JSON round-trip preserves schema version, scenario ID, and step count.
- Boundary steps with `"invalid"` status map to `blocked`.
- Boundary steps with `"blocked"` status map to `blocked`.

### Running tests

Run grading report tests (both modules):

```bash
cargo test -p eatme-alice grading_report
```

Run only edge-case tests:

```bash
cargo test -p eatme-alice grading_report::edge_case_tests
```

Run the full quality gate:

```bash
TMPDIR=/tmp ./scripts/quality-gates.sh
```
