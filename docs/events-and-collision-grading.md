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
- [Real-Alice integration test](#real-alice-integration-test)
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
under the 500-line quality gate. For the cross-cutting module map, shared
helper contracts, import patterns, and how to add new lesson grading functions,
see [Grading Module Architecture](grading-module-architecture.md).

### File layout

```text
crates/eatme-assets/src/
├── grading_report.rs                        # Shared types (GradingReport, StepGrade,
│                                            # StepStatus, GradingInput, LoopsGradingInput),
│                                            # first-lesson + loops grading, shared helpers
├── grading_report_events.rs                 # EventsGradingInput, grade_events_and_collision,
│                                            # event-specific AST helpers
├── grading_report_tests.rs                  # First-lesson grading unit tests
├── grading_report_loops_tests.rs            # Loops grading unit tests
├── grading_report_events_tests.rs           # Events grading unit tests
├── grading_report_extraction_tests.rs       # Extraction contract tests (25 tests):
│                                            # quality-gate, helper accessibility,
│                                            # structure, schema, dependency-chain,
│                                            # complete-program behavior
├── grading_report_extraction_edge_tests.rs  # Extraction edge-case tests (15 tests):
│                                            # boundary inputs, cascade failures,
│                                            # nested AST, JSON serialization
└── lib.rs                                   # pub(crate) mod grading_report_events;
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

### Extraction contract test split

The extraction contract tests verify that the `grading_report_events` extraction
preserves all behavioral contracts from the original `grading_report` module.
These tests are split across two files to stay under the 500-line quality gate:

| File | Lines | Tests | Responsibility |
| --- | --- | --- | --- |
| `grading_report_extraction_tests.rs` | ~277 | 25 | Core contracts: quality-gate line counts, `pub(crate)` helper accessibility, module structure, schema version, step names/order, dependency chain, and complete-program behavior. |
| `grading_report_extraction_edge_tests.rs` | ~404 | 15 | Edge cases: no-program boundary, missing individual listeners, blocked assets/deps cascade, both-blocked cascade, nested AST constructs (event-inside-collision, collision-inside-event, empty program, multi-procedure, loops, if/else), and JSON serialization round-trip. |

Both files import directly from `crate::grading_report` and
`crate::grading_report_events` and duplicate their own fixture functions. This
avoids coupling between the two test modules and keeps each independently
compilable. The `lib.rs` registers both with `#[cfg(test)] mod` declarations.

Run all extraction contract tests:

```bash
TMPDIR=/tmp cargo test -p eatme-assets -- grading_report_extraction
```

Run only the edge-case tests:

```bash
TMPDIR=/tmp cargo test -p eatme-assets -- grading_report_extraction_edge
```

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
| `EATME_REAL_ALICE` | No | Not needed by the grading function itself; required by the real-Alice integration test and the `launch-smoke` scenario step if exercised end-to-end. See [Real-Alice integration test](#real-alice-integration-test). |

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

Run the extraction contract tests (core + edge):

```bash
TMPDIR=/tmp cargo test -p eatme-assets grading_report_extraction -- --test-threads=1
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
| `ast_with_events_survives_json_round_trip` | Serialize a `Program` with event and collision listeners to JSON and deserialize it. The restored AST equals the original. |
| `events_grading_report_schema_version_and_lesson` | Schema version is `eatme.assets/grading/v1` and lesson is `events-collision-proximity-game`. |
| `events_grading_report_has_seven_steps` | Report always contains exactly 7 steps in the expected order. |
| `real_alice_events_collision_launch_smoke` | **Real-Alice gated (Phase 1).** Launches real Alice via `run_launch_smoke` with `events-collision-proximity-game` scenario, validates 6 manifest assertions, screenshot PNG, manifest.json round-trip, and alice.log. |
| `real_alice_events_grading_baseline_no_program` | **Real-Alice gated (Phase 2).** Calls `grade_events_and_collision` with `None` program, asserts steps 3–6 are `Blocked`. |
| `real_alice_events_grading_complete_program` | **Real-Alice gated (Phase 3).** Constructs synthetic AST with `EventListener` + `CollisionListener`, grades, asserts all steps `Ready` (except `run-world` = `NotYetTested`), JSON round-trip. |

### Running the E2E test

```bash
TMPDIR=/tmp cargo test -p eatme-alice --test events_and_collision_e2e -- --test-threads=1
```

The synthetic E2E tests do not launch Alice or require a display server. They
exercise the Rust API in-process using constructed AST fixtures. The real-Alice
integration tests (`real_alice_events_collision_launch_smoke`,
`real_alice_events_grading_baseline_no_program`,
`real_alice_events_grading_complete_program`) are gated behind
`EATME_REAL_ALICE=1` and skip automatically when the environment variable is
absent.

## Real-Alice integration test

Three gated test functions validate the complete events-and-collision pipeline
against a real Alice installation:

| Test | Phase | What it validates |
| --- | --- | --- |
| `real_alice_events_collision_launch_smoke` | 1 | Real Alice launch with `events-collision-proximity-game` scenario. |
| `real_alice_events_grading_baseline_no_program` | 2 | Baseline grading produces `Blocked` for all interaction steps when no program is provided. |
| `real_alice_events_grading_complete_program` | 3 | Complete grading with synthetic `EventListener` + `CollisionListener` AST. |

All three are gated behind `EATME_REAL_ALICE=1` and skip automatically when the
environment variable is absent.

### Usage

Run the real-Alice events-and-collision integration tests:

```bash
EATME_REAL_ALICE=1 cargo test -p eatme-alice --test events_and_collision_e2e \
  real_alice -- --nocapture
```

Run all events-and-collision tests (the real-Alice test skips automatically when
`EATME_REAL_ALICE` is unset):

```bash
cargo test -p eatme-alice --test events_and_collision_e2e
```

### Environment gate

| Variable | Required value | Effect |
| --- | --- | --- |
| `EATME_REAL_ALICE` | `1` | Enables the real-Alice integration test. Any other value or absence causes the test to skip with an `eprintln` message and early return. |
| `ALICE_HOME` | Path to Alice checkout | The Alice checkout directory. Defaults to `/opt/alice3` when not set (matches `launch_smoke_real.rs`). |

The gate is a runtime `std::env::var` check, not a compile-time `cfg`
attribute. This matches the pattern established by the
[Deterministic Real-Alice Smoke Test](deterministic-real-alice-smoke-test.md).
The test always compiles, always appears in the test binary, and returns early
when the gate is not satisfied.

### Three-phase test design

The real-Alice integration tests are split into three independent `#[test]`
functions that share `real_alice_enabled()` and `alice_home()` helpers (same
pattern as `launch_smoke_real.rs`):

#### Phase 1: `real_alice_events_collision_launch_smoke`

Launches real Alice through `run_launch_smoke` with the
`events-collision-proximity-game` scenario:

```rust
use eatme_alice::{LaunchSmokeOptions, LaunchSmokeScenario, run_launch_smoke};

let scenario = LaunchSmokeScenario::new("events-collision-proximity-game");
let manifest = run_launch_smoke(&LaunchSmokeOptions {
    alice_home: alice_home(),
    scenario,
    run_id: "real-alice-events-collision".into(),
    runs_dir: PathBuf::from("target/test-work/events-collision-real/runs"),
    timeout_seconds: 90,
    json: true,
    no_memory: true,
    offline_package: true,
}).expect("run_launch_smoke should succeed");
```

Phase 1 validates:

| Assertion | What it proves |
| --- | --- |
| 6 manifest assertions pass | `dependencies_available`, `display_responsive`, `process_started`, `startup_screenshot`, `no_fatal_logs`, `real_alice_execution_evidence` all passed. |
| `manifest.failure_category.is_none()` | No fatal failure during launch. |
| Screenshot exists and has PNG magic bytes | Visual evidence was captured. |
| Manifest file exists on disk | Evidence was persisted as `manifest.json`. |
| Manifest JSON round-trips | Serialize → deserialize produces identical manifest. |
| `alice.log` artifact is non-empty | Log evidence was captured. |

#### Phase 2: `real_alice_events_grading_baseline_no_program`

Verifies that when no student program is provided (`student_program: None`), all
interaction steps (3–6) are correctly `Blocked`. This confirms the grading
pipeline does not produce false positives when no AST is available:

```rust
let baseline = grade_events_and_collision(EventsGradingInput {
    assets_valid: true,
    asset_reason: "Assets valid".into(),
    deps_available: true,
    deps_reason: "Deps available".into(),
    student_program: None,
});

// No program provided — all interaction steps blocked
assert_eq!(baseline.steps[3].status, StepStatus::Blocked);
assert_eq!(baseline.steps[4].status, StepStatus::Blocked);
```

#### Phase 3: `real_alice_events_grading_complete_program`

Constructs a synthetic program with both `EventListener` and
`CollisionListener` and verifies the grading pipeline recognizes them:

```rust
let augmented_program = Program {
    procedures: vec![Procedure {
        name: "myFirstMethod".into(),
        body: vec![
            Statement::EventListener {
                event: "SceneActivated".into(),
                body: vec![/* ... */],
            },
            Statement::CollisionListener {
                object_a: "this.cat".into(),
                object_b: "this.dog".into(),
                body: vec![/* ... */],
            },
        ],
    }],
};

let augmented = grade_events_and_collision(EventsGradingInput {
    student_program: Some(augmented_program),
    // ... same precondition fields ...
});

assert_eq!(augmented.steps[3].status, StepStatus::Ready);
assert_eq!(augmented.steps[4].status, StepStatus::Ready);
```

Phase 3 also validates evidence persistence:

| Check | What it proves |
| --- | --- |
| All 7 step names in order | Step dependency graph is intact. |
| `validate-assets` and `check-dependencies` are `Ready` | Preconditions pass. |
| `launch-smoke` is `Ready` | Both preconditions satisfied. |
| `add-event-listener` is `Ready` | `EventListener` found in student AST. |
| `add-collision-listener` is `Ready` | `CollisionListener` found in student AST. |
| `run-world` is `NotYetTested` | Runtime execution not performed (expected). |
| `save-project` is `Ready` | AST round-trip passed. |
| JSON round-trip | `serde_json::to_string` → `serde_json::from_str` produces identical report. |
| Manifest persisted | `manifest.json` written to `target/test-work/events-collision-real/runs/`. |

### What the test proves end-to-end

When run against a real Alice installation, the integration test proves:

1. **Real launch works for Lesson 4.** The `events-collision-proximity-game`
   scenario launches through the same harness used by all other lesson smokes.
2. **Baseline blocks without a program.** When no student program is provided,
   all interaction steps correctly report `Blocked`, confirming no false
   positives.
3. **Grading recognizes augmented ASTs.** After adding event and collision
   listener constructs, all AST-aware grading steps report `Ready`.
4. **Evidence persists.** Both the launch manifest and the grading report survive
   JSON serialization and can be written to disk for CI artifact collection.
5. **No false positives.** The `run-world` step correctly reports `NotYetTested`
   because runtime execution is not performed by the grading function.

### Configuration

| Setting | Value | Rationale |
| --- | --- | --- |
| `alice_home` | `ALICE_HOME` env var or `/opt/alice3` | Standard Alice checkout location (matches `launch_smoke_real.rs`). |
| `scenario` | `events-collision-proximity-game` | Lesson 4 scenario from the committed roster. |
| `run_id` | `real-alice-events-collision` | Kebab-case identifier for the evidence directory. |
| `runs_dir` | `target/test-work/events-collision-real/runs` | Isolated under `target/` to avoid polluting project root. |
| `timeout_seconds` | `90` | Covers cold Maven builds and slow Java startup. |
| `json` | `true` | Machine-readable output. |
| `no_memory` | `true` | No persistent memory side effects from test runs. |
| `offline_package` | `true` | Uses cached Maven dependencies, no network access. |

### Host requirements

The real-Alice integration test requires the same Linux host dependencies as the
[Deterministic Real-Alice Smoke Test](deterministic-real-alice-smoke-test.md#host-requirements):

| Dependency | Minimum | Purpose |
| --- | --- | --- |
| Java | 21 | Alice runtime |
| Maven | 3.9+ | Alice packaging |
| Xvfb | Any | Virtual X display |
| xdpyinfo | Any | Display readiness probe |
| wmctrl | Any | Window list capture |
| xwininfo | Any | Fallback window tree capture |
| xdotool | Any | Window activation |
| scrot or ImageMagick `import` | Any | Screenshot capture |
| Mesa/llvmpipe | Any | Software OpenGL rendering |

### Examples

#### Run the real-Alice integration test on a self-hosted runner

```bash
export ALICE_HOME=/opt/alice3
EATME_REAL_ALICE=1 cargo test -p eatme-alice --test events_and_collision_e2e \
  real_alice -- --nocapture
```

#### Inspect evidence after a real run

```bash
cat target/test-work/events-collision-real/runs/*/manifest.json \
  | jq '.assertions | to_entries[] | {key, passed: .value.passed}'
```

#### Run all tests including the real-Alice test

```bash
EATME_REAL_ALICE=1 cargo test -p eatme-alice --test events_and_collision_e2e -- --nocapture
```

Output includes both synthetic and real-Alice tests:

```text
test events_grading_all_ready_with_complete_program ... ok
test events_grading_blocked_without_program ... ok
test events_grading_missing_event_listener_blocks_downstream ... ok
test events_grading_missing_collision_listener_blocks_downstream ... ok
test ast_with_events_survives_json_round_trip ... ok
test events_grading_report_schema_version_and_lesson ... ok
test events_grading_report_has_seven_steps ... ok
test real_alice_events_collision_launch_smoke ... ok
test real_alice_events_grading_baseline_no_program ... ok
test real_alice_events_grading_complete_program ... ok
```

#### Run without the real-Alice gate (test auto-skips)

```bash
cargo test -p eatme-alice --test events_and_collision_e2e
```

The real-Alice tests print skip messages to stderr and pass without
exercising Alice:

```text
skipping real-Alice events-collision launch smoke (set EATME_REAL_ALICE=1 to enable)
skipping real-Alice events-collision baseline grading (set EATME_REAL_ALICE=1 to enable)
skipping real-Alice events-collision complete grading (set EATME_REAL_ALICE=1 to enable)
```

## Troubleshooting

### Real-Alice integration test skips unexpectedly

Verify the environment variable is set to exactly `1`:

```bash
echo $EATME_REAL_ALICE   # should print: 1
```

The check is `std::env::var("EATME_REAL_ALICE") == Ok("1".into())`. Values
like `true`, `yes`, or empty string do not activate the test.

### Real-Alice integration test times out

The default timeout is 90 seconds. In slow CI environments with cold Maven
caches, increase the timeout by modifying the `timeout_seconds` field in the
test. If Maven needs to download dependencies on first run, use a longer
timeout or pre-warm the cache:

```bash
cd ${ALICE_HOME} && mvn dependency:go-offline
```

### Real-Alice test fails on Phase 1 (launch)

Check that all desktop dependencies are installed. Run the dependency check:

```bash
cargo run -q -p eatme-cli -- deps check --json
```

Common missing dependencies: Xvfb, xdpyinfo, wmctrl, scrot. Install with:

```bash
sudo apt-get install -y xvfb x11-utils wmctrl xdotool scrot
```

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
| `crates/eatme-assets/src/grading_report_extraction_tests.rs` | ~277 | 500 |
| `crates/eatme-assets/src/grading_report_extraction_edge_tests.rs` | ~404 | 500 |
| `crates/eatme-alice/tests/events_and_collision_e2e.rs` | ~440 | 500 |

If either `grading_report.rs` or `grading_report_events.rs` approaches the
500-line limit again, follow the same extraction pattern: identify the
lesson-specific code, create a new `grading_report_<lesson>.rs` module, widen
shared helpers to `pub(crate)`, and update `lib.rs` re-exports.

## Related documentation

- [Grading Module Architecture](grading-module-architecture.md) — Module
  layout, shared helpers, import patterns, and how to add new lesson grading.
- [Deterministic Real-Alice Smoke Test](deterministic-real-alice-smoke-test.md) —
  the baseline real-Alice launch integration test that this test builds upon.
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
- [Alice Lesson Smoke](alice-lesson-smoke.md) — desktop scenario roster and
  evidence contracts including `events-collision-proximity-game`.
- [Validation and Quality Gates](validation-quality-gates.md) — the 500-line
  module size gate and other quality checks.
- [Scenario Authoring](scenario-authoring.md) — how to author scenario YAML
  files including the `events-collision-proximity-game` scenario.
- [Student Missions](student-missions.md) — the `events-collision-proximity-game`
  scenario listed under student mission coverage.
