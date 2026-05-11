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
status is `"valid"`, `blocked` when the boundary exists but is not valid, and
`not_yet_tested` when the infrastructure cannot yet evaluate it.

| Step | Boundary ID | Evidence source |
| --- | --- | --- |
| Select project | `select_project` | Select Project scenario evidence from the comparison manifest. |
| Visible rendering | `visible_rendering` | Visible rendering scenario evidence observed during the run. |

### Meta-boundary steps

These steps represent curriculum outcomes that current infrastructure cannot
prove. They always report `not_yet_tested`. When infrastructure is extended to
evaluate these boundaries, their status mapping will change.

| Step | Boundary ID | Why not yet tested |
| --- | --- | --- |
| Grading | `grading` | Automated grading infrastructure does not exist. |
| Creative assessment | `creative_assessment` | Creative quality evaluation requires instructor judgment. |
| First-lesson completion | `first_lesson_completion` | Full lesson completion proof requires all prior steps plus assessment. |

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
  "schema": "eatme.first-lesson-grading-report/v1",
  "scenario_id": "first-lessons-real-ui-actions",
  "steps": [
    {
      "name": "verify-specific-alice-window",
      "status": "blocked"
    },
    {
      "name": "activate-specific-alice-window",
      "status": "blocked"
    },
    {
      "name": "place-object",
      "status": "blocked"
    },
    {
      "name": "edit-procedure-or-code-block",
      "status": "blocked"
    },
    {
      "name": "run-world",
      "status": "blocked"
    },
    {
      "name": "save-project",
      "status": "blocked"
    },
    {
      "name": "select_project",
      "status": "not_yet_tested"
    },
    {
      "name": "visible_rendering",
      "status": "not_yet_tested"
    },
    {
      "name": "grading",
      "status": "not_yet_tested"
    },
    {
      "name": "creative_assessment",
      "status": "not_yet_tested"
    },
    {
      "name": "first_lesson_completion",
      "status": "not_yet_tested"
    }
  ]
}
```

The example above shows the output when no execution evidence is present
(manifest-only mode). When the comparison manifest was produced with `--execute`
by `alice compare-launch-smoke` or `alice run-first-lesson-readiness`, UI action
steps move from `blocked` to `ready` as each action is observed, and boundary
steps move from `not_yet_tested` to `ready` or `blocked` as evidence is
evaluated.

### Schema fields

| Field | Type | Description |
| --- | --- | --- |
| `schema` | string | Always `"eatme.first-lesson-grading-report/v1"`. |
| `scenario_id` | string | The scenario used to produce the underlying readiness report. |
| `steps` | array | Exactly 11 elements, one per canonical step, in curriculum order. |
| `steps[].name` | string | Step identifier matching the UI action ID or boundary ID. |
| `steps[].status` | string | One of `"ready"`, `"blocked"`, or `"not_yet_tested"`. |

The `steps` array is always exactly 11 elements. The order is stable: UI action
steps first (in `REQUIRED_UI_ACTION_IDS` order), then boundary steps
(`select_project`, `visible_rendering`), then meta-boundary steps (`grading`,
`creative_assessment`, `first_lesson_completion`).

## Plain output

Without `--json`, the report prints a human-readable summary:

```text
First-lesson grading report
Scenario: first-lessons-real-ui-actions

  verify-specific-alice-window    blocked
  activate-specific-alice-window  blocked
  place-object                    blocked
  edit-procedure-or-code-block    blocked
  run-world                       blocked
  save-project                    blocked
  select_project                  not_yet_tested
  visible_rendering               not_yet_tested
  grading                         not_yet_tested
  creative_assessment             not_yet_tested
  first_lesson_completion         not_yet_tested
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
   `required_actions`. If the action was observed and passed, status is `ready`.
   Otherwise, status is `blocked`.

2. **Boundary steps** (`select_project`, `visible_rendering`): Look up the
   boundary ID in the readiness report's `evidence_boundaries`. If the boundary
   status is `"valid"`, status is `ready`. If the boundary is `"missing"` and
   the manifest contains no execution evidence for the relevant target (i.e.,
   the infrastructure never attempted evaluation), status is `not_yet_tested`.
   If the boundary is present but not valid after execution, status is `blocked`.

3. **Meta-boundary steps** (`grading`, `creative_assessment`,
   `first_lesson_completion`): Always `not_yet_tested`. These boundaries
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

let report = first_lesson_grading_report(manifest_path)?;
assert_eq!(report.steps.len(), 11);
```

The `FirstLessonGradingReport` struct and `GradingStep` struct are re-exported
from `eatme_alice::compare`:

```rust
pub struct FirstLessonGradingReport {
    pub schema: String,
    pub scenario_id: String,
    pub steps: Vec<GradingStep>,
}

pub struct GradingStep {
    pub name: String,
    pub status: String,
}
```

`first_lesson_grading_report(manifest_path: &Path) -> Result<FirstLessonGradingReport>`
calls `check_lesson_session_readiness()` internally and maps the result to the
11-step grading view.

## Testing

Unit tests verify:

- Step count is always exactly 11.
- Meta-boundary steps (`grading`, `creative_assessment`,
  `first_lesson_completion`) always report `not_yet_tested`.
- UI action steps map observed actions to `ready` and missing actions to
  `blocked`.
- Boundary steps map `"valid"` to `ready` and `"missing"` to the correct
  fallback status.
- The `steps` array order matches curriculum order.
- The schema string is `"eatme.first-lesson-grading-report/v1"`.

Run grading report tests:

```bash
cargo test -p eatme-alice grading_report
```

Run the full quality gate:

```bash
TMPDIR=/tmp ./scripts/quality-gates.sh
```
