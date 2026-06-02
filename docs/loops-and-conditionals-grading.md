# Loops and conditionals grading report

The loops-and-conditionals grading report evaluates whether a student program
built in the `loops-and-conditionals-mini-challenge` lesson contains the
required AST constructs — counting loops and if/else conditionals — and whether
the program survives a save/reopen round-trip. It extends the same grading
pipeline used by the [First-Lesson Grading Report](first-lesson-grading-report.md)
with AST-aware steps that inspect the in-memory program representation.

The grading report is a **structural readiness check**, not a creative grade. It
answers "does the student program contain loops and conditionals that survive
persistence?" — not "is the program good?" For the boundary between
machine-assessable and human-review-needed aspects, see
[Creative Assessment Boundary](creative-assessment-boundary.md).

## Contents

- [Usage](#usage)
- [AST model](#ast-model)
- [Output schema](#output-schema)
- [Lesson steps](#lesson-steps)
- [Step dependency graph](#step-dependency-graph)
- [Status semantics](#status-semantics)
- [API reference](#api-reference)
- [Configuration](#configuration)
- [Examples](#examples)
- [E2E test](#e2e-test)
- [Troubleshooting](#troubleshooting)
- [Related documentation](#related-documentation)

## Usage

Run the loops-and-conditionals grading report with JSON output:

```bash
cargo run -q -p eatme-cli -- assets grading-report \
  --lesson loops-and-conditionals-mini-challenge --json
```

The command evaluates seven steps in dependency order:

1. **validate-assets** — calls `assets validate` against committed scenario and
   persona assets. No dependencies (root step).
2. **check-dependencies** — calls `deps check` for host tools required by real
   Alice launch smokes. No dependencies (root step).
3. **launch-smoke** — evaluates whether both prior steps passed. Depends on
   `validate-assets` and `check-dependencies`.
4. **build-counting-loop** — checks that the student's AST contains at least one
   `CountLoop` node. Depends on `launch-smoke`.
5. **add-conditional-branch** — checks that the student's AST contains at least
   one `IfElse` node. Depends on `build-counting-loop`.
6. **run-world** — runs the student world and observes results. Depends on
   `add-conditional-branch`.
7. **save-project** — saves and reopens the project, then verifies the AST
   survives the round-trip unchanged. Depends on `run-world`.

The command does not launch Alice or drive lesson interactions. It reports
whether the preconditions are satisfied and whether the deeper lesson
interaction steps are blocked or awaiting runtime execution.

## AST model

The `eatme-core` crate provides a recursive AST for student programs. The AST
is the source of truth for grading — the grading function inspects it directly
rather than parsing text or screenshots.

### Type hierarchy

```text
Program
  └── procedures: Vec<Procedure>
        ├── name: String
        └── body: Vec<Statement>
              ├── MethodCall { object, method, arguments }
              ├── CountLoop { count, body: Vec<Statement> }
              └── IfElse { condition, if_body: Vec<Statement>,
                           else_body: Vec<Statement> }
```

### Rust types

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Program {
    pub procedures: Vec<Procedure>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Procedure {
    pub name: String,
    pub body: Vec<Statement>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Statement {
    MethodCall {
        object: String,
        method: String,
        arguments: Vec<String>,
    },
    CountLoop {
        count: u32,
        body: Vec<Statement>,
    },
    IfElse {
        condition: String,
        if_body: Vec<Statement>,
        else_body: Vec<Statement>,
    },
}
```

The `#[serde(tag = "kind")]` attribute means the JSON discriminant field is
`"kind"` with values `"MethodCall"`, `"CountLoop"`, or `"IfElse"`. Unknown
variants are rejected at deserialization time.

### Serde round-trip guarantee

The AST survives JSON serialization and deserialization without loss:

```rust
let json = serde_json::to_string(&program).unwrap();
let restored: Program = serde_json::from_str(&json).unwrap();
assert_eq!(program, restored);
```

This round-trip property is what the `save-project` grading step verifies — if
a student saves their project and reopens it, the deserialized AST must equal
the original.

## Output schema

The `--json` flag produces structured JSON using the same `GradingReport` schema
as the first-lesson grading report:

```json
{
  "schema_version": "eatme.assets/grading/v1",
  "lesson": "loops-and-conditionals-mini-challenge",
  "passed": false,
  "steps": [
    {
      "name": "validate-assets",
      "status": "ready",
      "depends_on": [],
      "reason": "All 105 scenario assets passed validation"
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
      "name": "build-counting-loop",
      "status": "blocked",
      "depends_on": ["launch-smoke"],
      "reason": "Blocked by: launch-smoke"
    },
    {
      "name": "add-conditional-branch",
      "status": "blocked",
      "depends_on": ["build-counting-loop"],
      "reason": "Blocked by: build-counting-loop"
    },
    {
      "name": "run-world",
      "status": "blocked",
      "depends_on": ["add-conditional-branch"],
      "reason": "Blocked by: add-conditional-branch"
    },
    {
      "name": "save-project",
      "status": "blocked",
      "depends_on": ["run-world"],
      "reason": "Blocked by: run-world"
    }
  ]
}
```

The schema fields are identical to the first-lesson grading report:

| Field | Type | Description |
| --- | --- | --- |
| `schema_version` | string | Always `eatme.assets/grading/v1`. |
| `lesson` | string | Always `loops-and-conditionals-mini-challenge`. |
| `passed` | bool | `true` only when all steps are `ready`. |
| `steps` | array | Ordered list of `StepGrade` objects. |
| `steps[].name` | string | Step identifier. Precondition step names match the scenario YAML step ids; lesson interaction step names are hardcoded in the grading function. |
| `steps[].status` | string | One of `ready`, `blocked`, or `not-yet-tested`. |
| `steps[].depends_on` | array of strings | Step names this step depends on. Empty array `[]` for root steps. |
| `steps[].reason` | string | Human-readable explanation of the status. |

## Lesson steps

The grading report evaluates seven steps for the
`loops-and-conditionals-mini-challenge` scenario. The first three are
**precondition steps** identical to the first-lesson grading report. The last
four are **lesson interaction steps** specific to the loops-and-conditionals
curriculum.

### Precondition steps

| Step | What it checks | Passes when |
| --- | --- | --- |
| `validate-assets` | Committed persona and scenario assets | `validate_assets()` returns `passed=true` |
| `check-dependencies` | Host tools for real Alice smoke runs | `check_dependencies()` returns `all_required_available=true` |
| `launch-smoke` | Preconditions for launching Alice | Both `validate-assets` and `check-dependencies` are `ready` |

### Lesson interaction steps

| Step | What it checks | With `Program` | Without `Program` |
| --- | --- | --- | --- |
| `build-counting-loop` | Student AST contains ≥1 `CountLoop` node | `ready` if found, `blocked` if missing | `blocked` |
| `add-conditional-branch` | Student AST contains ≥1 `IfElse` node | `ready` if found, `blocked` if missing | `blocked` |
| `run-world` | Student world executes successfully | `not-yet-tested` (requires runtime) | `blocked` |
| `save-project` | Saved AST round-trips without loss | `ready` if round-trip passes, `blocked` if not | `blocked` |

The "With `Program`" column assumes all upstream dependencies are satisfied.
When any upstream step is `blocked`, downstream steps cascade to `blocked`
regardless of the `Program`. The lesson interaction steps are hardcoded in the
grading function — they do not appear in the scenario YAML
(`loops-and-conditionals-mini-challenge.yaml`), which only defines the three
precondition steps. The interaction steps represent the alice.org curriculum's
Loops and Conditionals mini-challenge activities.

When a student `Program` is provided to the grading function, the
`build-counting-loop` step walks the AST recursively to find any `CountLoop`
statement. If found, the step reports `ready`. If no `CountLoop` exists, it
reports `blocked` with reason `"No CountLoop found in student program"`.

The same logic applies to `add-conditional-branch` with `IfElse` nodes.

The `run-world` step is **not** AST-aware — it requires runtime execution to
evaluate. When all upstream dependencies are satisfied it reports
`not-yet-tested`; when any upstream step is blocked it cascades to `blocked`.

The `save-project` step serializes the `Program` to JSON, deserializes it back,
and compares the result to the original using `PartialEq`. If equal, `ready`.
If not, `blocked` with reason `"AST did not survive save/reopen round-trip"`.

## Step dependency graph

Steps form a linear dependency chain with two root nodes:

```text
validate-assets ─┐
                  ├─→ launch-smoke → build-counting-loop → add-conditional-branch
check-dependencies┘                                         │
                                                            ↓
                                                        run-world → save-project
```

All seven steps form a single linear chain after the initial fan-in at
`launch-smoke`. Each subsequent step depends on exactly one predecessor. If any
step reports `blocked`, all downstream steps also report `blocked`. The
`not-yet-tested` status does **not** cascade — downstream steps evaluate
independently.

## Status semantics

The same three statuses used by the first-lesson grading report apply here:

| Status | Meaning |
| --- | --- |
| `ready` | Preconditions met or AST check passed. |
| `blocked` | Preconditions failed or required AST construct missing. |
| `not-yet-tested` | Requires runtime execution. All upstream dependencies are satisfied. |

When a `Program` is provided, AST-aware steps (`build-counting-loop`,
`add-conditional-branch`, `save-project`) produce `ready` or `blocked` based on
AST inspection. When no `Program` is provided (`None`), all lesson interaction
steps produce `blocked` with reason `"No student program provided"`.

The top-level `passed` field is `true` only when every step is `ready`.
Because `run-world` always produces `not-yet-tested` (it requires runtime
execution the grading function does not perform), `passed` is always `false`
when called from the grading function alone. This is intentional — the report
confirms structural readiness, not lesson completion.

## API reference

### `LoopsGradingInput`

Input struct for the loops-and-conditionals grading function:

```rust
use eatme_core::ast::Program;

pub struct LoopsGradingInput {
    pub assets_valid: bool,
    pub asset_reason: String,
    pub deps_available: bool,
    pub deps_reason: String,
    pub student_program: Option<Program>,
}
```

| Field | Type | Description |
| --- | --- | --- |
| `assets_valid` | `bool` | Whether committed assets pass validation. |
| `asset_reason` | `String` | Human-readable reason from asset validation. |
| `deps_available` | `bool` | Whether host dependencies are available. |
| `deps_reason` | `String` | Human-readable reason from dependency check. |
| `student_program` | `Option<Program>` | The student's program AST, or `None` if not yet created. |

### `grade_loops_and_conditionals`

Produces a `GradingReport` for the loops-and-conditionals lesson:

```rust
use eatme_assets::grading_report::{
    grade_loops_and_conditionals, LoopsGradingInput, GradingReport,
};

let report: GradingReport = grade_loops_and_conditionals(LoopsGradingInput {
    assets_valid: true,
    asset_reason: "All 105 scenario assets passed validation".into(),
    deps_available: true,
    deps_reason: "All required tools available".into(),
    student_program: Some(program),
});
```

The function is pure — it takes an input struct and returns a report. It does
not perform I/O, spawn processes, or access the filesystem.

### AST helper: `contains_count_loop`

Recursively walks a `Program` to determine if any `CountLoop` statement exists:

```rust
fn contains_count_loop(program: &Program) -> bool
```

### AST helper: `contains_if_else`

Recursively walks a `Program` to determine if any `IfElse` statement exists:

```rust
fn contains_if_else(program: &Program) -> bool
```

Both helpers traverse nested bodies (`CountLoop.body`, `IfElse.if_body`,
`IfElse.else_body`) recursively.

### Dependency propagation logic

The grading function propagates status through the dependency graph:

1. Root steps (`validate-assets`, `check-dependencies`) are graded from
   `LoopsGradingInput` fields.
2. `launch-smoke` checks its `depends_on` list. If any dependency is `Blocked`,
   `launch-smoke` is `Blocked` with a reason listing the blockers.
3. AST-aware steps (`build-counting-loop`, `add-conditional-branch`,
   `save-project`) evaluate their AST checks when all upstream dependencies are
   satisfied. If any upstream dependency is `Blocked`, the step cascades to
   `Blocked`. If the upstream dependency is `NotYetTested`, the step evaluates
   independently (`not-yet-tested` does not cascade).
4. The `run-world` step is not AST-aware. It reports `NotYetTested` when all
   upstream dependencies are satisfied, and `Blocked` when any upstream
   dependency is `Blocked`.

### Crate boundary

The `eatme-assets` crate owns the grading types and pure grading function. The
`eatme-core` crate owns the AST types. The `eatme-cli` crate orchestrates:

```text
eatme-cli (main.rs)
  ├── eatme_assets::validate_assets()    → AssetValidationReport
  ├── eatme_alice::check_dependencies()  → DependencyReport
  ├── eatme_core::ast::Program           → student program AST
  └── eatme_assets::grade_loops_and_conditionals(LoopsGradingInput { ... })
                                          → GradingReport (7 steps)
```

The `eatme-core` crate provides the `ast` module (`Program`, `Procedure`,
`Statement`). The `eatme-assets` crate depends on `eatme-core` for AST types.
The `eatme-cli` crate depends on both. This boundary ensures `eatme-assets`
does not depend on `eatme-alice`.

## Configuration

The loops-and-conditionals grading report does not require real Alice desktop
execution, Node, or environment variables when used as a Rust API.

| Setting | Required | Purpose |
| --- | --- | --- |
| `TMPDIR=/tmp` | Recommended | Avoids Unix socket path length errors in deep worktrees. |
| `NODE_OPTIONS` | No | Not needed; no Node processes are launched. |
| `EATME_REAL_ALICE` | No | Not needed by the grading function itself; required by the `launch-smoke` scenario step if exercised end-to-end. |

## Examples

### Build a minimal program and grade it

```rust
use eatme_core::ast::{Program, Procedure, Statement};
use eatme_assets::grading_report::{
    grade_loops_and_conditionals, LoopsGradingInput, StepStatus,
};

let program = Program {
    procedures: vec![Procedure {
        name: "myFirstMethod".into(),
        body: vec![
            Statement::CountLoop {
                count: 3,
                body: vec![Statement::MethodCall {
                    object: "this.cat".into(),
                    method: "walk".into(),
                    arguments: vec!["FORWARD".into(), "1.0".into()],
                }],
            },
            Statement::IfElse {
                condition: "this.cat isCloseTo this.dog".into(),
                if_body: vec![Statement::MethodCall {
                    object: "this.cat".into(),
                    method: "say".into(),
                    arguments: vec!["\"Hello!\"".into()],
                }],
                else_body: vec![Statement::MethodCall {
                    object: "this.cat".into(),
                    method: "think".into(),
                    arguments: vec!["\"Hmm...\"".into()],
                }],
            },
        ],
    }],
};

let report = grade_loops_and_conditionals(LoopsGradingInput {
    assets_valid: true,
    asset_reason: "All 105 scenario assets passed validation".into(),
    deps_available: true,
    deps_reason: "All required tools available".into(),
    student_program: Some(program),
});

assert_eq!(report.lesson, "loops-and-conditionals-mini-challenge");
assert_eq!(report.steps.len(), 7);
// build-counting-loop found the CountLoop → ready
assert_eq!(report.steps[3].status, StepStatus::Ready);
// add-conditional-branch found the IfElse → ready
assert_eq!(report.steps[4].status, StepStatus::Ready);
// run-world requires runtime — not-yet-tested
assert_eq!(report.steps[5].status, StepStatus::NotYetTested);
// save-project round-trip passed → ready
assert_eq!(report.steps[6].status, StepStatus::Ready);
// passed is false because run-world is not-yet-tested
assert!(!report.passed);
```

### Grade with no student program

```rust
let report = grade_loops_and_conditionals(LoopsGradingInput {
    assets_valid: true,
    asset_reason: "All 105 scenario assets passed validation".into(),
    deps_available: true,
    deps_reason: "All required tools available".into(),
    student_program: None,
});

// All interaction steps blocked
assert_eq!(report.steps[3].status, StepStatus::Blocked);
assert!(report.steps[3].reason.contains("No student program provided"));
```

### Grade with missing loop construct

```rust
let program = Program {
    procedures: vec![Procedure {
        name: "myFirstMethod".into(),
        body: vec![Statement::IfElse {
            condition: "this.cat isCloseTo this.dog".into(),
            if_body: vec![],
            else_body: vec![],
        }],
    }],
};

let report = grade_loops_and_conditionals(LoopsGradingInput {
    assets_valid: true,
    asset_reason: "All 105 scenario assets passed validation".into(),
    deps_available: true,
    deps_reason: "All required tools available".into(),
    student_program: Some(program),
});

// build-counting-loop: no CountLoop → blocked
assert_eq!(report.steps[3].status, StepStatus::Blocked);
assert!(report.steps[3].reason.contains("No CountLoop found"));
// add-conditional-branch: blocked cascades from build-counting-loop
assert_eq!(report.steps[4].status, StepStatus::Blocked);
// run-world and save-project: blocked cascades
assert_eq!(report.steps[5].status, StepStatus::Blocked);
assert_eq!(report.steps[6].status, StepStatus::Blocked);
```

### Verify AST round-trip

```rust
use eatme_core::ast::Program;

let program = Program { procedures: vec![/* ... */] };
let json = serde_json::to_string(&program).unwrap();
let restored: Program = serde_json::from_str(&json).unwrap();
assert_eq!(program, restored);
```

### Run tests from the command line

Run the AST model tests:

```bash
TMPDIR=/tmp cargo test -p eatme-core -- --test-threads=1
```

Run the loops grading unit tests:

```bash
TMPDIR=/tmp cargo test -p eatme-assets grading_report_loops -- --test-threads=1
```

Run the loops-and-conditionals E2E test:

```bash
TMPDIR=/tmp cargo test -p eatme-alice --test loops_and_conditionals_e2e -- --test-threads=1
```

Run the full quality gate:

```bash
TMPDIR=/tmp ./scripts/quality-gates.sh
```

### Plain text output (no --json)

```bash
cargo run -q -p eatme-cli -- assets grading-report \
  --lesson loops-and-conditionals-mini-challenge
```

```text
Loops grading: loops-and-conditionals-mini-challenge
  validate-assets: ready — All 105 scenario assets passed validation
  check-dependencies: blocked — Missing required tools: Xvfb, wmctrl
  launch-smoke: blocked — Blocked by: check-dependencies
  build-counting-loop: blocked — Blocked by: launch-smoke
  add-conditional-branch: blocked — Blocked by: build-counting-loop
  run-world: blocked — Blocked by: add-conditional-branch
  save-project: blocked — Blocked by: run-world
Result: NOT READY
```

The result is `NOT READY` because lesson interaction steps are blocked by
missing host dependencies. When all precondition steps are `ready` and a
complete program is provided, the result is still `NOT READY` because
`run-world` reports `not-yet-tested`.

## E2E test

The end-to-end test at `crates/eatme-alice/tests/loops_and_conditionals_e2e.rs`
validates the full pipeline: AST construction → grading report → JSON
serialization → save/reopen round-trip.

### Test inventory

| Test | What it validates |
| --- | --- |
| `loops_grading_all_ready_with_complete_program` | Complete program with loops and conditionals. Precondition steps are `ready`. `build-counting-loop`, `add-conditional-branch`, and `save-project` are `ready`. `run-world` is `not-yet-tested`. |
| `loops_grading_blocked_without_program` | No student program (`None`). All 4 interaction steps report `blocked`. |
| `loops_grading_missing_loop_blocks_downstream` | Program with `IfElse` but no `CountLoop`. The `build-counting-loop` step reports `blocked`, downstream steps cascade to `blocked`. |
| `loops_grading_missing_conditional_blocks_downstream` | Program with `CountLoop` but no `IfElse`. The `add-conditional-branch` step reports `blocked`, downstream steps cascade to `blocked`. |
| `ast_survives_json_round_trip` | Serialize a `Program` to JSON and deserialize it. The restored AST equals the original. |
| `grading_report_schema_version_and_lesson` | Schema version is `eatme.assets/grading/v1` and lesson is `loops-and-conditionals-mini-challenge`. |
| `grading_report_has_seven_steps` | Report always contains exactly 7 steps in the expected order. |

### Running the E2E test

```bash
TMPDIR=/tmp cargo test -p eatme-alice --test loops_and_conditionals_e2e -- --test-threads=1
```

The E2E test does not launch Alice or require a display server. It exercises the
Rust API in-process using constructed AST fixtures.

### Real-Alice AST structure test

In addition to synthetic-fixture E2E tests, the
`real_alice_ast_structure_loops_and_conditionals` test in
`crates/eatme-alice/tests/real_ast_grading.rs` validates the loops pipeline
against a real `.a3p` starter project. This test independently asserts that the
parsed AST contains `IfElse` and lacks `CountLoop` before running the grading
pipeline — catching parser regressions that synthetic fixtures cannot.

```bash
EATME_REAL_ALICE=1 cargo test -p eatme-alice --test real_ast_grading \
  real_alice_ast_structure_loops_and_conditionals -- --nocapture
```

For full details, see
[Real-Alice Grading Integration Tests](real-alice-grading-integration-tests.md).

## Troubleshooting

### `cargo test` fails with "unresolved import `eatme_core::ast`"

The `eatme-core` crate must contain the `ast` module. Verify that
`crates/eatme-core/src/ast.rs` exists and `crates/eatme-core/src/lib.rs`
contains `pub mod ast`.

### `cargo test` fails with "unresolved import `eatme_assets`" in eatme-alice tests

The `eatme-assets` crate must be listed as a `[dev-dependencies]` entry in
`crates/eatme-alice/Cargo.toml`.

### Grading report shows 7 steps but all interaction steps are `blocked`

The `student_program` field is `None`. Provide a `Some(Program { ... })` to
enable AST inspection. When no program is provided, all lesson interaction steps
report `blocked` with reason `"No student program provided"`.

### AST round-trip fails

The `Statement` enum uses `#[serde(tag = "kind")]`. Manually constructed JSON
must include a `"kind"` field with one of `"MethodCall"`, `"CountLoop"`, or
`"IfElse"`. Missing or misspelled `"kind"` values cause deserialization failure.

### Module too long (quality gate failure)

All Rust source modules must stay at or below 500 lines. The grading code is
split across `grading_report.rs` (357 lines) and `grading_report_events.rs`
(189 lines). The loops grading function lives in `grading_report.rs` alongside
the first-lesson function and shared helpers. For the full module map and how
to add new lesson grading functions, see
[Grading Module Architecture](grading-module-architecture.md).

| File | Expected lines | Limit |
| --- | --- | --- |
| `crates/eatme-core/src/ast.rs` | ~50 | 500 |
| `crates/eatme-core/src/ast_tests.rs` | ~417 | 500 |
| `crates/eatme-assets/src/grading_report.rs` | ~357 | 500 |
| `crates/eatme-assets/src/grading_report_events.rs` | ~189 | 500 |
| `crates/eatme-assets/src/grading_report_loops_tests.rs` | ~497 | 500 |
| `crates/eatme-alice/tests/loops_and_conditionals_e2e.rs` | ~238 | 500 |

If `grading_report.rs` exceeds 500 lines, extract the loops grading function
into a separate `grading_report_loops.rs` module following the same pattern
used for the events extraction.

## Related documentation

- [Grading Module Architecture](grading-module-architecture.md) — Module
  layout, shared helpers, import patterns, and how to add new lesson grading.
- [First-Lesson Grading Report](first-lesson-grading-report.md) — the original
  grading report for the Building a Scene first-lesson scenario.
- [Events and Collision Grading](events-and-collision-grading.md) — the events
  grading report extracted into its own module.
- [Creative Assessment Boundary](creative-assessment-boundary.md) — the
  boundary between machine-assessable and human-review-needed aspects.
- [Save/reopen Readiness](save-reopen-readiness.md) — the evidence contract for
  save/reopen persistence.
- [Student Lesson E2E Tests](student-lesson-e2e-tests.md) — the existing
  student lesson E2E test patterns this feature follows.
- [Validation and Quality Gates](validation-quality-gates.md) — the 500-line
  module size gate and other quality checks.
- [Scenario Authoring](scenario-authoring.md) — how to author scenario YAML
  files including the `loops-and-conditionals-mini-challenge` scenario.
