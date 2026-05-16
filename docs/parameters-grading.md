# Parameters grading report

The parameters grading report evaluates whether a student program built in the
`parameters-mini-challenge` lesson contains the required AST constructs —
parameterized procedure definitions and procedure calls with arguments — and
whether the program survives a save/reopen round-trip. It extends the same
grading pipeline used by the
[Functions Grading Report](functions-grading.md) with AST-aware steps that
inspect the in-memory program representation for parameter-oriented constructs.

The grading report is a **structural readiness check**, not a creative grade. It
answers "does the student program define procedures with parameters and call them
with arguments?" — not "is the program good?" For the boundary between
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

Run the parameters grading report with JSON output:

```bash
cargo run -q -p eatme-cli -- assets grading-report \
  --lesson parameters-mini-challenge --json
```

The command evaluates seven steps in dependency order:

1. **validate-assets** — calls `assets validate` against committed scenario and
   persona assets. No dependencies (root step).
2. **check-dependencies** — calls `deps check` for host tools required by real
   Alice launch smokes. No dependencies (root step).
3. **launch-smoke** — evaluates whether both prior steps passed. Depends on
   `validate-assets` and `check-dependencies`.
4. **create-parameterized-procedure** — checks that the student's AST contains
   at least one `Procedure` with a non-empty `parameters` list. Depends on
   `launch-smoke`.
5. **call-with-argument** — checks that the student's AST contains at least one
   `MethodCall` that invokes a parameterized procedure name with arguments.
   Depends on `create-parameterized-procedure`.
6. **run-world** — runs the student world and observes results. Depends on
   `call-with-argument`.
7. **save-project** — saves and reopens the project, then verifies the AST
   survives the round-trip unchanged. Depends on `run-world`.

The command does not launch Alice or drive lesson interactions. It reports
whether the preconditions are satisfied and whether the deeper lesson
interaction steps are blocked or awaiting runtime execution.

## AST model

The parameters grading pipeline uses the `Procedure.parameters` field and the
`Parameter` struct, both added to the AST alongside the function and variable
constructs.

### Parameter

Represents an Alice procedure parameter with a name and type:

```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub param_type: String,
}
```

Example:

```rust
Parameter {
    name: "speed".into(),
    param_type: "DecimalNumber".into(),
}
```

### Procedure.parameters

The `Procedure` struct gains a `parameters` field:

```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Procedure {
    pub name: String,
    #[serde(default)]
    pub parameters: Vec<Parameter>,
    pub body: Vec<Statement>,
}
```

The `#[serde(default)]` attribute ensures backward compatibility — old JSON
that does not contain a `parameters` field will deserialize with an empty vector.

### Parameterized procedure example

```rust
Procedure {
    name: "moveAnimal".into(),
    parameters: vec![
        Parameter {
            name: "animal".into(),
            param_type: "SJointedModel".into(),
        },
        Parameter {
            name: "distance".into(),
            param_type: "DecimalNumber".into(),
        },
    ],
    body: vec![Statement::MethodCall {
        object: "animal".into(),
        method: "move".into(),
        arguments: vec!["FORWARD".into(), "distance".into()],
    }],
}
```

## Output schema

The `--json` flag produces structured JSON using the same `GradingReport` schema
as all other grading reports:

