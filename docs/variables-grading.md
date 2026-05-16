# Variables grading report

The variables grading report evaluates whether a student program built in the
`variables-mini-challenge` lesson contains the required AST constructs —
variable declarations, variable usage in method calls, and variable modification
— and whether the program survives a save/reopen round-trip. It extends the same
grading pipeline used by the
[Functions Grading Report](functions-grading.md) with AST-aware steps that
inspect the in-memory program representation for variable-oriented constructs.

The grading report is a **structural readiness check**, not a creative grade. It
answers "does the student program declare variables, use them in method calls,
and modify them?" — not "is the program good?" For the boundary between
machine-assessable and human-review-needed aspects, see
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

Run the variables grading report with JSON output:

```bash
cargo run -q -p eatme-cli -- assets grading-report \
  --lesson variables-mini-challenge --json
```

The command evaluates eight steps in dependency order:

1. **validate-assets** — calls `assets validate` against committed scenario and
   persona assets. No dependencies (root step).
2. **check-dependencies** — calls `deps check` for host tools required by real
   Alice launch smokes. No dependencies (root step).
3. **launch-smoke** — evaluates whether both prior steps passed. Depends on
   `validate-assets` and `check-dependencies`.
4. **declare-variable** — checks that the student's AST contains at least one
   `VariableDeclaration` (either in `Program.variable_declarations` or as a
   `VariableDeclaration` statement in a procedure body). Depends on
   `launch-smoke`.
5. **use-variable-in-method** — checks that the student's AST contains at least
   one `MethodCall` whose arguments reference a declared variable name. Depends
   on `declare-variable`.
6. **modify-variable** — checks that the student's AST contains at least one
   `VariableAssignment` statement. Depends on `use-variable-in-method`.
7. **run-world** — runs the student world and observes results. Depends on
   `modify-variable`.
8. **save-project** — saves and reopens the project, then verifies the AST
   survives the round-trip unchanged. Depends on `run-world`.

The command does not launch Alice or drive lesson interactions. It reports
whether the preconditions are satisfied and whether the deeper lesson
interaction steps are blocked or awaiting runtime execution.

## AST model

The variables grading pipeline uses two `Statement` variants —
`VariableDeclaration` and `VariableAssignment` — added to the AST alongside
the function-oriented constructs.

### VariableDeclaration

Represents an Alice local variable declaration:

```rust
Statement::VariableDeclaration {
    name: "distance".into(),
    var_type: "DecimalNumber".into(),
    initial_value: Some("0.0".into()),
}
```

The `initial_value` field is `Option<String>` — Alice variables may be declared
without an initial value.

### VariableAssignment

Represents an assignment to an existing variable:

```rust
Statement::VariableAssignment {
    name: "distance".into(),
    value: "this.cat getDistanceTo this.dog".into(),
}
```

### Program.variable_declarations

Top-level variable declarations are also stored in
`Program.variable_declarations` as a `Vec<VariableDeclaration>` struct
(separate from `Statement`). This captures class-level or scene-level variables
that are not inside procedure bodies:

```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VariableDeclaration {
    pub name: String,
    pub var_type: String,
    pub initial_value: Option<String>,
}
```

The grading function checks both `Program.variable_declarations` and
`VariableDeclaration` statements within procedure bodies.

## Output schema

The `--json` flag produces structured JSON using the same `GradingReport` schema
as all other grading reports:

