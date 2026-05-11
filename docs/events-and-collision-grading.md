# Events and collision grading report

The events-and-collision grading report evaluates whether a student program
built in the `events-collision-proximity-game` lesson contains the required AST
constructs — event listeners and collision listeners — and whether the program
survives a save/reopen round-trip. It extends the same grading pipeline used by
the [First-Lesson Grading Report](first-lesson-grading-report.md) and the
[Loops and Conditionals Grading Report](loops-and-conditionals-grading.md) with
AST-aware steps that inspect the in-memory program representation for
trigger-driven constructs.

The grading report is a **structural readiness check**, not a creative grade. It
answers "does the student program contain event listeners and collision
listeners that survive persistence?" — not "is the program good?" For the
boundary between machine-assessable and human-review-needed aspects, see
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
- [Troubleshooting](#troubleshooting)
- [Related documentation](#related-documentation)

## Usage

Run the events-and-collision grading report with JSON output:

```bash
cargo run -q -p eatme-cli -- assets grading-report \
  --lesson events-collision-proximity-game --json
```

The command evaluates seven steps in dependency order:

1. **validate-assets** — calls `assets validate` against committed scenario and
   persona assets. No dependencies (root step).
2. **check-dependencies** — calls `deps check` for host tools required by real
   Alice launch smokes. No dependencies (root step).
3. **launch-smoke** — evaluates whether both prior steps passed. Depends on
   `validate-assets` and `check-dependencies`.
4. **add-event-listener** — checks that the student's AST contains at least one
   `EventListener` node. Depends on `launch-smoke`.
5. **add-collision-listener** — checks that the student's AST contains at least
   one `CollisionListener` node. Depends on `add-event-listener`.
6. **run-world** — runs the student world and observes results. Depends on
   `add-collision-listener`.
7. **save-project** — saves and reopens the project, then verifies the AST
   survives the round-trip unchanged. Depends on `run-world`.

The command does not launch Alice or drive lesson interactions. It reports
whether the preconditions are satisfied and whether the deeper lesson
interaction steps are blocked or awaiting runtime execution.

## AST model

The `eatme-core` crate provides a recursive AST for student programs. Two
variants — `EventListener` and `CollisionListener` — represent Alice's
trigger-driven constructs. These variants were added alongside the existing
`CountLoop` and `IfElse` variants and follow the same structural conventions.

### Type hierarchy

```text
Program
  └── procedures: Vec<Procedure>
        ├── name: String
        └── body: Vec<Statement>
              ├── MethodCall { object, method, arguments }
              ├── CountLoop { count, body: Vec<Statement> }
              ├── IfElse { condition, if_body: Vec<Statement>,
                           else_body: Vec<Statement> }
              ├── EventListener { event, body: Vec<Statement> }
              └── CollisionListener { object_a, object_b,
                                      body: Vec<Statement> }
```

### Rust types

```rust
use serde::{Deserialize, Serialize};

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
    EventListener {
        event: String,
        body: Vec<Statement>,
    },
    CollisionListener {
        object_a: String,
        object_b: String,
        body: Vec<Statement>,
    },
}
```

### EventListener

Represents an Alice event listener that fires when a named event occurs (e.g.,
`"SceneActivated"`, `"KeyPress"`, `"MouseClick"`). The `event` field names the
trigger. The `body` contains the statements to execute when the event fires.

```rust
Statement::EventListener {
    event: "SceneActivated".into(),
    body: vec![Statement::MethodCall {
        object: "this.cat".into(),
        method: "say".into(),
        arguments: vec!["\"Hello world!\"".into()],
    }],
}
```

### CollisionListener

Represents an Alice collision listener that fires when two objects collide. The
`object_a` and `object_b` fields name the two colliding objects. The `body`
contains the statements to execute on collision.

```rust
Statement::CollisionListener {
    object_a: "this.cat".into(),
    object_b: "this.dog".into(),
    body: vec![Statement::MethodCall {
        object: "this.cat".into(),
        method: "say".into(),
        arguments: vec!["\"Ouch!\"".into()],
    }],
}
```

### Serde round-trip guarantee

Both new variants survive JSON serialization and deserialization without loss,
just like the existing variants:

```rust
let json = serde_json::to_string(&program).unwrap();
let restored: Program = serde_json::from_str(&json).unwrap();
assert_eq!(program, restored);
```

The `#[serde(tag = "kind")]` attribute produces JSON with `"kind":
"EventListener"` or `"kind": "CollisionListener"`. Unknown variants are rejected
at deserialization time.

### Nesting and recursion

Event listeners and collision listeners have `body` fields that can contain any
`Statement` variant, including nested `CountLoop`, `IfElse`, `EventListener`, or
`CollisionListener` statements. The AST scanners recurse into all body fields to
find nested constructs.

For example, an event listener containing a counting loop:

```rust
Statement::EventListener {
    event: "SceneActivated".into(),
    body: vec![Statement::CountLoop {
        count: 5,
        body: vec![Statement::MethodCall {
            object: "this.bird".into(),
            method: "fly".into(),
            arguments: vec!["FORWARD".into(), "1.0".into()],
        }],
    }],
}
```

The loops scanner (`stmt_find_constructs`) recurses into `EventListener` and
`CollisionListener` bodies to find nested loops and conditionals. The events
scanner (`stmt_find_event_constructs`) recurses into `CountLoop`, `IfElse`,
`EventListener`, and `CollisionListener` bodies to find nested event constructs.

## Output schema

The `--json` flag produces structured JSON using the same `GradingReport` schema
as the first-lesson and loops-and-conditionals grading reports:

```json
{
  "schema_version": "eatme.assets/grading/v1",
  "lesson": "events-collision-proximity-game",
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
      "name": "add-event-listener",
      "status": "blocked",
      "depends_on": ["launch-smoke"],
      "reason": "Blocked by: launch-smoke"
    },
    {
      "name": "add-collision-listener",
      "status": "blocked",
      "depends_on": ["add-event-listener"],
      "reason": "Blocked by: add-event-listener"
    },
    {
      "name": "run-world",
      "status": "blocked",
      "depends_on": ["add-collision-listener"],
      "reason": "Blocked by: add-collision-listener"
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

The schema fields are identical to the loops-and-conditionals grading report:

| Field | Type | Description |
| --- | --- | --- |
| `schema_version` | string | Always `eatme.assets/grading/v1`. |
| `lesson` | string | Always `events-collision-proximity-game`. |
| `passed` | bool | `true` only when all steps are `ready`. |
| `steps` | array | Ordered list of `StepGrade` objects. |
| `steps[].name` | string | Step identifier. |
| `steps[].status` | string | One of `ready`, `blocked`, or `not-yet-tested`. |
| `steps[].depends_on` | array of strings | Step names this step depends on. Empty array `[]` for root steps. |
| `steps[].reason` | string | Human-readable explanation of the status. |

## Lesson steps

The grading report evaluates seven steps for the
`events-collision-proximity-game` scenario. The first three are **precondition
steps** identical to the first-lesson and loops-and-conditionals grading
reports. The last four are **lesson interaction steps** specific to the
events-and-collision curriculum.

### Precondition steps

| Step | What it checks | Passes when |
| --- | --- | --- |
| `validate-assets` | Committed persona and scenario assets | `validate_assets()` returns `passed=true` |
| `check-dependencies` | Host tools for real Alice smoke runs | `check_dependencies()` returns `all_required_available=true` |
| `launch-smoke` | Preconditions for launching Alice | Both `validate-assets` and `check-dependencies` are `ready` |

### Lesson interaction steps

| Step | What it checks | With `Program` | Without `Program` |
| --- | --- | --- | --- |
| `add-event-listener` | Student AST contains ≥1 `EventListener` node | `ready` if found, `blocked` if missing | `blocked` |
| `add-collision-listener` | Student AST contains ≥1 `CollisionListener` node | `ready` if found, `blocked` if missing | `blocked` |
| `run-world` | Student world executes successfully | `not-yet-tested` (requires runtime) | `blocked` |
| `save-project` | Saved AST round-trips without loss | `ready` if round-trip passes, `blocked` if not | `blocked` |

The "With `Program`" column assumes all upstream dependencies are satisfied.
When any upstream step is `blocked`, downstream steps cascade to `blocked`
regardless of the `Program`. The lesson interaction steps are hardcoded in the
grading function — they do not appear in the scenario YAML
(`events-collision-proximity-game.yaml`), which only defines the three
precondition steps. The interaction steps represent the alice.org curriculum's
Events and Collision proximity game activities.

When a student `Program` is provided to the grading function, the
`add-event-listener` step walks the AST recursively to find any `EventListener`
statement. If found, the step reports `ready`. If no `EventListener` exists, it
reports `blocked` with reason `"No EventListener found in student program"`.

The same logic applies to `add-collision-listener` with `CollisionListener`
nodes.

The `run-world` step is **not** AST-aware — it requires runtime execution. When
all upstream dependencies are satisfied it reports `not-yet-tested`; when any
upstream step is blocked it cascades to `blocked`.

The `save-project` step serializes the `Program` to JSON, deserializes it back,
and compares the result to the original using `PartialEq`. If equal, `ready`.
If not, `blocked` with reason `"AST did not survive save/reopen round-trip"`.
Unlike the loops-and-conditionals grading (which reports `ready` as a
placeholder), this step performs an actual round-trip verification to confirm
the new `EventListener` and `CollisionListener` variants serialize correctly.

## Step dependency graph

Steps form a linear dependency chain with two root nodes:

```text
validate-assets ─┐
                  ├─→ launch-smoke → add-event-listener → add-collision-listener
check-dependencies┘                                        │
                                                           ↓
                                                       run-world → save-project
```

All seven steps form a single linear chain after the initial fan-in at
`launch-smoke`. Each subsequent step depends on exactly one predecessor. If any
step reports `blocked`, all downstream steps also report `blocked`. The
`not-yet-tested` status does **not** cascade — downstream steps evaluate
independently.

## Status semantics

The same three statuses used by the first-lesson and loops-and-conditionals
grading reports apply here:

| Status | Meaning |
| --- | --- |
| `ready` | Preconditions met or AST check passed. |
| `blocked` | Preconditions failed or required AST construct missing. |
| `not-yet-tested` | Requires runtime execution. All upstream dependencies are satisfied. |

When a `Program` is provided, AST-aware steps (`add-event-listener`,
`add-collision-listener`, `save-project`) produce `ready` or `blocked` based on
AST inspection. When no `Program` is provided (`None`), all lesson interaction
steps produce `blocked` with reason `"No student program provided"`.

The top-level `passed` field is `true` only when every step is `ready`.
Because `run-world` always produces `not-yet-tested` (it requires runtime
execution that the grading function does not perform), `passed` is always `false`
when called from the grading function alone. This is intentional — the report
confirms structural readiness, not lesson completion.

## Module structure

The events-and-collision grading code lives in a dedicated module,
`grading_report_events`, extracted from `grading_report` to keep both files
under the 500-line quality gate.

### File layout

```text
crates/eatme-assets/src/
├── grading_report.rs             # Shared types (GradingReport, StepGrade,
│                                 # StepStatus, GradingInput, LoopsGradingInput),
│                                 # first-lesson + loops grading, shared helpers
├── grading_report_events.rs      # EventsGradingInput, grade_events_and_collision,
│                                 # event-specific AST helpers
├── grading_report_tests.rs       # First-lesson grading unit tests
├── grading_report_loops_tests.rs # Loops grading unit tests
├── grading_report_events_tests.rs# Events grading unit tests
└── lib.rs                        # pub(crate) mod grading_report_events;
                                  # re-exports EventsGradingInput and
                                  # grade_events_and_collision
```

### Shared helpers

Four helper functions in `grading_report.rs` are used by both the loops grading
and events grading modules. They are `pub(crate)` — visible within the crate
but not part of the public API:

| Helper | Purpose |
| --- | --- |
| `build_preconditions` | Produces the three precondition `StepGrade`s (validate-assets, check-dependencies, launch-smoke) |
| `cascade_blocked` | Creates a `StepGrade` with `Blocked` status and "Blocked by:" reason |
| `no_program_chain` | Creates a chain of `Blocked` steps when no student program is provided |
| `ast_check_step` | Creates a `StepGrade` based on whether an AST construct was found |

The `grading_report_events` module imports these helpers with a plain `use`
statement (they are called, not re-exported):

```rust
// In grading_report_events.rs — plain use for pub(crate) helpers
use crate::grading_report::{
    build_preconditions, cascade_blocked, no_program_chain, ast_check_step,
};
```

### Import paths

External callers use the crate-level re-exports and do not need to know which
internal module owns each symbol:

```rust
use eatme_assets::{
    grade_events_and_collision, EventsGradingInput, GradingReport, StepStatus,
};
```

For internal (in-crate) imports, the module path is explicit:

```rust
use crate::grading_report_events::{grade_events_and_collision, EventsGradingInput};
```

### Type re-exports for tests

Shared types (`GradingReport`, `StepGrade`, `StepStatus`) must be re-exported
with **`pub use`** (not plain `use`) in `grading_report_events.rs` so that the
test file's `use super::*` can access them:

```rust
// In grading_report_events.rs — MUST be pub use so super::* works in tests
pub use crate::grading_report::{GradingReport, StepGrade, StepStatus};
```

Using plain `use` instead of `pub use` here will cause the test file to fail
with "unresolved import" errors because `use super::*` only re-exports items
that are `pub` in the parent module.

### Test module ownership

The `#[cfg(test)]` declaration for `grading_report_events_tests.rs` lives in
`grading_report_events.rs` (not `grading_report.rs`), so `use super::*` in the
test file resolves to the events module.

## API reference

### `EventsGradingInput`

Input struct for the events-and-collision grading function. Defined in
`grading_report_events.rs`, re-exported from `eatme_assets`:

```rust
use eatme_core::ast::Program;

pub struct EventsGradingInput {
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

### `grade_events_and_collision`

Produces a `GradingReport` for the events-and-collision lesson. Defined in
`grading_report_events.rs`, re-exported from `eatme_assets`:

```rust
use eatme_assets::{
    grade_events_and_collision, EventsGradingInput, GradingReport,
};

let report: GradingReport = grade_events_and_collision(EventsGradingInput {
    assets_valid: true,
    asset_reason: "All 93 scenario assets passed validation".into(),
    deps_available: true,
    deps_reason: "All required tools available".into(),
    student_program: Some(program),
});
```

The function is pure — it takes an input struct and returns a report. It does
not perform I/O, spawn processes, or access the filesystem.

### AST helper: `ast_find_event_constructs`

Recursively walks a `Program` to determine if any `EventListener` or
`CollisionListener` statements exist. Private to `grading_report_events`:

```rust
fn ast_find_event_constructs(program: &Program) -> (bool, bool)
```

Returns `(has_event_listener, has_collision_listener)`.

### AST helper: `stmt_find_event_constructs`

Recursively walks a slice of statements to find event constructs. Private to
`grading_report_events`:

```rust
fn stmt_find_event_constructs(
    stmts: &[Statement],
    has_event: &mut bool,
    has_collision: &mut bool,
)
```

This function recurses into the body of every variant that has one —
`CountLoop.body`, `IfElse.if_body`, `IfElse.else_body`, `EventListener.body`,
and `CollisionListener.body` — to find nested event and collision listener
constructs.

### Exhaustive match update: `stmt_find_constructs`

The existing loops scanner (`stmt_find_constructs`) in `grading_report.rs` is
updated with two new match arms for the `EventListener` and
`CollisionListener` variants. These arms **recurse into the body** (to find
nested loops and conditionals) but do not set `has_loop` or `has_cond`. This
maintains exhaustive pattern matching after the new AST variants are added.

### Dependency propagation logic

The grading function propagates status through the dependency graph:

1. Root steps (`validate-assets`, `check-dependencies`) are graded from
   `EventsGradingInput` fields.
2. `launch-smoke` checks its `depends_on` list. If any dependency is `Blocked`,
   `launch-smoke` is `Blocked` with a reason listing the blockers.
3. AST-aware steps (`add-event-listener`, `add-collision-listener`,
   `save-project`) evaluate their AST checks when all upstream dependencies are
   satisfied. If any upstream dependency is `Blocked`, the step cascades to
   `Blocked`.
4. The `run-world` step is not AST-aware. It reports `NotYetTested` when all
   upstream dependencies are satisfied, and `Blocked` when any upstream
   dependency is `Blocked`.

### Crate boundary

The `eatme-assets` crate owns the grading types and pure grading functions. The
`eatme-core` crate owns the AST types. The `eatme-cli` crate orchestrates:

```text
eatme-cli (main.rs)
  ├── eatme_assets::validate_assets()    → AssetValidationReport
  ├── eatme_alice::check_dependencies()  → DependencyReport
  ├── eatme_core::ast::Program           → student program AST
  └── eatme_assets::grade_events_and_collision(EventsGradingInput { ... })
                                          → GradingReport (7 steps)
```

Within `eatme-assets`, the events grading code lives in
`grading_report_events.rs` and imports shared helpers from `grading_report.rs`
via `pub(crate)` visibility. The `lib.rs` re-exports ensure a flat public API —
callers import from `eatme_assets::` regardless of which internal module owns
the symbol.

```text
eatme-assets (lib.rs)
  ├── pub mod grading_report           # GradingReport, StepGrade, StepStatus,
  │                                    # GradingInput, LoopsGradingInput,
  │                                    # grade_first_lesson_readiness,
  │                                    # grade_loops_and_conditionals,
  │                                    # pub(crate) helpers
  └── pub(crate) mod grading_report_events  # EventsGradingInput,
                                            # grade_events_and_collision,
                                            # event-specific AST helpers
```

The `eatme-core` crate provides the `ast` module (`Program`, `Procedure`,
`Statement` with all five variants). The `eatme-assets` crate depends on
`eatme-core` for AST types. The `eatme-cli` crate depends on both. This
boundary ensures `eatme-assets` does not depend on `eatme-alice`.

## Configuration

The events-and-collision grading report does not require real Alice desktop
execution, Node, or environment variables when used as a Rust API.

| Setting | Required | Purpose |
| --- | --- | --- |
| `TMPDIR=/tmp` | Recommended | Avoids Unix socket path length errors in deep worktrees. |
| `NODE_OPTIONS` | No | Not needed; no Node processes are launched. |
| `EATME_REAL_ALICE` | No | Not needed by the grading function itself; required by the `launch-smoke` scenario step if exercised end-to-end. |

## Examples

### Build a minimal program with events and collision and grade it

```rust
use eatme_core::ast::{Program, Procedure, Statement};
use eatme_assets::{
    grade_events_and_collision, EventsGradingInput, StepStatus,
};

let program = Program {
    procedures: vec![Procedure {
        name: "myFirstMethod".into(),
        body: vec![
            Statement::EventListener {
                event: "SceneActivated".into(),
                body: vec![Statement::MethodCall {
                    object: "this.cat".into(),
                    method: "say".into(),
                    arguments: vec!["\"Hello world!\"".into()],
                }],
            },
            Statement::CollisionListener {
                object_a: "this.cat".into(),
                object_b: "this.dog".into(),
                body: vec![Statement::MethodCall {
                    object: "this.cat".into(),
                    method: "say".into(),
                    arguments: vec!["\"Ouch!\"".into()],
                }],
            },
        ],
    }],
};

let report = grade_events_and_collision(EventsGradingInput {
    assets_valid: true,
    asset_reason: "All 93 scenario assets passed validation".into(),
    deps_available: true,
    deps_reason: "All required tools available".into(),
    student_program: Some(program),
});

assert_eq!(report.lesson, "events-collision-proximity-game");
assert_eq!(report.steps.len(), 7);
// add-event-listener found the EventListener → ready
assert_eq!(report.steps[3].status, StepStatus::Ready);
// add-collision-listener found the CollisionListener → ready
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
let report = grade_events_and_collision(EventsGradingInput {
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

### Grade with missing event listener

```rust
let program = Program {
    procedures: vec![Procedure {
        name: "myFirstMethod".into(),
        body: vec![Statement::CollisionListener {
            object_a: "this.cat".into(),
            object_b: "this.dog".into(),
            body: vec![],
        }],
    }],
};

let report = grade_events_and_collision(EventsGradingInput {
    assets_valid: true,
    asset_reason: "All 93 scenario assets passed validation".into(),
    deps_available: true,
    deps_reason: "All required tools available".into(),
    student_program: Some(program),
});

// add-event-listener: no EventListener → blocked
assert_eq!(report.steps[3].status, StepStatus::Blocked);
assert!(report.steps[3].reason.contains("No EventListener found"));
// add-collision-listener: blocked cascades from add-event-listener
assert_eq!(report.steps[4].status, StepStatus::Blocked);
// run-world and save-project: blocked cascades
assert_eq!(report.steps[5].status, StepStatus::Blocked);
assert_eq!(report.steps[6].status, StepStatus::Blocked);
```

### Grade with missing collision listener

```rust
let program = Program {
    procedures: vec![Procedure {
        name: "myFirstMethod".into(),
        body: vec![Statement::EventListener {
            event: "SceneActivated".into(),
            body: vec![],
        }],
    }],
};

let report = grade_events_and_collision(EventsGradingInput {
    assets_valid: true,
    asset_reason: "All 93 scenario assets passed validation".into(),
    deps_available: true,
    deps_reason: "All required tools available".into(),
    student_program: Some(program),
});

// add-event-listener: EventListener found → ready
assert_eq!(report.steps[3].status, StepStatus::Ready);
// add-collision-listener: no CollisionListener → blocked
assert_eq!(report.steps[4].status, StepStatus::Blocked);
assert!(report.steps[4].reason.contains("No CollisionListener found"));
// run-world and save-project: blocked cascades
assert_eq!(report.steps[5].status, StepStatus::Blocked);
assert_eq!(report.steps[6].status, StepStatus::Blocked);
```

### Nested events inside loops

Event listeners and collision listeners can appear inside loop bodies. The
events scanner recurses into all nested bodies:

```rust
let program = Program {
    procedures: vec![Procedure {
        name: "myFirstMethod".into(),
        body: vec![Statement::CountLoop {
            count: 3,
            body: vec![
                Statement::EventListener {
                    event: "KeyPress".into(),
                    body: vec![],
                },
                Statement::CollisionListener {
                    object_a: "this.cat".into(),
                    object_b: "this.dog".into(),
                    body: vec![],
                },
            ],
        }],
    }],
};

let report = grade_events_and_collision(EventsGradingInput {
    assets_valid: true,
    asset_reason: "All 93 scenario assets passed validation".into(),
    deps_available: true,
    deps_reason: "All required tools available".into(),
    student_program: Some(program),
});

// Both found inside the CountLoop body
assert_eq!(report.steps[3].status, StepStatus::Ready);
assert_eq!(report.steps[4].status, StepStatus::Ready);
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

Run the events grading unit tests:

```bash
TMPDIR=/tmp cargo test -p eatme-assets grading_report_events -- --test-threads=1
```

Run the events-and-collision E2E test:

```bash
TMPDIR=/tmp cargo test -p eatme-alice --test events_and_collision_e2e -- --test-threads=1
```

Run the full quality gate:

```bash
TMPDIR=/tmp ./scripts/quality-gates.sh
```

### Plain text output (no --json)

```bash
cargo run -q -p eatme-cli -- assets grading-report \
  --lesson events-collision-proximity-game
```

```text
Events grading: events-collision-proximity-game
  validate-assets: ready — All 93 scenario assets passed validation
  check-dependencies: blocked — Missing required tools: Xvfb, wmctrl
  launch-smoke: blocked — Blocked by: check-dependencies
  add-event-listener: blocked — Blocked by: launch-smoke
  add-collision-listener: blocked — Blocked by: add-event-listener
  run-world: blocked — Blocked by: add-collision-listener
  save-project: blocked — Blocked by: run-world
Result: NOT READY
```

The result is `NOT READY` because lesson interaction steps are blocked by
missing host dependencies. When all precondition steps are `ready` and a
complete program is provided, the result is still `NOT READY` because
`run-world` reports `not-yet-tested`.

## E2E test

The end-to-end test at `crates/eatme-alice/tests/events_and_collision_e2e.rs`
validates the full pipeline: AST construction → grading report → JSON
serialization → save/reopen round-trip.

### Test inventory

| Test | What it validates |
| --- | --- |
| `events_grading_all_ready_with_complete_program` | Complete program with event listener and collision listener. Precondition steps are `ready`. `add-event-listener`, `add-collision-listener`, and `save-project` are `ready`. `run-world` is `not-yet-tested`. |
| `events_grading_blocked_without_program` | No student program (`None`). All 4 interaction steps report `blocked`. |
| `events_grading_missing_event_listener_blocks_downstream` | Program with `CollisionListener` but no `EventListener`. The `add-event-listener` step reports `blocked`, downstream steps cascade to `blocked`. |
| `events_grading_missing_collision_listener_blocks_downstream` | Program with `EventListener` but no `CollisionListener`. The `add-collision-listener` step reports `blocked`, downstream steps cascade to `blocked`. |
| `events_ast_survives_json_round_trip` | Serialize a `Program` with event and collision listeners to JSON and deserialize it. The restored AST equals the original. |
| `events_grading_report_schema_version_and_lesson` | Schema version is `eatme.assets/grading/v1` and lesson is `events-collision-proximity-game`. |
| `events_grading_report_has_seven_steps` | Report always contains exactly 7 steps in the expected order. |

### Running the E2E test

```bash
TMPDIR=/tmp cargo test -p eatme-alice --test events_and_collision_e2e -- --test-threads=1
```

The E2E test does not launch Alice or require a display server. It exercises the
Rust API in-process using constructed AST fixtures.

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
must include a `"kind"` field with one of `"MethodCall"`, `"CountLoop"`,
`"IfElse"`, `"EventListener"`, or `"CollisionListener"`. Missing or misspelled
`"kind"` values cause deserialization failure.

### Module too long (quality gate failure)

All Rust source modules must stay at or below 500 lines. The events grading
code is extracted into `grading_report_events.rs` to keep both files under the
limit. Expected file sizes after extraction:

| File | Expected lines | Limit |
| --- | --- | --- |
| `crates/eatme-core/src/ast.rs` | ~50 | 500 |
| `crates/eatme-core/src/ast_tests.rs` | ~320 | 500 |
| `crates/eatme-assets/src/grading_report.rs` | ~350 | 500 |
| `crates/eatme-assets/src/grading_report_events.rs` | ~180 | 500 |
| `crates/eatme-assets/src/grading_report_events_tests.rs` | ~490 | 500 |
| `crates/eatme-alice/tests/events_and_collision_e2e.rs` | ~240 | 500 |

If either `grading_report.rs` or `grading_report_events.rs` approaches the
500-line limit again, follow the same extraction pattern: identify the
lesson-specific code, create a new `grading_report_<lesson>.rs` module, widen
shared helpers to `pub(crate)`, and update `lib.rs` re-exports.

## Related documentation

- [Loops and Conditionals Grading Report](loops-and-conditionals-grading.md) —
  the loops-and-conditionals lesson grading report that this feature mirrors.
- [First-Lesson Grading Report](first-lesson-grading-report.md) — the original
  grading report for the Building a Scene first-lesson scenario.
- [Creative Assessment Boundary](creative-assessment-boundary.md) — the
  boundary between machine-assessable and human-review-needed aspects.
- [Save/reopen Readiness](save-reopen-readiness.md) — the evidence contract for
  save/reopen persistence.
- [Student Lesson E2E Tests](student-lesson-e2e-tests.md) — the existing
  student lesson E2E test patterns this feature follows.
- [Validation and Quality Gates](validation-quality-gates.md) — the 500-line
  module size gate and other quality checks.
- [Scenario Authoring](scenario-authoring.md) — how to author scenario YAML
  files including the `events-collision-proximity-game` scenario.
- [Student Missions](student-missions.md) — the `events-collision-proximity-game`
  scenario listed under student mission coverage.