```json
{
  "schema_version": "eatme.assets/grading/v1",
  "lesson": "parameters-mini-challenge",
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
      "name": "create-parameterized-procedure",
      "status": "blocked",
      "depends_on": ["launch-smoke"],
      "reason": "Blocked by: launch-smoke"
    },
    {
      "name": "call-with-argument",
      "status": "blocked",
      "depends_on": ["create-parameterized-procedure"],
      "reason": "Blocked by: create-parameterized-procedure"
    },
    {
      "name": "run-world",
      "status": "blocked",
      "depends_on": ["call-with-argument"],
      "reason": "Blocked by: call-with-argument"
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
| `lesson` | string | Always `parameters-mini-challenge`. |
| `passed` | bool | `true` only when all steps are `ready`. |
| `steps` | array | Ordered list of `StepGrade` objects. |
| `steps[].name` | string | Step identifier. |
| `steps[].status` | string | One of `ready`, `blocked`, or `not-yet-tested`. |
| `steps[].depends_on` | array of strings | Step names this step depends on. Empty array `[]` for root steps. |
| `steps[].reason` | string | Human-readable explanation of the status. |

## Lesson steps

The grading report evaluates seven steps for the `parameters-mini-challenge`
scenario. The first three are **precondition steps** identical to all other
grading reports. The last four are **lesson interaction steps** specific to the
parameters curriculum.

### Precondition steps

| Step | What it checks | Passes when |
| --- | --- | --- |
| `validate-assets` | Committed persona and scenario assets | `validate_assets()` returns `passed=true` |
| `check-dependencies` | Host tools for real Alice smoke runs | `check_dependencies()` returns `all_required_available=true` |
| `launch-smoke` | Preconditions for launching Alice | Both `validate-assets` and `check-dependencies` are `ready` |

### Lesson interaction steps

| Step | What it checks | With `Program` | Without `Program` |
| --- | --- | --- | --- |
| `create-parameterized-procedure` | At least one `Procedure` has non-empty `parameters` | `ready` if found, `blocked` if missing | `blocked` |
| `call-with-argument` | At least one `MethodCall` invokes a parameterized procedure with arguments | `ready` if found, `blocked` if missing | `blocked` |
| `run-world` | Student world executes successfully | `not-yet-tested` (requires runtime) | `blocked` |
| `save-project` | Saved AST round-trips without loss | `ready` if round-trip passes, `blocked` if not | `blocked` |

When a student `Program` is provided, the `create-parameterized-procedure` step
iterates over `program.procedures` and checks whether any procedure has
`!p.parameters.is_empty()`. If found, `ready`. If not, `blocked` with reason
`"No parameterized Procedure found in student program"`.

The `call-with-argument` step collects the names of all parameterized
procedures, then walks all procedure bodies to find a `MethodCall` whose
`method` or `object` matches a parameterized procedure name and whose
`arguments` are non-empty. If found, `ready`. If not, `blocked` with reason
`"No procedure call with arguments found"`.

## Step dependency graph

```text
validate-assets ─┐
                  ├─→ launch-smoke → create-parameterized-procedure → call-with-argument
check-dependencies┘                                                    │
                                                                       ↓
                                                                   run-world → save-project
```

All seven steps form a single linear chain after the initial fan-in at
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
├── grading_report_parameters.rs               # ParametersGradingInput,
│                                              # grade_parameters(),
│                                              # parameter-specific AST helpers
├── grading_report_parameters_tests.rs         # Parameters grading unit tests
└── lib.rs                                     # pub(crate) mod grading_report_parameters;
                                               # re-exports ParametersGradingInput
                                               # and grade_parameters
```

## API reference

### `ParametersGradingInput`

Input struct for the parameters grading function:

```rust
use eatme_core::ast::Program;

pub struct ParametersGradingInput {
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

### `grade_parameters`

Produces a `GradingReport` for the parameters lesson:

```rust
use eatme_assets::{
    grade_parameters, ParametersGradingInput, GradingReport,
};

let report: GradingReport = grade_parameters(ParametersGradingInput {
    assets_valid: true,
    asset_reason: "All 93 scenario assets passed validation".into(),
    deps_available: true,
    deps_reason: "All required tools available".into(),
    student_program: Some(program),
});
```

The function is pure — it takes an input struct and returns a report.

### AST helper: `has_parameterized_procedure`

Checks whether any procedure has non-empty `parameters`:

```rust
fn has_parameterized_procedure(program: &Program) -> bool
```

### AST helper: `has_call_with_argument`

Walks procedure bodies to find a `MethodCall` invoking a parameterized
procedure with non-empty arguments:

```rust
fn has_call_with_argument(program: &Program) -> bool
```

### Crate boundary

```text
eatme-cli (main.rs)
  ├── eatme_assets::validate_assets()    → AssetValidationReport
  ├── eatme_alice::check_dependencies()  → DependencyReport
  ├── eatme_core::ast::Program           → student program AST
  └── eatme_assets::grade_parameters(ParametersGradingInput { ... })
                                          → GradingReport (7 steps)
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
use eatme_core::ast::{Program, Procedure, Parameter, Statement};
use eatme_assets::{
    grade_parameters, ParametersGradingInput, StepStatus,
};

let program = Program {
    procedures: vec![
        Procedure {
            name: "moveAnimal".into(),
            parameters: vec![
                Parameter {
                    name: "animal".into(),
                    param_type: "SJointedModel".into(),
                },
                Parameter {
                    name: "distance".into(),
                    param_type: "DecimalNumber".into(),
                },
            ],
            body: vec![Statement::MethodCall {
                object: "animal".into(),
                method: "move".into(),
                arguments: vec!["FORWARD".into(), "distance".into()],
            }],
        },
        Procedure {
            name: "myFirstMethod".into(),
            parameters: vec![],
            body: vec![Statement::MethodCall {
                object: "this".into(),
                method: "moveAnimal".into(),
                arguments: vec!["this.cat".into(), "2.0".into()],
            }],
        },
    ],
    functions: vec![],
    variable_declarations: vec![],
};

