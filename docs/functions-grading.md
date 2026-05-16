# Functions grading report

The functions grading report evaluates whether a student program built in the
`functions-mini-challenge` lesson contains the required AST constructs — user
function definitions, return statements, and function calls from procedures —
and whether the program survives a save/reopen round-trip. It extends the same
grading pipeline used by the
[Loops and Conditionals Grading Report](loops-and-conditionals-grading.md) and
the [Events and Collision Grading Report](events-and-collision-grading.md) with
AST-aware steps that inspect the in-memory program representation for
function-oriented constructs.

The grading report is a **structural readiness check**, not a creative grade. It
answers "does the student program define functions with return statements and
call them from procedures?" — not "is the program good?" For the boundary
between machine-assessable and human-review-needed aspects, see
[Creative Assessment Boundary](creative-assessment-boundary.md).

## Contents

- [Usage](#usage)
- [AST model](#ast-model)
- [Output schema](#output-schema)
- [Lesson steps](#lesson-steps)
- [Step dependency graph](#step-dependency-graph)
- [Status semantics](#status-semantics)
- [Module structure](#module-structure)
- [API reference](#api-reference)
- [Configuration](#configuration)
- [Examples](#examples)
- [E2E test](#e2e-test)
- [Real-Alice integration test](#real-alice-integration-test)
- [Troubleshooting](#troubleshooting)
- [Related documentation](#related-documentation)

## Usage

Run the functions grading report with JSON output:

```bash
cargo run -q -p eatme-cli -- assets grading-report \
  --lesson functions-mini-challenge --json
```

The command evaluates eight steps in dependency order:

1. **validate-assets** — calls `assets validate` against committed scenario and
   persona assets. No dependencies (root step).
2. **check-dependencies** — calls `deps check` for host tools required by real
   Alice launch smokes. No dependencies (root step).
3. **launch-smoke** — evaluates whether both prior steps passed. Depends on
   `validate-assets` and `check-dependencies`.
4. **create-function** — checks that the student's AST contains at least one
   `Function` definition. Depends on `launch-smoke`.
5. **add-return-statement** — checks that the student's AST contains at least
   one `ReturnStatement` in a function body. Depends on `create-function`.
6. **call-function-from-procedure** — checks that the student's AST contains at
   least one `FunctionCall` statement inside a procedure body. Depends on
   `add-return-statement`.
7. **run-world** — runs the student world and observes results. Depends on
   `call-function-from-procedure`.
8. **save-project** — saves and reopens the project, then verifies the AST
   survives the round-trip unchanged. Depends on `run-world`.

The command does not launch Alice or drive lesson interactions. It reports
whether the preconditions are satisfied and whether the deeper lesson
interaction steps are blocked or awaiting runtime execution.

## AST model

The `eatme-core` crate provides a recursive AST for student programs. Three
constructs — `Function`, `ReturnStatement`, and `FunctionCall` — represent
Alice's function-oriented concepts. These types were added alongside the existing
loop, conditional, and event variants and follow the same structural conventions.

### Type hierarchy

```text
Program
  ├── procedures: Vec<Procedure>
  │     ├── name: String
  │     ├── parameters: Vec<Parameter>
  │     └── body: Vec<Statement>
  │           ├── MethodCall { object, method, arguments }
  │           ├── CountLoop { count, body: Vec<Statement> }
  │           ├── IfElse { condition, if_body, else_body }
  │           ├── EventListener { event, body }
  │           ├── CollisionListener { object_a, object_b, body }
  │           ├── FunctionCall { name, arguments }
  │           ├── ReturnStatement { value }
  │           ├── VariableDeclaration { name, var_type, initial_value }
  │           └── VariableAssignment { name, value }
  ├── functions: Vec<Function>
  │     ├── name: String
  │     ├── return_type: String
  │     └── body: Vec<Statement>
  └── variable_declarations: Vec<VariableDeclaration>
```

### Rust types (new additions)

```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Function {
    pub name: String,
    pub return_type: String,
    pub body: Vec<Statement>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub param_type: String,
}
```

New `Statement` variants:

```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Statement {
    // ... existing variants ...
    FunctionCall {
        name: String,
        arguments: Vec<String>,
    },
    ReturnStatement {
        value: String,
    },
    VariableDeclaration {
        name: String,
        var_type: String,
        initial_value: Option<String>,
    },
    VariableAssignment {
        name: String,
        value: String,
    },
}
```

All new fields on `Program` use `#[serde(default)]` for backward compatibility.
Old JSON that does not contain `functions` or `variable_declarations` will
deserialize with empty vectors.

### Serde round-trip guarantee

Both new struct types and new statement variants survive JSON serialization and
deserialization without loss:

```rust
let json = serde_json::to_string(&program).unwrap();
let restored: Program = serde_json::from_str(&json).unwrap();
assert_eq!(program, restored);
```

The `#[serde(tag = "kind")]` attribute on `Statement` produces JSON with
`"kind": "FunctionCall"` or `"kind": "ReturnStatement"`. Unknown variants are
rejected at deserialization time.

## Output schema

The `--json` flag produces structured JSON using the same `GradingReport` schema
as all other grading reports:

```json
{
  "schema_version": "eatme.assets/grading/v1",
  "lesson": "functions-mini-challenge",
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
      "name": "create-function",
      "status": "blocked",
      "depends_on": ["launch-smoke"],
      "reason": "Blocked by: launch-smoke"
    },
    {
      "name": "add-return-statement",
      "status": "blocked",
      "depends_on": ["create-function"],
      "reason": "Blocked by: create-function"
    },
    {
      "name": "call-function-from-procedure",
      "status": "blocked",
      "depends_on": ["add-return-statement"],
      "reason": "Blocked by: add-return-statement"
    },
    {
      "name": "run-world",
      "status": "blocked",
      "depends_on": ["call-function-from-procedure"],
      "reason": "Blocked by: call-function-from-procedure"
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

| Field | Type | Description |
| --- | --- | --- |
| `schema_version` | string | Always `eatme.assets/grading/v1`. |
| `lesson` | string | Always `functions-mini-challenge`. |
| `passed` | bool | `true` only when all steps are `ready`. |
| `steps` | array | Ordered list of `StepGrade` objects. |
| `steps[].name` | string | Step identifier. |
| `steps[].status` | string | One of `ready`, `blocked`, or `not-yet-tested`. |
| `steps[].depends_on` | array of strings | Step names this step depends on. Empty array `[]` for root steps. |
| `steps[].reason` | string | Human-readable explanation of the status. |

## Lesson steps

The grading report evaluates eight steps for the `functions-mini-challenge`
scenario. The first three are **precondition steps** identical to all other
grading reports. The last five are **lesson interaction steps** specific to the
functions curriculum.

### Precondition steps

| Step | What it checks | Passes when |
| --- | --- | --- |
| `validate-assets` | Committed persona and scenario assets | `validate_assets()` returns `passed=true` |
| `check-dependencies` | Host tools for real Alice smoke runs | `check_dependencies()` returns `all_required_available=true` |
| `launch-smoke` | Preconditions for launching Alice | Both `validate-assets` and `check-dependencies` are `ready` |

### Lesson interaction steps

| Step | What it checks | With `Program` | Without `Program` |
| --- | --- | --- | --- |
| `create-function` | Program contains ≥1 `Function` in `functions` | `ready` if found, `blocked` if missing | `blocked` |
| `add-return-statement` | At least one function body contains a `ReturnStatement` | `ready` if found, `blocked` if missing | `blocked` |
| `call-function-from-procedure` | At least one procedure body contains a `FunctionCall` | `ready` if found, `blocked` if missing | `blocked` |
| `run-world` | Student world executes successfully | `not-yet-tested` (requires runtime) | `blocked` |
| `save-project` | Saved AST round-trips without loss | `ready` if round-trip passes, `blocked` if not | `blocked` |

The "With `Program`" column assumes all upstream dependencies are satisfied.
When any upstream step is `blocked`, downstream steps cascade to `blocked`
regardless of the `Program`. The lesson interaction steps are hardcoded in the
grading function — they do not appear in the scenario YAML.

When a student `Program` is provided to the grading function, the
`create-function` step checks whether `program.functions` is non-empty. If at
least one `Function` exists, the step reports `ready`. If the `functions` vector
is empty, it reports `blocked` with reason `"No Function found in student
program"`.

The `add-return-statement` step walks the body of each `Function` recursively
to find any `ReturnStatement` variant. If found, `ready`. If not, `blocked` with
reason `"No ReturnStatement found in any function body"`.

The `call-function-from-procedure` step walks the body of each `Procedure`
recursively to find any `FunctionCall` variant. If found, `ready`. If not,
`blocked` with reason `"No FunctionCall found in any procedure body"`.

The `run-world` step is **not** AST-aware — it requires runtime execution. When
all upstream dependencies are satisfied it reports `not-yet-tested`; when any
upstream step is blocked it cascades to `blocked`.

The `save-project` step serializes the `Program` to JSON, deserializes it back,
and compares the result to the original using `PartialEq`. If equal, `ready`.
If not, `blocked` with reason `"AST did not survive save/reopen round-trip"`.

## Step dependency graph

Steps form a linear dependency chain with two root nodes:

```text
validate-assets ─┐
                  ├─→ launch-smoke → create-function → add-return-statement
check-dependencies┘                                     │
                                                        ↓
                                     call-function-from-procedure → run-world → save-project
```

All eight steps form a single linear chain after the initial fan-in at
`launch-smoke`. Each subsequent step depends on exactly one predecessor. If any
step reports `blocked`, all downstream steps also report `blocked`. The
`not-yet-tested` status does **not** cascade — downstream steps evaluate
independently.

## Status semantics

The same three statuses used by all other grading reports apply here:

| Status | Meaning |
| --- | --- |
| `ready` | Preconditions met or AST check passed. |
| `blocked` | Preconditions failed or required AST construct missing. |
| `not-yet-tested` | Requires runtime execution. All upstream dependencies are satisfied. |

When a `Program` is provided, AST-aware steps (`create-function`,
`add-return-statement`, `call-function-from-procedure`, `save-project`) produce
`ready` or `blocked` based on AST inspection. When no `Program` is provided
(`None`), all lesson interaction steps produce `blocked` with reason `"No
student program provided"`.

The top-level `passed` field is `true` only when every step is `ready`.
Because `run-world` always produces `not-yet-tested` (it requires runtime
execution that the grading function does not perform), `passed` is always
`false` when called from the grading function alone. This is intentional — the
report confirms structural readiness, not lesson completion.

## Module structure

The functions grading code lives in a dedicated module,
`grading_report_functions`, extracted to keep all files under the 500-line
quality gate.

### File layout

```text
crates/eatme-assets/src/
├── grading_report.rs                          # Shared types, first-lesson + loops grading,
│                                              # pub(crate) helpers
├── grading_report_events.rs                   # Events grading
├── grading_report_functions.rs                # FunctionsGradingInput,
│                                              # grade_functions(),
│                                              # function-specific AST helpers
├── grading_report_functions_tests.rs          # Functions grading unit tests
└── lib.rs                                     # pub(crate) mod grading_report_functions;
                                               # re-exports FunctionsGradingInput
                                               # and grade_functions
```

### Shared helpers

The functions grading module reuses the same `pub(crate)` helpers from
`grading_report.rs`:

| Helper | Purpose |
| --- | --- |
| `build_preconditions` | Produces the three precondition `StepGrade`s |
| `cascade_blocked` | Creates a `StepGrade` with `Blocked` status |
| `no_program_chain` | Creates a chain of `Blocked` steps when no program is provided |
| `ast_check_step` | Creates a `StepGrade` based on whether an AST construct was found |

## API reference

### `FunctionsGradingInput`

Input struct for the functions grading function. Defined in
`grading_report_functions.rs`, re-exported from `eatme_assets`:

```rust
use eatme_core::ast::Program;

pub struct FunctionsGradingInput {
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

### `grade_functions`

Produces a `GradingReport` for the functions lesson. Defined in
`grading_report_functions.rs`, re-exported from `eatme_assets`:

```rust
use eatme_assets::{
    grade_functions, FunctionsGradingInput, GradingReport,
};

let report: GradingReport = grade_functions(FunctionsGradingInput {
    assets_valid: true,
    asset_reason: "All 93 scenario assets passed validation".into(),
    deps_available: true,
    deps_reason: "All required tools available".into(),
    student_program: Some(program),
});
```

The function is pure — it takes an input struct and returns a report. It does
not perform I/O, spawn processes, or access the filesystem.

### AST helper: `contains_function`

Checks whether `Program.functions` contains at least one entry:

```rust
fn contains_function(program: &Program) -> bool
```

### AST helper: `contains_return_statement`

Recursively walks all function bodies to find any `ReturnStatement` variant:

```rust
fn contains_return_statement(program: &Program) -> bool
```

### AST helper: `contains_function_call`

Recursively walks all procedure bodies to find any `FunctionCall` variant:

```rust
fn contains_function_call(program: &Program) -> bool
```

### Crate boundary

```text
eatme-cli (main.rs)
  ├── eatme_assets::validate_assets()    → AssetValidationReport
  ├── eatme_alice::check_dependencies()  → DependencyReport
  ├── eatme_core::ast::Program           → student program AST
  └── eatme_assets::grade_functions(FunctionsGradingInput { ... })
                                          → GradingReport (8 steps)
```

## Configuration

The functions grading report does not require real Alice desktop execution,
Node, or environment variables when used as a Rust API.

| Setting | Required | Purpose |
| --- | --- | --- |
| `TMPDIR=/tmp` | Recommended | Avoids Unix socket path length errors in deep worktrees. |
| `NODE_OPTIONS` | No | Not needed; no Node processes are launched. |
| `EATME_REAL_ALICE` | No | Not needed by the grading function itself; required by the real-Alice integration test. |

## Examples

### Build a minimal program and grade it

```rust
use eatme_core::ast::{Program, Procedure, Function, Statement};
use eatme_assets::{
    grade_functions, FunctionsGradingInput, StepStatus,
};

let program = Program {
    procedures: vec![Procedure {
        name: "myFirstMethod".into(),
        parameters: vec![],
        body: vec![
            Statement::FunctionCall {
                name: "getDistance".into(),
                arguments: vec!["this.cat".into(), "this.dog".into()],
            },
        ],
    }],
    functions: vec![Function {
        name: "getDistance".into(),
        return_type: "DecimalNumber".into(),
        body: vec![
            Statement::ReturnStatement {
                value: "this.cat getDistanceTo this.dog".into(),
            },
        ],
    }],
    variable_declarations: vec![],
};

let report = grade_functions(FunctionsGradingInput {
    assets_valid: true,
    asset_reason: "All 93 scenario assets passed validation".into(),
    deps_available: true,
    deps_reason: "All required tools available".into(),
    student_program: Some(program),
});

assert_eq!(report.lesson, "functions-mini-challenge");
assert_eq!(report.steps.len(), 8);
// create-function found the Function → ready
assert_eq!(report.steps[3].status, StepStatus::Ready);
// add-return-statement found the ReturnStatement → ready
assert_eq!(report.steps[4].status, StepStatus::Ready);
// call-function-from-procedure found the FunctionCall → ready
assert_eq!(report.steps[5].status, StepStatus::Ready);
// run-world requires runtime — not-yet-tested
assert_eq!(report.steps[6].status, StepStatus::NotYetTested);
// save-project round-trip passed → ready
assert_eq!(report.steps[7].status, StepStatus::Ready);
// passed is false because run-world is not-yet-tested
assert!(!report.passed);
```

### Grade with no student program

```rust
let report = grade_functions(FunctionsGradingInput {
    assets_valid: true,
    asset_reason: "All 93 scenario assets passed validation".into(),
    deps_available: true,
    deps_reason: "All required tools available".into(),
    student_program: None,
});

// All interaction steps blocked
assert_eq!(report.steps[3].status, StepStatus::Blocked);
assert!(report.steps[3].reason.contains("No student program provided"));
```

### Grade with missing function

```rust
let program = Program {
    procedures: vec![Procedure {
        name: "myFirstMethod".into(),
        parameters: vec![],
        body: vec![Statement::MethodCall {
            object: "this.cat".into(),
            method: "walk".into(),
            arguments: vec!["FORWARD".into(), "1.0".into()],
        }],
    }],
    functions: vec![],
    variable_declarations: vec![],
};

let report = grade_functions(FunctionsGradingInput {
    assets_valid: true,
    asset_reason: "All 93 scenario assets passed validation".into(),
    deps_available: true,
    deps_reason: "All required tools available".into(),
    student_program: Some(program),
});

// create-function: no Function → blocked
assert_eq!(report.steps[3].status, StepStatus::Blocked);
assert!(report.steps[3].reason.contains("No Function found"));
// downstream steps cascade to blocked
assert_eq!(report.steps[4].status, StepStatus::Blocked);
assert_eq!(report.steps[5].status, StepStatus::Blocked);
assert_eq!(report.steps[6].status, StepStatus::Blocked);
assert_eq!(report.steps[7].status, StepStatus::Blocked);
```

### Run tests from the command line

Run the functions grading unit tests:

```bash
TMPDIR=/tmp cargo test -p eatme-assets grading_report_functions -- --test-threads=1
```

Run the functions E2E test:

```bash
TMPDIR=/tmp cargo test -p eatme-alice --test functions_e2e -- --test-threads=1
```

Run the full quality gate:

```bash
TMPDIR=/tmp ./scripts/quality-gates.sh
```

## E2E test

The end-to-end test at `crates/eatme-alice/tests/functions_e2e.rs` validates the
full pipeline: AST construction → grading report → JSON serialization →
save/reopen round-trip.

### Test inventory

| Test | What it validates |
| --- | --- |
| `functions_grading_all_ready_with_complete_program` | Complete program with function, return statement, and function call. Precondition steps are `ready`. `create-function`, `add-return-statement`, `call-function-from-procedure`, and `save-project` are `ready`. `run-world` is `not-yet-tested`. |
| `functions_grading_blocked_without_program` | No student program (`None`). All 5 interaction steps report `blocked`. |
| `functions_grading_missing_function_blocks_downstream` | Program with procedures but no functions. The `create-function` step reports `blocked`, downstream steps cascade to `blocked`. |
| `functions_grading_missing_return_statement_blocks_downstream` | Program with a function but no return statement. The `add-return-statement` step reports `blocked`, downstream steps cascade to `blocked`. |
| `functions_grading_missing_function_call_blocks_downstream` | Program with function and return statement but no function call from a procedure. The `call-function-from-procedure` step reports `blocked`, downstream steps cascade to `blocked`. |
| `functions_ast_survives_json_round_trip` | Serialize a `Program` with functions to JSON and deserialize it. The restored AST equals the original. |
| `functions_grading_report_schema_version_and_lesson` | Schema version is `eatme.assets/grading/v1` and lesson is `functions-mini-challenge`. |
| `functions_grading_report_has_eight_steps` | Report always contains exactly 8 steps in the expected order. |

### Running the E2E test

```bash
TMPDIR=/tmp cargo test -p eatme-alice --test functions_e2e -- --test-threads=1
```

The E2E test does not launch Alice or require a display server. It exercises the
Rust API in-process using constructed AST fixtures.

## Real-Alice integration test

The functions grading pipeline is also exercised against a real Alice `.a3p`
starter project in the real-Alice integration test. See
[Real-Alice AST Grading Integration Tests](real-alice-ast-grading.md) for the
full documentation of the A3P parser and `EATME_REAL_ALICE=1` gated tests.

## Troubleshooting

### `cargo test` fails with "unresolved import `eatme_core::ast`"

The `eatme-core` crate must contain the `ast` module. Verify that
`crates/eatme-core/src/ast.rs` exists and `crates/eatme-core/src/lib.rs`
contains `pub mod ast`.

### Grading report shows 8 steps but all interaction steps are `blocked`

The `student_program` field is `None`. Provide a `Some(Program { ... })` to
enable AST inspection. When no program is provided, all lesson interaction steps
report `blocked` with reason `"No student program provided"`.

### AST round-trip fails

The `Statement` enum uses `#[serde(tag = "kind")]`. Manually constructed JSON
must include a `"kind"` field. New valid values are `"FunctionCall"` and
`"ReturnStatement"` in addition to the existing ones.

### Module too long (quality gate failure)

All Rust source modules must stay at or below 500 lines.

| File | Expected lines | Limit |
| --- | --- | --- |
| `crates/eatme-core/src/ast.rs` | ~90 | 500 |
| `crates/eatme-assets/src/grading_report_functions.rs` | ~140 | 500 |
| `crates/eatme-assets/src/grading_report_functions_tests.rs` | ~300 | 500 |
| `crates/eatme-alice/tests/functions_e2e.rs` | ~250 | 500 |

## Related documentation

- [Variables Grading Report](variables-grading.md) — the variables lesson
  grading report that uses the same AST extensions.
- [Parameters Grading Report](parameters-grading.md) — the parameters lesson
  grading report that uses the same AST extensions.
- [Loops and Conditionals Grading Report](loops-and-conditionals-grading.md) —
  the loops grading report this feature mirrors.
- [Events and Collision Grading Report](events-and-collision-grading.md) — the
  events grading report this feature mirrors.
- [Creative Assessment Boundary](creative-assessment-boundary.md) — the boundary
  between machine-assessable and human-review-needed aspects.
- [Real-Alice AST Grading Integration Tests](real-alice-ast-grading.md) — the
  A3P parser and real-Alice integration test that exercises all grading
  pipelines against real starter projects.
- [Validation and Quality Gates](validation-quality-gates.md) — the 500-line
  module size gate and other quality checks.