```json
{
  "schema_version": "eatme.assets/grading/v1",
  "lesson": "variables-mini-challenge",
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
      "name": "declare-variable",
      "status": "blocked",
      "depends_on": ["launch-smoke"],
      "reason": "Blocked by: launch-smoke"
    },
    {
      "name": "use-variable-in-method",
      "status": "blocked",
      "depends_on": ["declare-variable"],
      "reason": "Blocked by: declare-variable"
    },
    {
      "name": "modify-variable",
      "status": "blocked",
      "depends_on": ["use-variable-in-method"],
      "reason": "Blocked by: use-variable-in-method"
    },
    {
      "name": "run-world",
      "status": "blocked",
      "depends_on": ["modify-variable"],
      "reason": "Blocked by: modify-variable"
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
| `lesson` | string | Always `variables-mini-challenge`. |
| `passed` | bool | `true` only when all steps are `ready`. |
| `steps` | array | Ordered list of `StepGrade` objects. |
| `steps[].name` | string | Step identifier. |
| `steps[].status` | string | One of `ready`, `blocked`, or `not-yet-tested`. |
| `steps[].depends_on` | array of strings | Step names this step depends on. Empty array `[]` for root steps. |
| `steps[].reason` | string | Human-readable explanation of the status. |

## Lesson steps

The grading report evaluates eight steps for the `variables-mini-challenge`
scenario. The first three are **precondition steps** identical to all other
grading reports. The last five are **lesson interaction steps** specific to the
variables curriculum.

### Precondition steps

| Step | What it checks | Passes when |
| --- | --- | --- |
| `validate-assets` | Committed persona and scenario assets | `validate_assets()` returns `passed=true` |
| `check-dependencies` | Host tools for real Alice smoke runs | `check_dependencies()` returns `all_required_available=true` |
| `launch-smoke` | Preconditions for launching Alice | Both `validate-assets` and `check-dependencies` are `ready` |

### Lesson interaction steps

| Step | What it checks | With `Program` | Without `Program` |
| --- | --- | --- | --- |
| `declare-variable` | Program contains ≥1 variable declaration | `ready` if found, `blocked` if missing | `blocked` |
| `use-variable-in-method` | At least one `MethodCall` references a declared variable name in its arguments | `ready` if found, `blocked` if missing | `blocked` |
| `modify-variable` | Program contains ≥1 `VariableAssignment` statement | `ready` if found, `blocked` if missing | `blocked` |
| `run-world` | Student world executes successfully | `not-yet-tested` (requires runtime) | `blocked` |
| `save-project` | Saved AST round-trips without loss | `ready` if round-trip passes, `blocked` if not | `blocked` |

When a student `Program` is provided, the `declare-variable` step checks both
`program.variable_declarations` (top-level) and `VariableDeclaration` statements
within procedure bodies. If at least one declaration is found in either location,
the step reports `ready`. If none, `blocked` with reason `"No variable
declaration found in student program"`.

The `use-variable-in-method` step collects all declared variable names, then
walks procedure bodies to find any `MethodCall` whose `arguments` contain a
declared variable name. If found, `ready`. If not, `blocked` with reason `"No
variable used in method arguments"`.

The `modify-variable` step walks procedure bodies recursively to find any
`VariableAssignment` statement. If found, `ready`. If not, `blocked` with reason
`"No VariableAssignment found in student program"`.

## Step dependency graph

```text
validate-assets ─┐
                  ├─→ launch-smoke → declare-variable → use-variable-in-method
check-dependencies┘                                      │
                                                         ↓
                                              modify-variable → run-world → save-project
```

All eight steps form a single linear chain after the initial fan-in at
`launch-smoke`. If any step reports `blocked`, all downstream steps also report
`blocked`.

## Status semantics

| Status | Meaning |
| --- | --- |
| `ready` | Preconditions met or AST check passed. |
| `blocked` | Preconditions failed or required AST construct missing. |
| `not-yet-tested` | Requires runtime execution. All upstream dependencies are satisfied. |

The top-level `passed` field is `true` only when every step is `ready`.
Because `run-world` always produces `not-yet-tested`, `passed` is always `false`
when called from the grading function alone.

## Module structure

### File layout

```text
crates/eatme-assets/src/
├── grading_report.rs                          # Shared types, pub(crate) helpers
├── grading_report_variables.rs                # VariablesGradingInput,
│                                              # grade_variables(),
│                                              # variable-specific AST helpers
├── grading_report_variables_tests.rs          # Variables grading unit tests
└── lib.rs                                     # pub(crate) mod grading_report_variables;
                                               # re-exports VariablesGradingInput
                                               # and grade_variables
```

## API reference

### `VariablesGradingInput`

Input struct for the variables grading function:

```rust
use eatme_core::ast::Program;

pub struct VariablesGradingInput {
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

### `grade_variables`

Produces a `GradingReport` for the variables lesson:

```rust
use eatme_assets::{
    grade_variables, VariablesGradingInput, GradingReport,
};

let report: GradingReport = grade_variables(VariablesGradingInput {
    assets_valid: true,
    asset_reason: "All 93 scenario assets passed validation".into(),
    deps_available: true,
    deps_reason: "All required tools available".into(),
    student_program: Some(program),
});
```

The function is pure — it takes an input struct and returns a report. It does
not perform I/O, spawn processes, or access the filesystem.

### AST helper: `contains_var_declaration`

Checks both `Program.variable_declarations` and `VariableDeclaration` statements
within procedure bodies:

```rust
fn contains_var_declaration(program: &Program) -> bool
```

### AST helper: `contains_var_in_method_args`

Walks procedure bodies to find `MethodCall` arguments referencing declared
variable names:

```rust
fn contains_var_in_method_args(program: &Program) -> bool
```

### AST helper: `contains_var_assignment`

Recursively walks procedure bodies to find any `VariableAssignment` statement:

```rust
fn contains_var_assignment(program: &Program) -> bool
```

### Crate boundary

```text
eatme-cli (main.rs)
  ├── eatme_assets::validate_assets()    → AssetValidationReport
  ├── eatme_alice::check_dependencies()  → DependencyReport
  ├── eatme_core::ast::Program           → student program AST
  └── eatme_assets::grade_variables(VariablesGradingInput { ... })
                                          → GradingReport (8 steps)
```

## Configuration

| Setting | Required | Purpose |
| --- | --- | --- |
| `TMPDIR=/tmp` | Recommended | Avoids Unix socket path length errors in deep worktrees. |
| `NODE_OPTIONS` | No | Not needed; no Node processes are launched. |
| `EATME_REAL_ALICE` | No | Not needed by the grading function itself; required by the real-Alice integration test. |

## Examples

### Build a minimal program and grade it

```rust
use eatme_core::ast::{Program, Procedure, Statement, VariableDeclaration};
use eatme_assets::{
    grade_variables, VariablesGradingInput, StepStatus,
};

let program = Program {
    procedures: vec![Procedure {
        name: "myFirstMethod".into(),
        parameters: vec![],
        body: vec![
            Statement::VariableDeclaration {
                name: "distance".into(),
                var_type: "DecimalNumber".into(),
                initial_value: Some("0.0".into()),
            },
            Statement::MethodCall {
                object: "this.cat".into(),
                method: "move".into(),
                arguments: vec!["FORWARD".into(), "distance".into()],
            },
            Statement::VariableAssignment {
                name: "distance".into(),
                value: "distance + 1.0".into(),
            },
        ],
    }],
    functions: vec![],
    variable_declarations: vec![],
};

let report = grade_variables(VariablesGradingInput {
    assets_valid: true,
    asset_reason: "All 93 scenario assets passed validation".into(),
    deps_available: true,
    deps_reason: "All required tools available".into(),
    student_program: Some(program),
});

assert_eq!(report.lesson, "variables-mini-challenge");
assert_eq!(report.steps.len(), 8);
// declare-variable found the VariableDeclaration → ready
assert_eq!(report.steps[3].status, StepStatus::Ready);
// use-variable-in-method found "distance" in MethodCall arguments → ready
assert_eq!(report.steps[4].status, StepStatus::Ready);
// modify-variable found the VariableAssignment → ready
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
let report = grade_variables(VariablesGradingInput {
    assets_valid: true,
    asset_reason: "All 93 scenario assets passed validation".into(),
    deps_available: true,
    deps_reason: "All required tools available".into(),
    student_program: None,
});

