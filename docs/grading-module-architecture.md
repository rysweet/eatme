# Grading module architecture

The `eatme-assets` crate contains three lesson-grading functions that share a
common pipeline of precondition checks, step-dependency propagation, and AST
inspection. The grading code is split across two Rust source modules to stay
within the repository's 500-line quality gate.

This document describes the module layout, public API surface, shared helper
contracts, import patterns, and how to add a new lesson grading function.

## Contents

- [Module map](#module-map)
- [Public API surface](#public-api-surface)
- [Shared helpers](#shared-helpers)
- [Import patterns](#import-patterns)
- [Quality gate compliance](#quality-gate-compliance)
- [Adding a new lesson grading function](#adding-a-new-lesson-grading-function)
- [Test layout](#test-layout)
- [Related documentation](#related-documentation)

## Module map

```text
crates/eatme-assets/src/
├── lib.rs
│     ├── pub mod grading_report
│     ├── pub(crate) mod grading_report_events
│     ├── pub use grading_report::{GradingReport, StepGrade, StepStatus,
│     │       GradingInput, LoopsGradingInput,
│     │       grade_first_lesson_readiness, grade_loops_and_conditionals}
│     └── pub use grading_report_events::{EventsGradingInput,
│             grade_events_and_collision}
│
├── grading_report.rs  (357 lines)
│     ├── Types:    GradingReport, StepGrade, StepStatus, GradingInput,
│     │             LoopsGradingInput
│     ├── Public:   grade_first_lesson_readiness, grade_loops_and_conditionals
│     ├── Helpers:  build_preconditions, cascade_blocked, no_program_chain,
│     │             ast_check_step  [pub(crate)]
│     └── Private:  interaction_step, evaluate_loops_steps,
│                   ast_find_constructs, stmt_find_constructs
│
├── grading_report_events.rs  (189 lines)
│     ├── Types:    EventsGradingInput
│     ├── Public:   grade_events_and_collision
│     ├── Re-exports: GradingReport, StepGrade, StepStatus  [pub use]
│     └── Private:  evaluate_events_steps, ast_find_event_constructs,
│                   stmt_find_event_constructs
│
├── grading_report_tests.rs              # first-lesson unit tests
├── grading_report_loops_tests.rs        # loops grading unit tests
├── grading_report_events_tests.rs       # events grading unit tests
├── grading_report_extraction_tests.rs   # extraction contract tests (25)
└── grading_report_extraction_edge_tests.rs  # extraction edge cases (15)
```

### Why two modules?

Before the split, `grading_report.rs` contained all three lesson grading
functions plus shared helpers and exceeded 500 lines. The event-grading code
(types, grading function, AST helpers) was extracted into
`grading_report_events.rs` because it is the most self-contained subset: it
depends on shared helpers but no other lesson grading function depends on it.

The first-lesson and loops-and-conditionals grading functions remain in
`grading_report.rs` because they share private helpers (`interaction_step`,
`evaluate_loops_steps`, `ast_find_constructs`) that would create circular
dependencies if split further.

## Public API surface

All public types and functions are re-exported from `eatme_assets` via `lib.rs`.
Callers use a flat import path regardless of which internal module owns the
symbol:

```rust
use eatme_assets::{
    // Shared types
    GradingReport, StepGrade, StepStatus,
    // First-lesson grading
    GradingInput, grade_first_lesson_readiness,
    // Loops grading
    LoopsGradingInput, grade_loops_and_conditionals,
    // Events grading
    EventsGradingInput, grade_events_and_collision,
};
```

### Grading functions

| Function | Input | Lesson ID | Steps |
| --- | --- | --- | --- |
| `grade_first_lesson_readiness` | `GradingInput` | `building-a-scene-first-world` | 6 (3 precondition + 3 interaction) |
| `grade_loops_and_conditionals` | `LoopsGradingInput` | `loops-and-conditionals-mini-challenge` | 7 (3 precondition + 4 interaction) |
| `grade_events_and_collision` | `EventsGradingInput` | `events-collision-proximity-game` | 7 (3 precondition + 4 interaction) |

All three functions are pure — they accept an input struct and return a
`GradingReport`. No I/O, no side effects.

### Input structs

The first-lesson input does not accept a student program because its
interaction steps are runtime-only:

```rust
pub struct GradingInput {
    pub assets_valid: bool,
    pub asset_reason: String,
    pub deps_available: bool,
    pub deps_reason: String,
}
```

The loops and events inputs add an optional student `Program` for AST
inspection:

```rust
pub struct LoopsGradingInput {
    pub assets_valid: bool,
    pub asset_reason: String,
    pub deps_available: bool,
    pub deps_reason: String,
    pub student_program: Option<Program>,
}

pub struct EventsGradingInput {
    pub assets_valid: bool,
    pub asset_reason: String,
    pub deps_available: bool,
    pub deps_reason: String,
    pub student_program: Option<Program>,
}
```

### Output

All three functions return the same `GradingReport`:

```rust
#[derive(Clone, Debug, Serialize)]
pub struct GradingReport {
    pub schema_version: String,  // "eatme.assets/grading/v1"
    pub lesson: String,
    pub passed: bool,
    pub steps: Vec<StepGrade>,
}

#[derive(Clone, Debug, Serialize)]
pub struct StepGrade {
    pub name: String,
    pub status: StepStatus,
    pub reason: String,
    pub depends_on: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum StepStatus {
    Ready,       // "ready"
    Blocked,     // "blocked"
    NotYetTested, // "not-yet-tested"
}
```

## Shared helpers

Four `pub(crate)` helpers in `grading_report.rs` are shared across modules:

| Helper | Signature | Purpose |
| --- | --- | --- |
| `build_preconditions` | `(bool, String, bool, String) → (Vec<StepGrade>, bool)` | Produces the three precondition steps and returns whether any are blocked |
| `cascade_blocked` | `(&str, &[&str]) → StepGrade` | Creates a `Blocked` step with "Blocked by: …" reason |
| `no_program_chain` | `(&[(&str, &str)]) → Vec<StepGrade>` | Creates a chain of `Blocked` steps when no student program is provided |
| `ast_check_step` | `(&str, &str, bool, &str) → StepGrade` | Creates `Ready` or `Blocked` based on whether an AST construct was found |

These helpers are **not** part of the public API. They are visible to
`grading_report_events.rs` via `pub(crate)` but hidden from external callers.

### Dependency propagation contract

All three grading functions follow the same propagation pattern:

1. **Root steps** (`validate-assets`, `check-dependencies`) are graded directly
   from input fields.
2. **`launch-smoke`** checks both root steps. If either is `Blocked`,
   `launch-smoke` is `Blocked` with a reason listing the blockers.
3. **Lesson interaction steps** depend on `launch-smoke` and on each other in a
   linear chain. If any upstream step is `Blocked`, downstream steps cascade to
   `Blocked` via `cascade_blocked`.
4. **`not-yet-tested` does not cascade.** Steps evaluate independently when
   upstream steps are `Ready` or `NotYetTested`.

## Import patterns

### External callers (other crates)

```rust
use eatme_assets::{
    grade_events_and_collision, EventsGradingInput, GradingReport, StepStatus,
};
```

### Internal callers (within eatme-assets)

The events module imports shared types and helpers explicitly:

```rust
// grading_report_events.rs

// Re-export shared types so test file's `use super::*` works
pub use crate::grading_report::{GradingReport, StepGrade, StepStatus};

// Import helpers (called, not re-exported)
use crate::grading_report::{
    ast_check_step, build_preconditions, cascade_blocked, no_program_chain,
};
```

The `pub use` for shared types is required — without it, the test file
(`grading_report_events_tests.rs`) cannot access `GradingReport` via
`use super::*`. Using plain `use` instead of `pub use` causes "unresolved
import" compilation errors.

### Test files

Each test file uses `use super::*` to import from its parent module:

```rust
// grading_report_tests.rs (parent: grading_report)
use super::*;  // gets GradingReport, StepGrade, StepStatus, GradingInput, etc.

// grading_report_events_tests.rs (parent: grading_report_events)
use super::*;  // gets EventsGradingInput, grade_events_and_collision,
               // plus re-exported GradingReport, StepGrade, StepStatus
```

## Quality gate compliance

The repository enforces a 500-line limit on Rust source modules under `crates/`.

| File | Lines | Budget |
| --- | --- | --- |
| `grading_report.rs` | 357 | ≤ 500 ✓ |
| `grading_report_events.rs` | 189 | ≤ 500 ✓ |
| `grading_report_tests.rs` | 341 | ≤ 500 ✓ |
| `grading_report_loops_tests.rs` | 497 | ≤ 500 ✓ |
| `grading_report_events_tests.rs` | 491 | ≤ 500 ✓ |
| `grading_report_extraction_tests.rs` | 275 | ≤ 500 ✓ |
| `grading_report_extraction_edge_tests.rs` | 404 | ≤ 500 ✓ |
| `grading_report_integration_tests.rs` | 141 | ≤ 500 ✓ |

Run the quality gate check:

```bash
TMPDIR=/tmp ./scripts/quality-gates.sh
```

Or check just the grading module line counts:

```bash
wc -l crates/eatme-assets/src/grading_report.rs \
      crates/eatme-assets/src/grading_report_events.rs
```

## Adding a new lesson grading function

To add grading for a fourth lesson (e.g., `animation-sequences`):

### 1. Choose a module

If the new grading function fits in `grading_report.rs` without exceeding 500
lines, add it there. Otherwise, create a new module
(`grading_report_animation.rs`).

### 2. Define the input struct

Follow the existing pattern. If the lesson needs AST inspection, include
`student_program: Option<Program>`:

```rust
pub struct AnimationGradingInput {
    pub assets_valid: bool,
    pub asset_reason: String,
    pub deps_available: bool,
    pub deps_reason: String,
    pub student_program: Option<Program>,
}
```

### 3. Implement the grading function

Use the shared helpers from `grading_report`:

```rust
use crate::grading_report::{
    build_preconditions, cascade_blocked, no_program_chain, ast_check_step,
};

pub fn grade_animation_sequences(input: AnimationGradingInput) -> GradingReport {
    let (mut steps, preconditions_blocked) = build_preconditions(
        input.assets_valid,
        input.asset_reason,
        input.deps_available,
        input.deps_reason,
    );

    let interaction_steps = if preconditions_blocked {
        vec![
            cascade_blocked("add-animation", &["launch-smoke"]),
            // ... more steps
        ]
    } else {
        evaluate_animation_steps(&input.student_program)
    };

    let passed = !preconditions_blocked
        && interaction_steps.iter().all(|s| s.status == StepStatus::Ready);

    steps.extend(interaction_steps);

    GradingReport {
        schema_version: "eatme.assets/grading/v1".into(),
        lesson: "animation-sequences".into(),
        passed,
        steps,
    }
}
```

### 4. Wire up lib.rs

If you created a new module file:

```rust
// In lib.rs
pub(crate) mod grading_report_animation;

pub use grading_report_animation::{AnimationGradingInput, grade_animation_sequences};
```

If you added to `grading_report.rs`:

```rust
// In lib.rs — add to the existing pub use
pub use grading_report::{
    GradingInput, GradingReport, LoopsGradingInput, StepGrade, StepStatus,
    AnimationGradingInput,
    grade_first_lesson_readiness, grade_loops_and_conditionals,
    grade_animation_sequences,
};
```

### 5. Add tests

Create a test file (`grading_report_animation_tests.rs`) and register it in the
module that owns the grading function:

```rust
#[cfg(test)]
#[path = "grading_report_animation_tests.rs"]
mod animation_tests;
```

### 6. Verify

```bash
TMPDIR=/tmp cargo test -p eatme-assets -- grading_report
wc -l crates/eatme-assets/src/grading_report*.rs
TMPDIR=/tmp ./scripts/quality-gates.sh
```

## Test layout

| Test file | Registered in | Tests | What it covers |
| --- | --- | --- | --- |
| `grading_report_tests.rs` | `grading_report.rs` | First-lesson unit tests | All 6 steps, cascade logic, edge cases |
| `grading_report_loops_tests.rs` | `grading_report.rs` | Loops grading unit tests | All 7 steps, AST inspection, cascade logic |
| `grading_report_events_tests.rs` | `grading_report_events.rs` | Events grading unit tests | All 7 steps, AST inspection, round-trip |
| `grading_report_extraction_tests.rs` | `lib.rs` | 25 extraction contract tests | Quality-gate line counts, helper accessibility, module structure, schema, dependency chain, complete-program behavior |
| `grading_report_extraction_edge_tests.rs` | `lib.rs` | 15 extraction edge cases | Boundary inputs, cascade failures, nested AST, JSON serialization |
| `grading_report_integration_tests.rs` | `lib.rs` | Integration tests | Cross-module behavior |

Run all grading tests:

```bash
TMPDIR=/tmp cargo test -p eatme-assets -- grading_report
```

Run only events grading tests:

```bash
TMPDIR=/tmp cargo test -p eatme-assets -- grading_report_events
```

Run only extraction contract tests:

```bash
TMPDIR=/tmp cargo test -p eatme-assets -- grading_report_extraction
```

## Related documentation

- [First-Lesson Grading Report](first-lesson-grading-report.md) — Usage,
  output schema, and examples for the Building a Scene lesson.
- [Loops and Conditionals Grading](loops-and-conditionals-grading.md) — Usage,
  output schema, and examples for the Loops and Conditionals lesson.
- [Events and Collision Grading](events-and-collision-grading.md) — Usage,
  output schema, module structure details, and examples for the Events and
  Collision lesson.
- [Validation and Quality Gates](validation-quality-gates.md) — Repository
  quality gates including the 500-line module limit.
- [Creative Assessment Boundary](creative-assessment-boundary.md) — What can
  be machine-assessed vs. what needs human review.
