# Real-Alice e2e grading integration tests

The `real_ast_grading` integration test suite validates the grading
pipelines against programs extracted from real Alice `.a3p` starter projects.
Unlike the unit tests in `eatme-assets` that use synthetic AST fixtures, these
tests load actual `.a3p` ZIP files from `ALICE_HOME`, parse the embedded
`programType.xml` into `eatme-core` `Program` structs, and feed them through
the production grading functions. The tests are gated behind the
`EATME_REAL_ALICE=1` environment variable so CI and developer machines without
Alice installations skip automatically.

Currently, two grading pipelines are implemented: **loops-and-conditionals** and
**events-and-collision**. The remaining four pipelines (functions, variables,
parameters, creative) require new `Statement` variants and grading functions
before their tests can be added. The test file is structured to accommodate
all six pipelines as they are implemented.

The suite proves that the grading pipelines produce correct `StepStatus` values
when faced with real Alice scene data — including correctly identifying missing
constructs as `Blocked` rather than false-positive `Ready`.

In addition to pipeline-level grading tests, the suite includes **independent
AST structure tests** that verify the parsed `Program` contains (or lacks)
specific `Statement` variants before passing it through the grading pipeline.
These AST-shape assertions catch parser regressions that a grading-only test
might miss — for example, a parser bug that silently drops `IfElse` nodes would
still produce the same `Blocked` cascade in the grading test but would fail the
AST structure test.

## Contents

