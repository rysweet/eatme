# Real-Alice lesson grading integration tests

The real-Alice lesson grading integration tests validate the full pipeline for
Lessons 5–8 against a real Alice desktop session: launch Alice under Xvfb, load
a starter project, parse the `.a3p` file to extract the Tweedle AST, augment it
with student-authored constructs, and run the lesson grading pipeline. Each test
is gated behind the `EATME_REAL_ALICE=1` environment variable so CI and developer
machines without Alice desktop dependencies skip the test automatically.

The tests cover four lessons:

| Lesson | Scenario ID | Test file | Grading function |
| --- | --- | --- | --- |
| L5 Functions | `functions-as-questions-about-the-world` | `functions_e2e.rs` | `grade_functions` (lesson: `using-functions-mini-challenge`) |
| L6 Variables | `variables-scorekeeper-timekeeper` | `variables_e2e.rs` | `grade_variables` (lesson: `using-variables-mini-challenge`) |
| L7 Parameters | `reusable-methods-and-parameters` | `parameters_e2e.rs` | `grade_parameters` (lesson: `parameters-mini-challenge`) |
| L8 Creative project | `design-process-story-or-game` | `creative_project_e2e.rs` | `grade_creative_project` (lesson: `creative-design-project`) |

Each test file contains both synthetic fixture tests (always-run, no real Alice
required) and a single real-Alice integration test that exercises the complete
launch → parse → augment → grade pipeline.

## Contents

