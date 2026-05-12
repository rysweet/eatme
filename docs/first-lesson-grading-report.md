# First-lesson grading report

The `assets grading-report` command evaluates whether the host environment is
ready to execute the Building a Scene first-lesson scenario. It checks committed
asset validity, host dependency availability, launch-smoke preconditions, and
three deeper lesson interaction steps, then outputs a structured JSON grading
report with per-step status and explicit dependency tracking.

The grading report is a **readiness preflight**, not a lesson grade. It answers
"can we run the first lesson?" — not "did the student pass?" For the boundary
between machine-assessable and human-review-needed aspects, see
[Creative Assessment Boundary](creative-assessment-boundary.md).

## Contents

- [Usage](#usage)
- [Output schema](#output-schema)
- [Lesson steps](#lesson-steps)
- [Step dependency graph](#step-dependency-graph)
- [Status semantics](#status-semantics)
- [API reference](#api-reference)
- [Configuration](#configuration)
- [Examples](#examples)
- [Troubleshooting](#troubleshooting)
- [Related documentation](#related-documentation)

## Usage

Run the grading report with JSON output:

```bash
cargo run -q -p eatme-cli -- assets grading-report --json
```

The command evaluates six steps in dependency order:

1. **validate-assets** — calls `assets validate` against committed scenario and
   persona assets. No dependencies (root step).
2. **check-dependencies** — calls `deps check` for host tools required by real
   Alice launch smokes (Java, Maven, Xvfb, wmctrl, screenshot tools, etc.).
   No dependencies (root step).
3. **launch-smoke** — evaluates whether both prior steps passed. Depends on
   `validate-assets` and `check-dependencies`.
4. **place-object** — first lesson interaction step. Depends on `launch-smoke`.
   Reports `not-yet-tested` when unblocked (requires runtime execution).
5. **edit-code** — second lesson interaction step. Depends on `place-object`.
   Reports `not-yet-tested` when unblocked.
6. **run-world** — third lesson interaction step. Depends on `edit-code`.
   Reports `not-yet-tested` when unblocked.

The command does not launch Alice or drive lesson interactions. It reports
whether the preconditions for launching Alice are satisfied and whether the
deeper lesson interaction steps are blocked or awaiting runtime execution.

## Output schema

The `--json` flag produces structured JSON:

```json
{
  "schema_version": "eatme.assets/grading/v1",
  "lesson": "building-a-scene-first-world",
  "passed": false,
  "steps": [
    {
      "name": "validate-assets",
      "status": "ready",
      "depends_on": [],
      "reason": "All 93 scenario assets passed validation"
    },
    {
      "name": "check-dependencies",
      "status": "blocked",
      "depends_on": [],
      "reason": "Missing required tools: Xvfb, wmctrl"
    },
    {
      "name": "launch-smoke",
      "status": "blocked",
      "depends_on": ["validate-assets", "check-dependencies"],
      "reason": "Blocked by: check-dependencies"
    },
    {
      "name": "place-object",
      "status": "blocked",
      "depends_on": ["launch-smoke"],
      "reason": "Blocked by: launch-smoke"
    },
    {
      "name": "edit-code",
      "status": "blocked",
      "depends_on": ["place-object"],
      "reason": "Blocked by: place-object"
    },
    {
      "name": "run-world",
      "status": "blocked",
      "depends_on": ["edit-code"],
      "reason": "Blocked by: edit-code"
    }
  ]
}
```

| Field | Type | Description |
| --- | --- | --- |
| `schema_version` | string | Always `eatme.assets/grading/v1`. |
| `lesson` | string | The lesson scenario id. Always `building-a-scene-first-world`. |
| `passed` | bool | `true` only when all steps are `ready`. |
| `steps` | array | Ordered list of `StepGrade` objects. |
| `steps[].name` | string | Step identifier. Precondition step names match the scenario YAML step ids; lesson interaction step names are hardcoded in the grading function. |
| `steps[].status` | string | One of `ready`, `blocked`, or `not-yet-tested`. |
| `steps[].depends_on` | array of strings | Step names this step depends on. Empty array `[]` for root steps. |
| `steps[].reason` | string | Human-readable explanation of the status. |

The `depends_on` field is additive — it was not present in earlier versions of
the schema. Consumers that ignore unknown fields are unaffected.

Without `--json`, the command prints a plain-text summary:

```text
First-lesson grading: building-a-scene-first-world
  validate-assets: ready — All 93 scenario assets passed validation
  check-dependencies: blocked — Missing required tools: Xvfb, wmctrl
  launch-smoke: blocked — Blocked by: check-dependencies
  place-object: blocked — Blocked by: launch-smoke
  edit-code: blocked — Blocked by: place-object
  run-world: blocked — Blocked by: edit-code
Result: NOT READY
```

## Lesson steps

The grading report evaluates six steps for the `building-a-scene-first-world`
scenario. The first three are **precondition steps** that can be fully evaluated
from pre-computed results. The last three are **lesson interaction steps** that
require runtime execution to evaluate.

### Precondition steps

| Step | What it checks | Passes when |
| --- | --- | --- |
| `validate-assets` | Committed persona and scenario assets | `validate_assets()` returns `passed=true` |
| `check-dependencies` | Host tools for real Alice smoke runs | `check_dependencies()` returns `all_required_available=true` |
| `launch-smoke` | Preconditions for launching Alice | Both `validate-assets` and `check-dependencies` are `ready` |

### Lesson interaction steps

| Step | What it represents | Status when unblocked |
| --- | --- | --- |
| `place-object` | Placing a 3D object in the Alice scene | `not-yet-tested` |
| `edit-code` | Editing code in the Alice code editor | `not-yet-tested` |
| `run-world` | Running the completed Alice world | `not-yet-tested` |

Lesson interaction steps always report `not-yet-tested` when their upstream
dependencies are satisfied. They report `blocked` when any upstream dependency
is `blocked`. The grading report does not drive lesson interactions — it reports
whether the preconditions for each step are met.

The lesson interaction steps are hardcoded in the grading function — they do
not appear in the scenario YAML (`building-a-scene-first-world.yaml`), which
only defines the three precondition steps. The interaction steps represent the
alice.org curriculum's "Building a Scene" first-lesson activities.

## Step dependency graph

Steps form a linear dependency chain with two root nodes:

```text
validate-assets ─┐
                  ├─→ launch-smoke → place-object → edit-code → run-world
check-dependencies┘
```

The `depends_on` field on each step makes this graph explicit in the JSON
output. Consumers can use the dependency graph to:

- Determine which steps are actionable (all dependencies satisfied).
- Identify the root cause of a blocked step (trace back through `depends_on`).
- Visualize the lesson progression pipeline.

A step is `blocked` if any step in its `depends_on` list is `blocked`. A step
is `not-yet-tested` when all dependencies are `ready` or `not-yet-tested` and
the step itself requires runtime execution. A step is `ready` when all
dependencies are `ready` and the step's own check passes.

## Status semantics

Each step receives one of three statuses:

| Status | Meaning |
| --- | --- |
| `ready` | Preconditions met. The step can execute. |
| `blocked` | Preconditions failed. The reason field explains what is missing. |
| `not-yet-tested` | Step requires runtime execution to evaluate. All upstream dependencies are satisfied, but the step has not been executed. |

The three precondition steps (`validate-assets`, `check-dependencies`,
`launch-smoke`) produce `ready` or `blocked`. They never produce
`not-yet-tested` because they can be fully evaluated from pre-computed results.

The three lesson interaction steps (`place-object`, `edit-code`, `run-world`)
produce `not-yet-tested` or `blocked`. They produce `not-yet-tested` when all
upstream dependencies are satisfied (the preconditions for running the lesson
are met but the step has not been executed). They produce `blocked` when any
upstream dependency is blocked.

The top-level `passed` field is `true` only when every step is `ready`. Because
lesson interaction steps produce `not-yet-tested` rather than `ready`, `passed`
is `false` in the current report. This is intentional — the report confirms
readiness, not completion.

## API reference

The grading report is implemented in `eatme-assets` as a pure function with no
side effects beyond the validation and dependency checks it orchestrates.

### Types

```rust
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct GradingReport {
    pub schema_version: String,
    pub lesson: String,
    pub passed: bool,
    pub steps: Vec<StepGrade>,
}

#[derive(Clone, Debug, Serialize)]
pub struct StepGrade {
    pub name: String,
    pub status: StepStatus,
    pub depends_on: Vec<String>,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum StepStatus {
    #[serde(rename = "ready")]
    Ready,
    #[serde(rename = "blocked")]
    Blocked,
    #[serde(rename = "not-yet-tested")]
    NotYetTested,
}

pub struct GradingInput {
    pub assets_valid: bool,
    pub asset_reason: String,
    pub deps_available: bool,
    pub deps_reason: String,
}
```

The `depends_on` field on `StepGrade` is a `Vec<String>` containing the names
of steps that must be `ready` or `not-yet-tested` before this step can proceed.
Root steps have an empty vector. The field serializes as a JSON array.

### Function

```rust
pub fn grade_first_lesson_readiness(input: GradingInput) -> GradingReport
```

The function accepts a `GradingInput` with pre-computed results from asset
validation and dependency checking, then returns a `GradingReport` with all
six Building a Scene lesson steps graded. The function is deterministic and
performs no I/O.

`GradingInput` is not `Deserialize` — it is constructed by the CLI from the
results of `eatme_assets::validate_assets()` and
`eatme_alice::check_dependencies()`. This keeps the `eatme-assets` crate free
of any dependency on `eatme-alice`. No new `GradingInput` fields are needed
for the lesson interaction steps — their status is derived from the
precondition step results.

### Dependency propagation logic

The grading function propagates status through the dependency graph:

1. Root steps (`validate-assets`, `check-dependencies`) are graded from
   `GradingInput` fields.
2. `launch-smoke` checks its `depends_on` list. If any dependency is `Blocked`,
   `launch-smoke` is `Blocked` with a reason listing the blockers.
3. Lesson interaction steps (`place-object`, `edit-code`, `run-world`) check
   their `depends_on` list. If any dependency is `Blocked`, the step is
   `Blocked`. Otherwise the step is `NotYetTested` — it requires runtime
   execution that the grading report does not perform.

### Crate boundary

The `eatme-assets` crate owns the grading types and pure grading function. The
`eatme-cli` crate orchestrates the calls:

```text
eatme-cli (main.rs)
  ├── eatme_assets::validate_assets()    → AssetValidationReport
  ├── eatme_alice::check_dependencies()  → DependencyReport
  └── eatme_assets::grade_first_lesson_readiness(GradingInput { ... })
                                          → GradingReport (6 steps)
```

This boundary ensures `eatme-assets` does not depend on `eatme-alice`.

## Configuration

The grading report has no configuration beyond the repository root. It uses the
same asset discovery and dependency checking as the existing `assets validate`
and `deps check` commands.

| Parameter | Source | Default |
| --- | --- | --- |
| Repository root | Current working directory | `.` |
| `--json` flag | CLI argument | Off (plain text) |
| Lesson scenario | Hardcoded | `building-a-scene-first-world` |
| Precondition steps | Hardcoded | `validate-assets`, `check-dependencies`, `launch-smoke` |
| Lesson interaction steps | Hardcoded | `place-object`, `edit-code`, `run-world` |

## Examples

### All preconditions ready, lesson steps not yet tested

When assets are valid and all host dependencies are available:

```bash
cargo run -q -p eatme-cli -- assets grading-report --json
```

```json
{
  "schema_version": "eatme.assets/grading/v1",
  "lesson": "building-a-scene-first-world",
  "passed": false,
  "steps": [
    {
      "name": "validate-assets",
      "status": "ready",
      "depends_on": [],
      "reason": "All 93 scenario assets passed validation"
    },
    {
      "name": "check-dependencies",
      "status": "ready",
      "depends_on": [],
      "reason": "All required dependencies available"
    },
    {
      "name": "launch-smoke",
      "status": "ready",
      "depends_on": ["validate-assets", "check-dependencies"],
      "reason": "All preconditions met"
    },
    {
      "name": "place-object",
      "status": "not-yet-tested",
      "depends_on": ["launch-smoke"],
      "reason": "Requires runtime execution"
    },
    {
      "name": "edit-code",
      "status": "not-yet-tested",
      "depends_on": ["place-object"],
      "reason": "Requires runtime execution"
    },
    {
      "name": "run-world",
      "status": "not-yet-tested",
      "depends_on": ["edit-code"],
      "reason": "Requires runtime execution"
    }
  ]
}
```

The `passed` field is `false` because the lesson interaction steps are
`not-yet-tested`. The three precondition steps are `ready`, which means
the host environment satisfies all preconditions for launching the Building a
Scene first-lesson scenario. The lesson interaction steps require runtime
execution to complete.

### Blocked by missing dependencies

When host dependencies are missing, the blockage cascades through the
dependency graph:

```bash
cargo run -q -p eatme-cli -- assets grading-report --json
```

```json
{
  "schema_version": "eatme.assets/grading/v1",
  "lesson": "building-a-scene-first-world",
  "passed": false,
  "steps": [
    {
      "name": "validate-assets",
      "status": "ready",
      "depends_on": [],
      "reason": "All 93 scenario assets passed validation"
    },
    {
      "name": "check-dependencies",
      "status": "blocked",
      "depends_on": [],
      "reason": "Missing required tools: Xvfb, wmctrl"
    },
    {
      "name": "launch-smoke",
      "status": "blocked",
      "depends_on": ["validate-assets", "check-dependencies"],
      "reason": "Blocked by: check-dependencies"
    },
    {
      "name": "place-object",
      "status": "blocked",
      "depends_on": ["launch-smoke"],
      "reason": "Blocked by: launch-smoke"
    },
    {
      "name": "edit-code",
      "status": "blocked",
      "depends_on": ["place-object"],
      "reason": "Blocked by: place-object"
    },
    {
      "name": "run-world",
      "status": "blocked",
      "depends_on": ["edit-code"],
      "reason": "Blocked by: edit-code"
    }
  ]
}
```

### Blocked by invalid assets

When a committed scenario asset has validation errors:

```json
{
  "schema_version": "eatme.assets/grading/v1",
  "lesson": "building-a-scene-first-world",
  "passed": false,
  "steps": [
    {
      "name": "validate-assets",
      "status": "blocked",
      "depends_on": [],
      "reason": "Asset validation failed: 2 errors"
    },
    {
      "name": "check-dependencies",
      "status": "ready",
      "depends_on": [],
      "reason": "All required dependencies available"
    },
    {
      "name": "launch-smoke",
      "status": "blocked",
      "depends_on": ["validate-assets", "check-dependencies"],
      "reason": "Blocked by: validate-assets"
    },
    {
      "name": "place-object",
      "status": "blocked",
      "depends_on": ["launch-smoke"],
      "reason": "Blocked by: launch-smoke"
    },
    {
      "name": "edit-code",
      "status": "blocked",
      "depends_on": ["place-object"],
      "reason": "Blocked by: place-object"
    },
    {
      "name": "run-world",
      "status": "blocked",
      "depends_on": ["edit-code"],
      "reason": "Blocked by: edit-code"
    }
  ]
}
```

### Both preconditions blocked

```json
{
  "schema_version": "eatme.assets/grading/v1",
  "lesson": "building-a-scene-first-world",
  "passed": false,
  "steps": [
    {
      "name": "validate-assets",
      "status": "blocked",
      "depends_on": [],
      "reason": "Asset validation failed: 2 errors"
    },
    {
      "name": "check-dependencies",
      "status": "blocked",
      "depends_on": [],
      "reason": "Missing required tools: java, mvn"
    },
    {
      "name": "launch-smoke",
      "status": "blocked",
      "depends_on": ["validate-assets", "check-dependencies"],
      "reason": "Blocked by: validate-assets, check-dependencies"
    },
    {
      "name": "place-object",
      "status": "blocked",
      "depends_on": ["launch-smoke"],
      "reason": "Blocked by: launch-smoke"
    },
    {
      "name": "edit-code",
      "status": "blocked",
      "depends_on": ["place-object"],
      "reason": "Blocked by: place-object"
    },
    {
      "name": "run-world",
      "status": "blocked",
      "depends_on": ["edit-code"],
      "reason": "Blocked by: edit-code"
    }
  ]
}
```

### Plain text output (no --json)

```bash
cargo run -q -p eatme-cli -- assets grading-report
```

```text
First-lesson grading: building-a-scene-first-world
  validate-assets: ready — All 93 scenario assets passed validation
  check-dependencies: ready — All required dependencies available
  launch-smoke: ready — All preconditions met
  place-object: not-yet-tested — Requires runtime execution
  edit-code: not-yet-tested — Requires runtime execution
  run-world: not-yet-tested — Requires runtime execution
Result: NOT READY
```

The result is `NOT READY` because lesson interaction steps are
`not-yet-tested`. When all precondition steps are `ready`, the environment is
launch-ready, but the lesson has not been executed.

### Using in CI

The command exits with code 0 regardless of readiness status. Use `jq` to
gate CI pipelines on precondition readiness:

```bash
cargo run -q -p eatme-cli -- assets grading-report --json \
  | jq -e '[.steps[] | select(.depends_on | length == 0 or .name == "launch-smoke")] | all(.status == "ready")' > /dev/null
```

To check only that no steps are blocked (accepting `not-yet-tested`):

```bash
cargo run -q -p eatme-cli -- assets grading-report --json \
  | jq -e '[.steps[].status] | all(. != "blocked")' > /dev/null
```

### Querying the dependency graph

Extract the dependency graph as a list of edges:

```bash
cargo run -q -p eatme-cli -- assets grading-report --json \
  | jq '[.steps[] | {step: .name, depends_on}]'
```

```json
[
  {"step": "validate-assets", "depends_on": []},
  {"step": "check-dependencies", "depends_on": []},
  {"step": "launch-smoke", "depends_on": ["validate-assets", "check-dependencies"]},
  {"step": "place-object", "depends_on": ["launch-smoke"]},
  {"step": "edit-code", "depends_on": ["place-object"]},
  {"step": "run-world", "depends_on": ["edit-code"]}
]
```

## Troubleshooting

### "Asset validation failed" but `assets validate` passes

The grading report runs the same `validate_assets()` function. If `assets
validate --json` passes independently but the grading report shows `blocked`,
check that both commands are running from the same working directory (the
repository root).

### Dependencies show "blocked" on a CI runner

The grading report calls `check_dependencies()` which looks for host tools
like Java, Maven, Xvfb, wmctrl, and screenshot tools. CI runners without
desktop dependencies will correctly report `blocked`. This is expected — use
the grading report to confirm which tools are missing before attempting a real
Alice launch smoke. When `check-dependencies` is blocked, `launch-smoke` and
all lesson interaction steps are also blocked via dependency propagation.

### launch-smoke shows "ready" — does that mean Alice launched?

No. `ready` means both preconditions (`validate-assets` and
`check-dependencies`) passed. The grading report does not launch Alice.
The lesson interaction steps (`place-object`, `edit-code`, `run-world`) will
show `not-yet-tested` — they require an actual Alice session.

To prove launch readiness, run:

```bash
EATME_REAL_ALICE=1 cargo run -q -p eatme-cli -- alice launch-smoke \
  --alice-home "${ALICE_HOME}" \
  --scenario building-a-scene-first-world \
  --run-id local-grading-check \
  --runs-dir runs \
  --json \
  --no-memory \
  --offline-package
```

### Lesson interaction steps are always "not-yet-tested"

This is expected. The grading report evaluates readiness, not completion.
Lesson interaction steps (`place-object`, `edit-code`, `run-world`) require
a real Alice session to execute. The report confirms whether the preconditions
for reaching those steps are met. For the boundary between what can be
machine-assessed and what needs human review, see
[Creative Assessment Boundary](creative-assessment-boundary.md).

### 500-line module limit

The grading code is split across two source modules to stay within the
repository's 500-line quality gate: `grading_report.rs` (357 lines) contains
the first-lesson and loops grading functions plus shared helpers, and
`grading_report_events.rs` (189 lines) contains the events grading function.
Tests are split into dedicated test files, each under 500 lines.

For the full module map, shared helper contracts, and import patterns, see
[Grading Module Architecture](grading-module-architecture.md).

## Related documentation

- [Grading Module Architecture](grading-module-architecture.md) — Module
  layout, shared helpers, import patterns, and how to add new lesson grading.
- [Creative Assessment Boundary](creative-assessment-boundary.md) — What can
  be machine-assessed vs. what needs human review for Building a Scene.
- [CLI Usage](cli-usage.md) — Full command reference including `assets
  grading-report`.
- [Scenario Authoring](scenario-authoring.md) — How scenario YAML files
  define lesson steps.
- [Validation and Quality Gates](validation-quality-gates.md) — Repository
  quality gates including the 500-line module limit.
- [Alice Integration](alice-integration.md) — Real Alice launch smoke
  execution.
- [Deterministic Real-Alice Smoke Test](deterministic-real-alice-smoke-test.md)
  — The integration test that exercises launch-smoke end to end.
- [First-Lesson Vertical Slice](first-lesson-vertical-slice.md) — The
  first-lesson UI-action pipeline and evidence model.