let report = grade_parameters(ParametersGradingInput {
    assets_valid: true,
    asset_reason: "All 93 scenario assets passed validation".into(),
    deps_available: true,
    deps_reason: "All required tools available".into(),
    student_program: Some(program),
});

assert_eq!(report.lesson, "parameters-mini-challenge");
assert_eq!(report.steps.len(), 7);
// create-parameterized-procedure found moveAnimal with parameters → ready
assert_eq!(report.steps[3].status, StepStatus::Ready);
// call-with-argument found moveAnimal call with arguments → ready
assert_eq!(report.steps[4].status, StepStatus::Ready);
// run-world requires runtime — not-yet-tested
assert_eq!(report.steps[5].status, StepStatus::NotYetTested);
// save-project round-trip passed → ready
assert_eq!(report.steps[6].status, StepStatus::Ready);
// passed is false because run-world is not-yet-tested
assert!(!report.passed);
```

### Grade with no parameterized procedures

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

let report = grade_parameters(ParametersGradingInput {
    assets_valid: true,
    asset_reason: "All 93 scenario assets passed validation".into(),
    deps_available: true,
    deps_reason: "All required tools available".into(),
    student_program: Some(program),
});

// create-parameterized-procedure: no parameters → blocked
assert_eq!(report.steps[3].status, StepStatus::Blocked);
assert!(report.steps[3].reason.contains("No parameterized Procedure found"));
// downstream steps cascade
assert_eq!(report.steps[4].status, StepStatus::Blocked);
assert_eq!(report.steps[5].status, StepStatus::Blocked);
assert_eq!(report.steps[6].status, StepStatus::Blocked);
```

### Run tests from the command line

```bash
TMPDIR=/tmp cargo test -p eatme-assets grading_report_parameters -- --test-threads=1
TMPDIR=/tmp cargo test -p eatme-alice --test parameters_e2e -- --test-threads=1
TMPDIR=/tmp ./scripts/quality-gates.sh
```

## E2E test

The end-to-end test at `crates/eatme-alice/tests/parameters_e2e.rs` validates
the full pipeline: AST construction → grading report → JSON serialization →
save/reopen round-trip.

### Test inventory

| Test | What it validates |
| --- | --- |
| `parameters_grading_all_ready_with_complete_program` | Complete program with parameterized procedure and procedure call with arguments. All AST-aware steps are `ready`. `run-world` is `not-yet-tested`. |
| `parameters_grading_blocked_without_program` | No student program (`None`). All 4 interaction steps report `blocked`. |
| `parameters_grading_missing_parameterized_procedure_blocks_downstream` | Program with procedures that have no parameters. The `create-parameterized-procedure` step reports `blocked`, downstream steps cascade. |
| `parameters_grading_missing_call_with_argument_blocks_downstream` | Program with a parameterized procedure but no call invoking it with arguments. The `call-with-argument` step reports `blocked`, downstream steps cascade. |
| `parameters_ast_survives_json_round_trip` | Serialize a `Program` with parameterized procedures to JSON and deserialize it. The restored AST equals the original. |
| `parameters_grading_report_schema_version_and_lesson` | Schema version is `eatme.assets/grading/v1` and lesson is `parameters-mini-challenge`. |
| `parameters_grading_report_has_seven_steps` | Report always contains exactly 7 steps in the expected order. |

### Running the E2E test

```bash
TMPDIR=/tmp cargo test -p eatme-alice --test parameters_e2e -- --test-threads=1
```

## Real-Alice integration test

The parameters grading pipeline is also exercised against a real Alice `.a3p`
starter project. See
[Real-Alice AST Grading Integration Tests](real-alice-ast-grading.md).

## Troubleshooting

### `create-parameterized-procedure` reports `blocked` even though procedures exist

The step checks specifically for `!p.parameters.is_empty()`. A procedure with
an empty parameters list does not satisfy this check. The student must define
at least one procedure that accepts parameters.

### Module too long (quality gate failure)

| File | Expected lines | Limit |
| --- | --- | --- |
| `crates/eatme-assets/src/grading_report_parameters.rs` | ~100 | 500 |
| `crates/eatme-assets/src/grading_report_parameters_tests.rs` | ~250 | 500 |
| `crates/eatme-alice/tests/parameters_e2e.rs` | ~200 | 500 |

## Related documentation

- [Functions Grading Report](functions-grading.md) — the functions lesson
  grading report that uses the same AST extensions.
- [Variables Grading Report](variables-grading.md) — the variables lesson
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