- [Usage](#usage)
- [Environment gate](#environment-gate)
- [The a3p parser](#the-a3p-parser)
- [Test inventory](#test-inventory)
- [Expected outcomes for starter projects](#expected-outcomes-for-starter-projects)
- [API surface](#api-surface)
- [Configuration](#configuration)
- [Examples](#examples)
- [Authoring workflow](#authoring-workflow)
- [Troubleshooting](#troubleshooting)
- [Related documentation](#related-documentation)

## Usage

Run all real-Alice grading integration tests:

```bash
EATME_REAL_ALICE=1 cargo test -p eatme-alice --test real_ast_grading
```

Run a single pipeline test by name:

```bash
EATME_REAL_ALICE=1 cargo test -p eatme-alice --test real_ast_grading \
  real_alice_loops_grading_with_starter_project
```

Run the AST structure test for Lesson 3:

```bash
EATME_REAL_ALICE=1 cargo test -p eatme-alice --test real_ast_grading \
  real_alice_ast_structure_loops_and_conditionals
```

Run the full `eatme-alice` crate test suite (real tests auto-skip when
`EATME_REAL_ALICE` is unset):

```bash
cargo test -p eatme-alice
```

The tests always compile and are always present in the test binary. The
environment gate is a runtime `std::env::var` check using the `real_alice_enabled()`
helper, not a compile-time `cfg` attribute.

## Environment gate

| Variable | Required value | Effect |
| --- | --- | --- |
| `EATME_REAL_ALICE` | `1` | Enables the real-Alice integration tests. Any other value or absence causes each test to skip with a message and pass. |
| `ALICE_HOME` | Path to Alice checkout | The Alice checkout directory containing starter projects under `src/main/resources/starter-projects/`. Defaults to `/opt/alice3` when not set (matching `launch_smoke_real.rs`). |

The gate follows the same runtime pattern used by `launch_smoke_real.rs` and
`first_lesson_vertical_slice.rs`:

```rust
fn real_alice_enabled() -> bool {
    std::env::var("EATME_REAL_ALICE")
        .map(|v| v == "1")
        .unwrap_or(false)
}

fn alice_home() -> PathBuf {
    PathBuf::from(std::env::var("ALICE_HOME").unwrap_or_else(|_| "/opt/alice3".into()))
}
```

When the gate is not satisfied, each test prints a skip message
(`skipping real-Alice grading test (set EATME_REAL_ALICE=1 to enable)`) and
returns early. No `#[ignore]` attribute is used. This means
`cargo test -p eatme-alice` always reports all tests as passing regardless of
whether Alice is installed.

## The a3p parser

The `parse_a3p_program` helper function reads an Alice `.a3p` file (which is a
ZIP archive) and extracts a `Program` struct suitable for grading pipeline
input.

### How it works

1. **Open the ZIP** — uses the `zip` crate's `ZipArchive::new()` to open the
   `.a3p` file as an in-memory ZIP archive.

2. **Collect all XML entries** — iterates every entry in the ZIP archive using
   `ZipArchive::by_index()` and collects the content of all `.xml` files into
   a single string. Alice projects store scene program data across multiple
   XML files; the parser concatenates them for regex matching.

3. **Regex extraction** — applies regex patterns against the XML content to
   identify Alice AST nodes. The parser does not use a full XML parser; it uses
   targeted regex matches on `type=` attribute values, which are stable across
   Alice 3 versions.

4. **Node mapping** — maps Alice XML node types to `eatme-core` AST types:

   | Alice XML type | `eatme-core` type | Status |
   | --- | --- | --- |
   | `UserMethod` | `Procedure` | ✅ Implemented |
   | `MethodInvocation` | `Statement::MethodCall` | ✅ Implemented |
   | `CountLoop` | `Statement::CountLoop` | ✅ Implemented |
   | `ConditionalStatement` | `Statement::IfElse` | ✅ Implemented |
   | `EventListener` | `Statement::EventListener` | ✅ Implemented |
   | `CollisionListener` | `Statement::CollisionListener` | ✅ Implemented |
   | `UserParameter` | *(not yet in AST)* | ⏳ Requires new `Statement` variant |
   | `LocalDeclarationStatement` | *(not yet in AST)* | ⏳ Requires new `Statement` variant |
   | `UserFunction` | *(not yet in AST)* | ⏳ Requires new type on `Program` |

   > **Note:** The current `Statement` enum has five variants: `MethodCall`,
   > `CountLoop`, `IfElse`, `EventListener`, `CollisionListener`. The parser
   > initially maps only these. Variable, parameter, and function types require
   > AST extensions tracked separately.

5. **Build Program** — assembles the extracted nodes into a `Program` struct
   with `procedures` containing the matched statements.

### Signature

```rust
fn parse_a3p_program(path: &std::path::Path) -> Option<Program>
```

### Design decisions

- **Regex over full XML parse.** Alice `.a3p` XML files use deeply nested,
  schema-heavy XML. A regex approach targeting `type=` attributes is simpler,
  faster, and sufficient for detecting construct presence. The grading
  pipelines only need to know "does this construct exist?" — not the full AST
  tree structure.

- **In-memory ZIP reading.** The `ZipArchive::by_index()` method iterates
  entries without extracting to disk, eliminating zip-slip risk.

- **No `unwrap()` on IO/ZIP operations.** All fallible operations use `?` or
  `.ok()` to produce clean test failure messages rather than panics.

### Dependencies

The parser requires the `zip` and `regex` crates as dev-dependencies of
`eatme-alice`:

```toml
[dev-dependencies]
zip = "0.6"
regex = "1"
```

These dependencies are test-only and do not affect the production binary.

## Test inventory

The test file lives at `crates/eatme-alice/tests/real_ast_grading.rs`.

All tests share the same structure:

1. Check the `EATME_REAL_ALICE=1` gate via `real_alice_enabled()`.
2. Resolve `alice_home()` to the starter project directory.
3. Call `parse_a3p_program()` on `amazonMinimum.a3p`.
4. Construct the pipeline-specific grading input with `assets_valid: true`,
   `deps_available: true`, and the parsed program.
5. Call the grading function.
6. Assert `StepStatus` values for each step in the report.

### Implementable now (grading functions exist)

| Test | Type | Grading function | Lesson | Import path |
| --- | --- | --- | --- | --- |
| `real_alice_loops_grading_with_starter_project` | Grading | `grade_loops_and_conditionals` | `loops-and-conditionals-mini-challenge` | `eatme_assets::grade_loops_and_conditionals` |
| `real_alice_ast_structure_loops_and_conditionals` | AST + Grading | `grade_loops_and_conditionals` | `loops-and-conditionals-mini-challenge` | `eatme_assets::grade_loops_and_conditionals` |
| `real_alice_events_grading_with_starter_project` | Grading | `grade_events_and_collision` | `events-collision-proximity-game` | `eatme_assets::grade_events_and_collision` |

**Test type distinction:**

- **Grading** — feeds the parsed program through the grading pipeline and
  asserts `StepStatus` values. Validates the grading function's output contract.
- **AST + Grading** — first performs independent AST-level assertions
  (`Statement::IfElse` present, `Statement::CountLoop` absent) directly against
  the parsed `Program`, then runs the same grading pipeline assertions. This
  catches parser regressions that a grading-only test would mask.

### Requires new grading functions + AST extensions

| Test | Grading function | Blockers |
| --- | --- | --- |
| `real_alice_functions_grading_with_starter_project` | `grade_functions` | Needs `Function` type on `Program`, `FunctionCall`/`ReturnStatement` on `Statement`, and new grading module |
| `real_alice_variables_grading_with_starter_project` | `grade_variables` | Needs `VariableDeclaration`/`VariableAssignment` on `Statement`, and new grading module |
| `real_alice_parameters_grading_with_starter_project` | `grade_parameters` | Needs `UserParameter` representation in AST, and new grading module |
| `real_alice_creative_grading_with_starter_project` | `grade_creative_project` | Needs new grading pipeline (distinct from existing `creative_assessment::for_building_a_scene()` which returns `CreativeAssessmentReport`, not `GradingReport`) |

## Expected outcomes for starter projects

The `amazonMinimum.a3p` starter project is a minimal Alice scene that contains
procedures with method calls and conditional statements. It does **not** contain
counting loops, event listeners, or collision listeners. The tests validate
that pipelines correctly identify these missing constructs.

### Loops and conditionals — grading test (`real_alice_loops_grading_with_starter_project`)

| Step | Expected status | Reason |
| --- | --- | --- |
| `validate-assets` | Ready | Precondition passed |
| `check-dependencies` | Ready | Precondition passed |
| `launch-smoke` | Ready | Preconditions met |
| `build-counting-loop` | **Blocked** | No `CountLoop` in starter project |
| `add-conditional-branch` | **Blocked** | Cascaded from `build-counting-loop` |
| `run-world` | **Blocked** | Cascaded from `add-conditional-branch` |
| `save-project` | **Blocked** | Cascaded from `run-world` |

The starter project has `IfElse` nodes, but because `build-counting-loop` is
blocked first (no `CountLoop`), the conditional check is never reached due to
cascade blocking. This is verified in the source: `evaluate_loops_steps()`
checks `CountLoop` first, and if blocked, calls `cascade_blocked()` on
`add-conditional-branch`.

### Loops and conditionals — AST structure test (`real_alice_ast_structure_loops_and_conditionals`)

This test complements the grading test with independent AST-level assertions
before running the same grading pipeline.

**Phase 1: AST shape assertions**

| Assertion | Expected | Rationale |
| --- | --- | --- |
| `Statement::IfElse` present in procedure bodies | ✅ Yes | `amazonMinimum.a3p` contains `ConditionalStatement` nodes |
| `Statement::CountLoop` present in procedure bodies | ❌ No | Starter project has no counting loops |

The AST assertions walk all procedure bodies via `flat_map` and use `matches!`
to check for variant presence. This validates that the regex-based `.a3p`
parser correctly extracts conditional constructs from the real XML data.

**Phase 2: Grading pipeline verification**

Identical to the grading-only test above — all 7 steps are asserted with the
same expected `StepStatus` values. The grading assertions are repeated
intentionally: the primary value of this test is the AST-level checks, but
verifying grading consistency after those checks confirms the pipeline
behavior is stable across both test paths.

### Events and collision (verified against `grade_events_and_collision`)

| Step | Expected status | Reason |
| --- | --- | --- |
| `validate-assets` | Ready | Precondition passed |
| `check-dependencies` | Ready | Precondition passed |
| `launch-smoke` | Ready | Preconditions met |
| `add-event-listener` | **Blocked** | No `EventListener` in starter project |
| `add-collision-listener` | **Blocked** | Cascaded from `add-event-listener` |
| `run-world` | **Blocked** | Cascaded from `add-collision-listener` |
| `save-project` | **Blocked** | Cascaded from `run-world` |

### Functions (speculative — grading function not yet implemented)

Expected step names and statuses will be determined when `grade_functions` is
implemented. The starter project likely has no user-defined functions (only
procedures via `UserMethod`), so function-related steps should be Blocked.

### Variables (speculative — grading function not yet implemented)

Expected step names and statuses will be determined when `grade_variables` is
implemented. The `Statement` enum does not yet include variable-related
variants, so AST extensions are a prerequisite.

### Parameters (speculative — grading function not yet implemented)

Expected step names and statuses will be determined when `grade_parameters` is
implemented. The `Statement` enum does not yet include parameter-related
variants.

### Creative project (speculative — grading function not yet implemented)

The existing `creative_assessment.rs` provides `for_building_a_scene()` which
returns `CreativeAssessmentReport` — a different type than `GradingReport`. A
new `grade_creative_project` function returning `GradingReport` with step-based
grading would need to be created to match the pattern used by loops and events.

## API surface

The integration tests consume the following public APIs:

### Grading input types (currently implemented)

| Type | Crate | Pipeline | Import |
| --- | --- | --- | --- |
| `LoopsGradingInput` | `eatme-assets` | Loops and conditionals | `eatme_assets::LoopsGradingInput` |
| `EventsGradingInput` | `eatme-assets` | Events and collision | `eatme_assets::EventsGradingInput` |

Both input types share the same shape:

```rust
pub struct XxxGradingInput {
    pub assets_valid: bool,
    pub asset_reason: String,
    pub deps_available: bool,
    pub deps_reason: String,
    pub student_program: Option<Program>,
}
```

> **Note:** The base `GradingInput` type (used by `grade_first_lesson_readiness`)
> does **not** have a `student_program` field. Only the pipeline-specific inputs do.

### Grading functions (currently implemented)

| Function | Crate | Returns |
| --- | --- | --- |
| `grade_loops_and_conditionals(input)` | `eatme-assets` | `GradingReport` |
| `grade_events_and_collision(input)` | `eatme-assets` | `GradingReport` |

### Report types

| Type | Crate | Purpose |
| --- | --- | --- |
| `GradingReport` | `eatme-assets` | Contains `schema_version`, `lesson`, `passed`, and `steps: Vec<StepGrade>`. |
| `StepGrade` | `eatme-assets` | Individual step with `name`, `status: StepStatus`, `reason`, `depends_on`. |
| `StepStatus` | `eatme-assets` | Enum: `Ready`, `Blocked`, `NotYetTested`. |

### AST types

| Type | Crate | Purpose |
| --- | --- | --- |
| `Program` | `eatme-core` | Root AST node containing `procedures: Vec<Procedure>`. |
| `Procedure` | `eatme-core` | Named procedure with `name: String` and `body: Vec<Statement>`. |
| `Statement` | `eatme-core` | Tagged enum with five variants: `MethodCall`, `CountLoop`, `IfElse`, `EventListener`, `CollisionListener`. |

The `Statement` enum currently has no variants for variables, parameters,
functions, or return values. These would need to be added as part of
implementing the remaining four grading pipelines.

## Configuration

### Test-specific settings

| Setting | Required | Default | Purpose |
| --- | --- | --- | --- |
| `EATME_REAL_ALICE` | Yes (to run) | Unset (tests skip) | Enables real-Alice integration tests |
| `ALICE_HOME` | No | `/opt/alice3` | Path to Alice checkout containing starter projects |

### Starter project path resolution

The tests resolve the starter project path as:

```
{ALICE_HOME}/src/main/resources/starter-projects/amazonMinimum.a3p
```

The `ALICE_HOME` default of `/opt/alice3` matches the existing pattern in
`launch_smoke_real.rs` and `first_lesson_vertical_slice.rs`:

```
/opt/alice3/                      # Default ALICE_HOME
└── src/main/resources/
    └── starter-projects/
        ├── amazonMinimum.a3p
        ├── snowPeople.a3p
        └── ...
```

### Dev-dependency

The `zip` and `regex` crates are required as dev-dependencies in
`crates/eatme-alice/Cargo.toml`:

```toml
[dev-dependencies]
eatme-assets = { path = "../eatme-assets" }
eatme-test-support = { path = "../eatme-test-support" }
zip = "0.6"
regex = "1"
```

These crates are used only by the test parser. They are not compiled into
production binaries.

### Host requirements

The real-Alice grading tests require only:

| Dependency | Purpose |
| --- | --- |
| Rust toolchain | Compile and run the tests |
| Alice checkout | Source of `.a3p` starter project files |

No desktop dependencies, Xvfb, Java, or Maven are required. The tests parse
the `.a3p` files in-memory without launching Alice.

## Examples

### Run all pipeline tests

```bash
export ALICE_HOME=/opt/alice3
EATME_REAL_ALICE=1 cargo test -p eatme-alice --test real_ast_grading -- --nocapture
```

Expected output (initially 3 tests, growing as pipelines are added):

```text
running 3 tests
test real_alice_loops_grading_with_starter_project ... ok
test real_alice_ast_structure_loops_and_conditionals ... ok
test real_alice_events_grading_with_starter_project ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### Run without Alice (tests auto-skip)

```bash
cargo test -p eatme-alice --test real_ast_grading -- --nocapture
```

Output:

```text
running 3 tests
test real_alice_loops_grading_with_starter_project ... ok
test real_alice_ast_structure_loops_and_conditionals ... ok
test real_alice_events_grading_with_starter_project ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Each test prints a skip message (e.g., `skipping real-Alice AST structure test
(set EATME_REAL_ALICE=1 to enable)`) before returning.

### Run only the AST structure test

```bash
EATME_REAL_ALICE=1 cargo test -p eatme-alice --test real_ast_grading \
  real_alice_ast_structure_loops_and_conditionals -- --nocapture
```

This is useful when debugging parser regressions — the AST assertions will
pinpoint whether a construct is being dropped by the regex parser, independent
of grading pipeline behavior.

### Verify a specific pipeline against a different starter project

To test against `snowPeople.a3p` instead of `amazonMinimum.a3p`, modify the
starter project constant in the test file. Different starter projects may have
different AST constructs, so expected outcomes will change.

### Inspect what the parser extracts

Add `--nocapture` and use `eprintln!` in the test to see the parsed program:

```bash
EATME_REAL_ALICE=1 cargo test -p eatme-alice --test real_ast_grading \
  real_alice_loops_grading_with_starter_project -- --nocapture
```

The `parse_a3p_program` function returns a `Program` that can be serialized to
JSON for inspection:

```rust
let json = serde_json::to_string_pretty(&program).unwrap();
eprintln!("Parsed program:\n{json}");
```

## Authoring workflow

Use this workflow when adding a new grading pipeline test or supporting a new
starter project.

1. **Add the grading function to `eatme-assets`.** Follow the existing pattern
   in `grading_report.rs` and `grading_report_events.rs`. The function must
   accept an input struct with `student_program: Option<Program>` and return
   `GradingReport`.

2. **Add the test to `real_ast_grading.rs`.** Follow the structure used by
   existing tests. For grading-only tests: gate check → resolve path →
   parse `.a3p` → build input → call grader → assert steps. For AST structure
   tests: add an AST-assertion phase between parsing and grading (see
   `real_alice_ast_structure_loops_and_conditionals` for the pattern).

3. **Determine expected outcomes.** Open the `.a3p` file manually (it's a ZIP)
   and inspect `programType.xml` to identify which AST constructs are present.
   Map them to the expected `StepStatus` values using cascade blocking rules.

4. **Run the test with real data:**

   ```bash
   EATME_REAL_ALICE=1 cargo test -p eatme-alice --test real_ast_grading \
     your_new_test_name -- --nocapture
   ```

5. **Run the full crate suite to confirm zero regressions:**

   ```bash
   cargo test -p eatme-alice
   ```

6. **Update this documentation** with the new test in the inventory table and
   the expected outcomes for any new starter projects.

## Troubleshooting

### Tests skip unexpectedly

Verify the environment variable is set to exactly `1`:

```bash
echo $EATME_REAL_ALICE   # should print: 1
```

The check is `std::env::var("EATME_REAL_ALICE") == Ok("1".into())`. Values
like `true`, `yes`, or empty string do not activate the tests.

### Starter project not found

Check that `ALICE_HOME` points to the Alice checkout root and that the
expected starter project exists:

```bash
ls "${ALICE_HOME}/src/main/resources/starter-projects/amazonMinimum.a3p"
```

If `ALICE_HOME` is not set, the default `/opt/alice3` is used.

### ZIP parse failure

If the `.a3p` file is corrupted or not a valid ZIP, the test will fail with
an `anyhow::Error`. Verify the file is a valid ZIP:

```bash
file "${ALICE_HOME}/src/main/resources/starter-projects/amazonMinimum.a3p"
# Expected: Zip archive data
```

### Unexpected StepStatus values

If a grading pipeline returns different `StepStatus` values than expected,
the starter project may have been updated with new AST constructs. Re-inspect
the `.a3p` contents:

```bash
unzip -p "${ALICE_HOME}/src/main/resources/starter-projects/amazonMinimum.a3p" \
  "*/programType.xml" | grep -oP 'type="[^"]*"' | sort -u
```

Map the extracted types to the [node mapping table](#the-a3p-parser) and
update the expected outcomes accordingly.

### Regex misses a node type

If a new Alice version introduces a different XML attribute format for an
existing construct, the regex may miss it. The parser uses patterns matching
`type="NodeTypeName"` which has been stable across Alice 3.x releases.
If a new format is detected, update the regex patterns in
`parse_a3p_program()`.

## Related documentation

- [Loops and Conditionals Grading](loops-and-conditionals-grading.md) — Loops
  pipeline step definitions, AST model, and synthetic test coverage.
- [Events and Collision Grading](events-and-collision-grading.md) — Events
  pipeline step definitions and cascade blocking rules.
- [Creative Assessment Boundary](creative-assessment-boundary.md) — Machine-
  assessable vs human-review-needed classification for creative projects.
- [Deterministic Real-Alice Smoke Test](deterministic-real-alice-smoke-test.md)
  — The real-Alice launch smoke test that uses the same `EATME_REAL_ALICE=1`
  environment gate pattern.
- [Alice Integration](alice-integration.md) — CLI commands for Alice
  discovery, packaging, and launch.
- [Outside-in Alice Test Modules](outside-in-alice-test-modules.md) — Rust
  test module layout and authoring patterns.