assert_eq!(report.steps[3].status, StepStatus::Blocked);
assert!(report.steps[3].reason.contains("No student program provided"));
```

### Grade with declaration but no assignment

```rust
let program = Program {
    procedures: vec![Procedure {
        name: "myFirstMethod".into(),
        parameters: vec![],
        body: vec![
            Statement::VariableDeclaration {
                name: "count".into(),
                var_type: "WholeNumber".into(),
                initial_value: Some("0".into()),
            },
            Statement::MethodCall {
                object: "this.cat".into(),
                method: "say".into(),
                arguments: vec!["count".into()],
            },
        ],
    }],
    functions: vec![],
    variable_declarations: vec![],
};

let report = grade_variables(VariablesGradingInput {
    assets_valid: true,
    asset_reason: "All 93 scenario assets passed validation".into(),
    deps_available: true,
    deps_reason: "All required tools available".into(),
    student_program: Some(program),
});

// declare-variable → ready
assert_eq!(report.steps[3].status, StepStatus::Ready);
// use-variable-in-method → ready
assert_eq!(report.steps[4].status, StepStatus::Ready);
// modify-variable → blocked (no VariableAssignment)
assert_eq!(report.steps[5].status, StepStatus::Blocked);
assert!(report.steps[5].reason.contains("No VariableAssignment found"));
```

### Run tests from the command line

```bash
TMPDIR=/tmp cargo test -p eatme-assets grading_report_variables -- --test-threads=1
TMPDIR=/tmp cargo test -p eatme-alice --test variables_e2e -- --test-threads=1
TMPDIR=/tmp ./scripts/quality-gates.sh
```

## E2E test

The end-to-end test at `crates/eatme-alice/tests/variables_e2e.rs` validates
the full pipeline: AST construction → grading report → JSON serialization →
save/reopen round-trip.

### Test inventory

| Test | What it validates |
| --- | --- |
| `variables_grading_all_ready_with_complete_program` | Complete program with variable declaration, method usage, and assignment. All AST-aware steps are `ready`. `run-world` is `not-yet-tested`. |
| `variables_grading_blocked_without_program` | No student program (`None`). All 5 interaction steps report `blocked`. |
| `variables_grading_missing_declaration_blocks_downstream` | Program with no variable declarations. The `declare-variable` step reports `blocked`, downstream steps cascade. |
| `variables_grading_missing_assignment_blocks_downstream` | Program with declaration and method usage but no assignment. The `modify-variable` step reports `blocked`, downstream steps cascade. |
| `variables_ast_survives_json_round_trip` | Serialize a `Program` with variable constructs to JSON and deserialize it. The restored AST equals the original. |
| `variables_grading_report_schema_version_and_lesson` | Schema version is `eatme.assets/grading/v1` and lesson is `variables-mini-challenge`. |
| `variables_grading_report_has_eight_steps` | Report always contains exactly 8 steps in the expected order. |

### Running the E2E test

```bash
TMPDIR=/tmp cargo test -p eatme-alice --test variables_e2e -- --test-threads=1
```

## Real-Alice integration test

The variables grading pipeline is also exercised against a real Alice `.a3p`
starter project. See
[Real-Alice AST Grading Integration Tests](real-alice-ast-grading.md).

## Troubleshooting

### All interaction steps are `blocked` with "No student program provided"

The `student_program` field is `None`. Provide a `Some(Program { ... })` to
enable AST inspection.

### `use-variable-in-method` reports `blocked` even though a variable exists

The step checks that a declared variable name appears as an argument in a
`MethodCall`. If the variable is declared but never passed to a method, the
step reports `blocked`. Ensure at least one `MethodCall.arguments` entry
matches a declared variable name exactly.

### Module too long (quality gate failure)

| File | Expected lines | Limit |
| --- | --- | --- |
| `crates/eatme-assets/src/grading_report_variables.rs` | ~150 | 500 |
| `crates/eatme-assets/src/grading_report_variables_tests.rs` | ~300 | 500 |
| `crates/eatme-alice/tests/variables_e2e.rs` | ~250 | 500 |

## Related documentation

- [Functions Grading Report](functions-grading.md) — the functions lesson
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
  A3P parser and real-Alice integration test.
- [Validation and Quality Gates](validation-quality-gates.md) — the 500-line
  module size gate and other quality checks.
