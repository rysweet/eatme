# First-lesson grading report

The `assets grading-report` command evaluates whether the host environment is
ready to execute the Building a Scene first-lesson scenario. It checks committed
asset validity, host dependency availability, and launch-smoke preconditions,
then outputs a structured JSON grading report with per-step status.

The grading report is a **readiness preflight**, not a lesson grade. It answers
"can we run the first lesson?" — not "did the student pass?"

## Contents

- [Usage](#usage)
- [Output schema](#output-schema)
- [Lesson steps](#lesson-steps)
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

The command performs three checks in sequence:

1. **validate-assets** — calls `assets validate` against committed scenario and
   persona assets.
2. **check-dependencies** — calls `deps check` for host tools required by real
   Alice launch smokes (Java, Maven, Xvfb, wmctrl, screenshot tools, etc.).
3. **launch-smoke** — evaluates whether both prior steps passed. If they did,
   the step is `ready` (preconditions met for a real launch). If either failed,
   the step is `blocked`.

The command does not launch Alice. It reports whether the preconditions for
launching Alice are satisfied.

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
      "reason": "All 93 scenario assets passed validation"
    },
    {
      "name": "check-dependencies",
      "status": "blocked",
      "reason": "Missing required tools: Xvfb, wmctrl"
    },
    {
      "name": "launch-smoke",
      "status": "blocked",
      "reason": "Blocked: check-dependencies is not ready"
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
| `steps[].name` | string | Step identifier matching the scenario YAML step ids. |
| `steps[].status` | string | One of `ready`, `blocked`, or `not-yet-tested`. |
| `steps[].reason` | string | Human-readable explanation of the status. |

Without `--json`, the command prints a plain-text summary:

```text
First-lesson grading: building-a-scene-first-world
  validate-assets: ready — All 93 scenario assets passed validation
  check-dependencies: blocked — Missing required tools: Xvfb, wmctrl
  launch-smoke: blocked — Blocked: check-dependencies is not ready
Result: NOT READY
```

## Lesson steps

The grading report evaluates the three steps defined in the
`building-a-scene-first-world` scenario YAML:

| Step | What it checks | Passes when |
| --- | --- | --- |
| `validate-assets` | Committed persona and scenario assets | `validate_assets()` returns `passed=true` |
| `check-dependencies` | Host tools for real Alice smoke runs | `check_dependencies()` returns `all_required_available=true` |
| `launch-smoke` | Preconditions for launching Alice | Both `validate-assets` and `check-dependencies` are `ready` |

These steps come from the
[Building a Scene First World](scenario-authoring.md) scenario asset
(`assets/scenarios/eatme/building-a-scene-first-world.yaml`), which defines
the alice.org curriculum's "Building a Scene" first-lesson family.

## Status semantics

Each step receives one of three statuses:

| Status | Meaning |
| --- | --- |
| `ready` | Preconditions met. The step can execute. |
| `blocked` | Preconditions failed. The reason field explains what is missing. |
| `not-yet-tested` | Reserved for future steps that require runtime execution to evaluate. Not produced by the current three-step report. |

The `launch-smoke` step is `ready` when both preceding steps are `ready`.
This means the preconditions for launching Alice are satisfied — not that
Alice has actually been launched. When either preceding step is `blocked`,
`launch-smoke` is also `blocked` with a reason listing the blocking steps.

The `not-yet-tested` status is reserved for future steps that require
runtime execution to evaluate. The current three-step grading report does
not produce `not-yet-tested` because all steps can be evaluated from
pre-computed results.

The top-level `passed` field is `true` only when every step is `ready`.

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
    pub reason: String,
}

#[derive(Clone, Debug, Serialize)]
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

### Function

```rust
pub fn grade_first_lesson_readiness(input: GradingInput) -> GradingReport
```

The function accepts a `GradingInput` with pre-computed results from asset
validation and dependency checking, then returns a `GradingReport` with the
three Building a Scene lesson steps graded. The function is deterministic and
performs no I/O.

`GradingInput` is not `Deserialize` — it is constructed by the CLI from the
results of `eatme_assets::validate_assets()` and
`eatme_alice::check_dependencies()`. This keeps the `eatme-assets` crate free
of any dependency on `eatme-alice`.

### Crate boundary

The `eatme-assets` crate owns the grading types and pure grading function. The
`eatme-cli` crate orchestrates the calls:

```text
eatme-cli (main.rs)
  ├── eatme_assets::validate_assets()    → AssetValidationReport
  ├── eatme_alice::check_dependencies()  → DependencyReport
  └── eatme_assets::grade_first_lesson_readiness(GradingInput { ... })
                                          → GradingReport
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
| Step definitions | Scenario YAML | `validate-assets`, `check-dependencies`, `launch-smoke` |

## Examples

### All steps ready

When assets are valid and all host dependencies are available:

```bash
cargo run -q -p eatme-cli -- assets grading-report --json
```

```json
{
  "schema_version": "eatme.assets/grading/v1",
  "lesson": "building-a-scene-first-world",
  "passed": true,
  "steps": [
    {
      "name": "validate-assets",
      "status": "ready",
      "reason": "All 93 scenario assets passed validation"
    },
    {
      "name": "check-dependencies",
      "status": "ready",
      "reason": "All required dependencies available"
    },
    {
      "name": "launch-smoke",
      "status": "ready",
      "reason": "All preconditions met"
    }
  ]
}
```

The `passed` field is `true` because all three steps are `ready`. This means
the host environment satisfies all preconditions for launching the Building a
Scene first-lesson scenario. It does not mean Alice has been launched.

### Blocked by missing dependencies

When host dependencies are missing:

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
      "reason": "All 93 scenario assets passed validation"
    },
    {
      "name": "check-dependencies",
      "status": "blocked",
      "reason": "Missing required tools: Xvfb, wmctrl"
    },
    {
      "name": "launch-smoke",
      "status": "blocked",
      "reason": "Blocked: check-dependencies is not ready"
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
      "reason": "Asset validation failed: 2 errors"
    },
    {
      "name": "check-dependencies",
      "status": "ready",
      "reason": "All required dependencies available"
    },
    {
      "name": "launch-smoke",
      "status": "blocked",
      "reason": "Blocked: validate-assets is not ready"
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
      "reason": "Asset validation failed: 2 errors"
    },
    {
      "name": "check-dependencies",
      "status": "blocked",
      "reason": "Missing required tools: java, mvn"
    },
    {
      "name": "launch-smoke",
      "status": "blocked",
      "reason": "Blocked: validate-assets, check-dependencies are not ready"
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
Result: READY
```

### Using in CI

The command exits with code 0 regardless of readiness status. Use `jq` to
gate CI pipelines:

```bash
cargo run -q -p eatme-cli -- assets grading-report --json \
  | jq -e '.passed' > /dev/null
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
Alice launch smoke.

### launch-smoke shows "ready" — does that mean Alice launched?

No. `ready` means both preconditions (`validate-assets` and
`check-dependencies`) passed. The grading report does not launch Alice.
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

### 500-line module limit

The grading report module (`grading_report.rs`) targets under 300 lines.
Tests are split into unit tests (`grading_report_tests.rs`) and integration
tests (`grading_report_integration_tests.rs`), each under 300 lines. This
keeps all modules well within the repository's 500-line quality gate.

## Related documentation

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