- [Usage](#usage)
- [Environment gate](#environment-gate)
- [What the tests prove](#what-the-tests-prove)
- [Test pipeline](#test-pipeline)
- [Per-lesson test details](#per-lesson-test-details)
- [AST augmentation per lesson](#ast-augmentation-per-lesson)
- [Grading step expectations](#grading-step-expectations)
- [Shared test support modules](#shared-test-support-modules)
- [Configuration](#configuration)
- [Examples](#examples)
- [Troubleshooting](#troubleshooting)
- [Authoring workflow](#authoring-workflow)
- [Related documentation](#related-documentation)

## Usage

Run all four real-Alice lesson grading tests:

```bash
TMPDIR=/tmp EATME_REAL_ALICE=1 cargo test -p eatme-alice \
  --test functions_e2e \
  --test variables_e2e \
  --test parameters_e2e \
  --test creative_project_e2e \
  -- real_alice --test-threads=1
```

Run a single lesson's real-Alice test:

```bash
TMPDIR=/tmp EATME_REAL_ALICE=1 cargo test -p eatme-alice \
  --test functions_e2e real_alice_functions_grading_integration
```

Run only the synthetic fixture tests (no real Alice needed):

```bash
TMPDIR=/tmp cargo test -p eatme-alice \
  --test functions_e2e \
  --test variables_e2e \
  --test parameters_e2e \
  --test creative_project_e2e \
  -- --test-threads=1
```

Run the full `eatme-alice` crate test suite (real-Alice tests auto-skip when the
environment variable is absent):

```bash
TMPDIR=/tmp cargo test -p eatme-alice -- --test-threads=1
```

Use `TMPDIR=/tmp` in deep worktrees to avoid Unix socket path length failures.

## Environment gate

| Variable | Required value | Effect |
| --- | --- | --- |
| `EATME_REAL_ALICE` | `1` | Enables real-Alice integration tests. Any other value or absence causes the test to return early with a skip message and pass. |
| `ALICE_HOME` | Path to Alice checkout | The Alice checkout directory. Defaults to `/opt/alice3` when not set. |

The gate is a runtime `std::env::var` check, not a compile-time `cfg`
attribute. This means:

- `cargo test -p eatme-alice` always compiles the test.
- The test binary always includes all four real-Alice tests.
- Each test body returns early when the gate is not satisfied.
- CI workflows that set `EATME_REAL_ALICE=1` on self-hosted runners with Alice
  desktop dependencies get the full integration validation.

## What the tests prove

Each real-Alice integration test exercises the complete pipeline with a real
Alice installation:

1. **Alice launch** — a real Alice process starts under Xvfb using the lesson's
   scenario and starter project, managed by `run_launch_smoke`.
2. **Starter project parsing** — the `.a3p` file from the starter project is
   parsed by the `a3p_parser_support` module to extract the Tweedle AST
   baseline.
3. **Baseline AST verification** — the parsed AST is validated to confirm the
   starter project does NOT contain the constructs that students must add
   (functions for L5, variables for L6, parameters for L7, creative elements
   for L8).
4. **Student augmentation** — the baseline AST is augmented with the AST
   constructs a student would add during the lesson.
5. **Grading pipeline** — the augmented program is passed through the
   lesson-specific grading function, which evaluates each step in dependency
   order.
6. **Pass assertion** — all grading steps report `Ready` and the overall report
   `passed` is `true`.

The tests do NOT prove:

- Full UI automation of the lesson workflow
- Creative quality of student programs
- Production readiness of Alice
- Lesson completion through the entire student hook chain

## Test pipeline

Each real-Alice test follows the same 6-phase pipeline:

```text
┌──────────────────┐
│ 1. Check gate    │  EATME_REAL_ALICE=1?
└────────┬─────────┘
         │ yes
┌────────▼─────────┐
│ 2. Launch Alice  │  run_launch_smoke(&LaunchSmokeOptions { scenario, .. })
└────────┬─────────┘
         │ manifest.failure_category.is_none()
┌────────▼─────────┐
│ 3. Parse .a3p    │  a3p_parser_support::parse_starter_a3p()
└────────┬─────────┘
         │ Program { procedures }
┌────────▼─────────┐
│ 4. Verify base   │  assert baseline lacks student constructs
└────────┬─────────┘
         │
┌────────▼─────────┐
│ 5. Augment AST   │  add student-authored constructs
└────────┬─────────┘
         │ augmented Program
┌────────▼─────────┐
│ 6. Grade         │  grade_<lesson>(input with augmented program)
└──────────────────┘
         │ assert report.passed == true
```

## Per-lesson test details

### L5 — Functions (`functions_e2e.rs`)

**Test name:** `real_alice_functions_grading_integration`

**Scenario:** `functions-as-questions-about-the-world`

**Baseline AST expectations:**
- `program.procedures` contains at least one `Procedure` (the default
  `myFirstMethod`)
- `program.functions` is empty (functions are a student-added construct)
- No `FunctionCall` statements exist in any procedure body
- No `ReturnStatement` nodes exist in any function body

**Student augmentation:** Adds a `Function` (with `return_type` and a
`ReturnStatement` in its body) to `program.functions`, plus a `FunctionCall`
statement inside an existing procedure's body.

**Grading steps (8 total):**

| Step | Name | Expected status |
| --- | --- | --- |
| 1 | `validate-assets` | `ready` |
| 2 | `check-dependencies` | `ready` |
| 3 | `launch-smoke` | `ready` |
| 4 | `create-function` | `ready` |
| 5 | `add-return-statement` | `ready` |
| 6 | `call-function-from-procedure` | `ready` |
| 7 | `run-world` | `ready` |
| 8 | `save-project` | `ready` |

**Run directory:** `target/test-work/functions-real`

### L6 — Variables (`variables_e2e.rs`)

**Test name:** `real_alice_variables_grading_integration`

**Scenario:** `variables-scorekeeper-timekeeper`

**Baseline AST expectations:**
- `program.procedures` contains at least one `Procedure`
- No `VariableDeclaration` statements exist in any procedure body
- No `VariableAssignment` statements exist in any procedure body
- `MethodCall` arguments do not reference variables (all start with `"`)

**Student augmentation:** Adds a `VariableDeclaration` statement, a
`MethodCall` with a non-literal (variable) argument, and a
`VariableAssignment` statement to a procedure body.

**Grading steps (8 total):**

| Step | Name | Expected status |
| --- | --- | --- |
| 1 | `validate-assets` | `ready` |
| 2 | `check-dependencies` | `ready` |
| 3 | `launch-smoke` | `ready` |
| 4 | `declare-variable` | `ready` |
| 5 | `use-variable-in-method` | `ready` |
| 6 | `modify-variable` | `ready` |
| 7 | `run-world` | `ready` |
| 8 | `save-project` | `ready` |

**Run directory:** `target/test-work/variables-real`

### L7 — Parameters (`parameters_e2e.rs`)

**Test name:** `real_alice_parameters_grading_integration`

**Scenario:** `reusable-methods-and-parameters`

**Baseline AST expectations:**
- `program.procedures` contains at least one `Procedure`
- All `Procedure::parameters` fields are empty
- `MethodCall::arguments` are empty or contain only literal values

**Student augmentation:** Adds a `Parameter` to one procedure's `parameters`
field and includes a `MethodCall` statement with non-empty `arguments`.

**Grading steps (7 total):**

| Step | Name | Expected status |
| --- | --- | --- |
| 1 | `validate-assets` | `ready` |
| 2 | `check-dependencies` | `ready` |
| 3 | `launch-smoke` | `ready` |
| 4 | `create-parameterized-procedure` | `ready` |
| 5 | `call-with-argument` | `ready` |
| 6 | `run-world` | `ready` |
| 7 | `save-project` | `ready` |

**Run directory:** `target/test-work/parameters-real`

### L8 — Creative project (`creative_project_e2e.rs`)

**Test name:** `real_alice_creative_project_grading_integration`

**Scenario:** `design-process-story-or-game`

**Baseline AST expectations:**
- `program.procedures` contains at least one `Procedure`
- May have `MethodCall` and `IfElse` statements
- No `EventListener` or `CollisionListener` statements exist
- May have only 1 procedure (students add more)

**Student augmentation:** Ensures ≥2 `MethodCall` statements (scene building
evidence), ≥2 procedures or a parameterized procedure, at least one control
structure (`CountLoop` or `IfElse`), and at least one `EventListener` or
`CollisionListener`.

**Grading steps (9 total):**

| Step | Name | Expected status |
| --- | --- | --- |
| 1 | `validate-assets` | `ready` |
| 2 | `check-dependencies` | `ready` |
| 3 | `launch-smoke` | `ready` |
| 4 | `build-scene-with-objects` | `ready` |
| 5 | `create-custom-procedure` | `ready` |
| 6 | `add-control-structure` | `ready` |
| 7 | `add-event-or-interaction` | `ready` |
| 8 | `run-world` | `ready` |
| 9 | `save-project` | `ready` |

**Run directory:** `target/test-work/creative-project-real`

## AST augmentation per lesson

The real-Alice tests parse a real `.a3p` starter project to get the baseline
AST, verify the baseline lacks student-added constructs, then augment the AST
before grading. This models the student journey: they start with a baseline
project and add constructs during the lesson.

### AST types used per lesson

| AST type | L5 Functions | L6 Variables | L7 Parameters | L8 Creative |
| --- | --- | --- | --- | --- |
| `Function` (struct) | ✓ added to `program.functions` | | | |
| `ReturnStatement` | ✓ added inside function body | | | |
| `FunctionCall` | ✓ added in procedure body | | | |
| `VariableDeclaration` | | ✓ added | | |
| `VariableAssignment` | | ✓ added | | |
| `Parameter` (struct) | | | ✓ added to `procedure.parameters` | may be present |
| `EventListener` | | | | ✓ added |
| `CollisionListener` | | | | ✓ added |
| `CountLoop` | | | | ✓ present or added |
| `IfElse` | | | | may be present |
| `MethodCall` | present | ✓ with non-literal arg | ✓ with non-empty arguments | ✓ ≥2 required |
| `Procedure` | present | present | present (✓ with parameters) | ✓ ≥2 required |

### Augmentation strategy

Each test follows an additive-only strategy:

1. Parse the real `.a3p` to get the baseline `Program`.
2. Assert the baseline lacks the student-added constructs.
3. Clone the baseline and add constructs via `program.functions.push(...)`,
   `procedure.body.push(...)`, or `procedure.parameters.push(...)`.
4. The augmented `Program` is passed to the grading function.

Note: `Function` is a top-level struct on `Program`, not a `Statement` variant.
`Parameter` is a struct on `Procedure`, not a `Statement` variant. Both were
added to the AST in the L5–L8 extension on `main`.

This approach mirrors what a student does in Alice: they start with a starter
project and add code to it. The test verifies the grading pipeline correctly
detects the added constructs.

## Grading step expectations

All four lessons use `ast_check_step` for their lesson-specific steps. This
helper produces only `Ready` or `Blocked` status values — never
`NotYetTested`. When the augmented program contains all required constructs,
all steps report `Ready` and the report `passed` field is `true`.

The three precondition steps (`validate-assets`, `check-dependencies`,
`launch-smoke`) also resolve to `Ready` in these tests because the real-Alice
launch succeeds.

## Shared test support modules

The real-Alice tests reuse helpers from the existing real-Alice test
infrastructure and add a new `.a3p` parser module:

| Module | Import | Purpose |
| --- | --- | --- |
| `launch_smoke_support` | `mod launch_smoke_support;` | `PathOverride` for `$PATH` management, `TestFixture` for fake toolchains (used by synthetic tests). The real-Alice tests also use `run_launch_smoke` from `eatme_alice` directly. |
| `a3p_parser_support` | `mod a3p_parser_support;` | `parse_starter_a3p(project_path)` parses a `.a3p` file and returns a `Program` AST. Uses regex parsing to extract `Procedure` bodies from Tweedle source. **New module** created alongside these tests. |

The real-Alice environment gate (`real_alice_enabled()`) and Alice home
resolution (`alice_home()`) are defined inline in each test file, following
the pattern from `launch_smoke_real.rs`.

Note: The L3 (`loops_and_conditionals_e2e.rs`) and L4
(`events_and_collision_e2e.rs`) tests do not currently have real-Alice
variants — they only contain synthetic fixture tests. The L5–L8 tests are the
first lesson-grading tests to include the full real-Alice pipeline.

### `launch_smoke_support` helpers

```rust
// PathOverride for synthetic tests (not used in real-Alice tests)
let fixture = launch_smoke_support::TestFixture::new();
let _path_override = launch_smoke_support::PathOverride::prepend(&fixture.bin);
```

### Environment gate (inline per test file)

```rust
fn real_alice_enabled() -> bool {
    std::env::var("EATME_REAL_ALICE")
        .map(|v| v == "1")
        .unwrap_or(false)
}

fn alice_home() -> std::path::PathBuf {
    std::path::PathBuf::from(
        std::env::var("ALICE_HOME").unwrap_or_else(|_| "/opt/alice3".into()),
    )
}
```

### `a3p_parser_support` helpers

```rust
// Parse a starter project's .a3p file into an AST
let program = a3p_parser_support::parse_starter_a3p(&starter_project_path);
```

## Configuration

### Real-Alice test options

Each real-Alice test configures `LaunchSmokeOptions` with:

| Option | Value | Rationale |
| --- | --- | --- |
| `alice_home` | `alice_home()` | Resolved from `ALICE_HOME` or default `/opt/alice3`. |
| `scenario` | `LaunchSmokeScenario::new("<scenario-id>")` | Lesson-specific scenario from `assets/scenarios/eatme/`. Uses the default starter project (`africa.a3p`). Override `starter_project` if the lesson needs a different `.a3p`. |
| `run_id` | `<lesson>-real` | Kebab-case identifier (e.g., `functions-real`). |
| `runs_dir` | `target/test-work/<lesson>-real/runs` | Isolated under `target/` to avoid polluting project root. |
| `timeout_seconds` | `900` | 15-minute timeout for cold Maven builds and slow Java startup. |
| `json` | `true` | Machine-readable output. |
| `no_memory` | `true` | No persistent memory side effects from test runs. |
| `offline_package` | `true` | Uses cached Maven dependencies, no network access. |

### Host requirements

The real-Alice integration tests require a Linux host with all Alice desktop
dependencies. See
[Deterministic Real-Alice Smoke Test — Host requirements](deterministic-real-alice-smoke-test.md#host-requirements)
for the full dependency list and install command.

### Environment variables

| Variable | Required | Purpose |
| --- | --- | --- |
| `EATME_REAL_ALICE` | Yes (for real-Alice tests) | Gate variable. Must be exactly `1`. |
| `ALICE_HOME` | Recommended | Alice checkout path. Defaults to `/opt/alice3`. |
| `TMPDIR` | Recommended | Set to `/tmp` to avoid Unix socket path length errors in deep worktrees. |
| `NODE_OPTIONS` | No | Not needed; no Node processes are launched. |

## Examples

### Run L5 real-Alice test with verbose output

```bash
TMPDIR=/tmp EATME_REAL_ALICE=1 cargo test -p eatme-alice \
  --test functions_e2e real_alice_functions_grading_integration \
  -- --nocapture
```

### Run all real-Alice lesson tests in sequence

```bash
for test in functions_e2e variables_e2e parameters_e2e creative_project_e2e; do
  echo "--- Running ${test} ---"
  TMPDIR=/tmp EATME_REAL_ALICE=1 cargo test -p eatme-alice \
    --test "${test}" real_alice -- --test-threads=1 --nocapture
done
```

### Run only synthetic fixture tests (CI-safe)

```bash
TMPDIR=/tmp cargo test -p eatme-alice \
  --test functions_e2e \
  --test variables_e2e \
  --test parameters_e2e \
  --test creative_project_e2e \
  -- --test-threads=1
```

Output includes:

```text
test real_alice_functions_grading_integration ... ok   (skipped: EATME_REAL_ALICE not set)
test functions_grading_all_ready_with_complete_program ... ok
test functions_grading_blocked_without_program ... ok
...
```

When `EATME_REAL_ALICE` is not set, the real-Alice test prints a skip message
and passes without exercising Alice.

### Inspect grading evidence after a real run

```bash
ls target/test-work/functions-real/runs/
```

The run directory contains the launch smoke manifest, Alice logs, screenshot,
and window list artifacts from the real Alice session.

### Run a single synthetic test

```bash
TMPDIR=/tmp cargo test -p eatme-alice --test variables_e2e \
  variables_grading_all_ready_with_complete_program
```

## Troubleshooting

### Test skips unexpectedly

Verify the environment variable is set to exactly `1`:

```bash
echo $EATME_REAL_ALICE   # should print: 1
```

The check is `std::env::var("EATME_REAL_ALICE") == Ok("1".into())`. Values
like `true`, `yes`, or empty string do not activate the test.

### Launch smoke fails (manifest has failure_category)

The real-Alice test asserts `manifest.failure_category.is_none()`. If the
launch fails, check:

1. Alice checkout is valid: `ls ${ALICE_HOME}/alice-ide/`
2. Maven build succeeds: `cargo run -q -p eatme-cli -- alice package --alice-home "${ALICE_HOME}" --offline --json`
3. Desktop tools are available: `cargo run -q -p eatme-cli -- deps check --json`

### Baseline AST assertion fails

If the test fails on "expected baseline to lack <construct>" assertions, the
starter project's `.a3p` file may have been updated to include constructs that
previously were absent. Check the starter project and update baseline
expectations accordingly.

### Grading report not all Ready

If the augmented program fails grading, the augmentation may be insufficient.
Check the grading function's step requirements against the augmented AST:

- L5: needs `Function` (in `program.functions`) with `ReturnStatement` + `FunctionCall` in procedure body
- L6: needs `VariableDeclaration` + `VariableAssignment` + `MethodCall` with non-literal arg
- L7: needs `Parameter` on a procedure + `MethodCall` with non-empty arguments
- L8: needs ≥2 `MethodCall` + ≥2 procedures (or parameterized) + control structure + `EventListener`/`CollisionListener`

### Unix socket path too long

In deep worktree paths, use `TMPDIR=/tmp`:

```bash
TMPDIR=/tmp EATME_REAL_ALICE=1 cargo test -p eatme-alice --test functions_e2e
```

### Module too long (quality gate failure)

All Rust source modules must stay at or below 500 lines. Expected file sizes
after adding real-Alice tests:

| File | Expected lines | Limit |
| --- | --- | --- |
| `crates/eatme-alice/tests/functions_e2e.rs` | ~330 | 500 |
| `crates/eatme-alice/tests/variables_e2e.rs` | ~230 | 500 |
| `crates/eatme-alice/tests/parameters_e2e.rs` | ~200 | 500 |
| `crates/eatme-alice/tests/creative_project_e2e.rs` | ~250 | 500 |

If any file approaches 500 lines, extract the real-Alice test into a separate
`<lesson>_e2e_real.rs` test file.

### `cargo test` fails with unresolved import

The grading modules must be exported from `eatme-assets`. Verify:

- `crates/eatme-assets/src/lib.rs` re-exports the grading functions:
  `pub use grading_report_functions::{FunctionsGradingInput, grade_functions};`
  (and corresponding re-exports for variables, parameters, creative)
- `crates/eatme-alice/Cargo.toml` lists `eatme-assets` in `[dev-dependencies]`

## Authoring workflow

Use this workflow when adding a new real-Alice lesson grading test.

1. **Verify the scenario YAML exists.** Check
   `assets/scenarios/eatme/<scenario-id>.yaml`. If missing, create it following
   [Scenario Authoring](scenario-authoring.md).

2. **Add the grading module.** Create `grading_report_<lesson>.rs` in
   `crates/eatme-assets/src/` following the pattern in `grading_report_events.rs`:
   input struct, grading function, AST check helpers.

3. **Add synthetic E2E tests first.** Create the test file in
   `crates/eatme-alice/tests/` with synthetic fixture tests that exercise the
   grading pipeline without real Alice. These run on every `cargo test`.

4. **Add the real-Alice test.** Import `launch_smoke_support` and
   `a3p_parser_support`. Follow the 6-phase pipeline:
   gate → launch → parse → verify baseline → augment → grade.

5. **Keep augmentation additive.** Only add constructs to the parsed AST; never
   remove baseline content. This mirrors the student experience.

6. **Run the full suite:**

   ```bash
   TMPDIR=/tmp cargo test -p eatme-alice -- --test-threads=1
   TMPDIR=/tmp EATME_REAL_ALICE=1 cargo test -p eatme-alice --test <test> \
     real_alice -- --nocapture
   ```

7. **Run the quality gate:**

   ```bash
   TMPDIR=/tmp ./scripts/quality-gates.sh
   ```

## Related documentation

- [Deterministic Real-Alice Smoke Test](deterministic-real-alice-smoke-test.md)
  — the baseline real-Alice launch smoke test pattern these tests extend.
- [Loops and Conditionals Grading](loops-and-conditionals-grading.md) — the L3
  grading report and E2E test that established the grading pipeline pattern.
- [Events and Collision Grading](events-and-collision-grading.md) — the L4
  grading report and E2E test pattern.
- [Alice Lesson Smoke](alice-lesson-smoke.md) — desktop scenario roster and
  evidence contracts.
- [Outside-in Alice Test Modules](outside-in-alice-test-modules.md) — Rust test
  module layout and authoring workflow.
- [Creative Assessment Boundary](creative-assessment-boundary.md) — the boundary
  between machine-assessable and human-review-needed aspects.
- [Student Missions](student-missions.md) — the classroom mission descriptions
  for all lessons.
- [Validation and Quality Gates](validation-quality-gates.md) — the 500-line
  module size gate and other quality checks.
- [Scenario Authoring](scenario-authoring.md) — how to author scenario YAML
  files including the four lesson scenarios.
