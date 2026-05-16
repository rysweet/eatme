# Real-Alice AST grading integration tests

The real-Alice AST grading integration test exercises all five grading
pipelines — loops, events, functions, variables, and parameters — against real
Alice `.a3p` starter project files. It parses the Alice XML inside `.a3p` ZIP
archives into eatme AST types using regex-based extraction, then feeds the
resulting `Program` to each grading function and verifies the report structure.

These tests are gated behind the `EATME_REAL_ALICE=1` environment variable.
When the gate is not satisfied, the tests skip automatically. Parser unit tests
that verify regex extraction from XML snippets run unconditionally on every
`cargo test` invocation.

## Contents

- [Usage](#usage)
- [Environment gate](#environment-gate)
- [A3P file format](#a3p-file-format)
- [A3P parser](#a3p-parser)
- [Parser unit tests](#parser-unit-tests)
- [Integration tests](#integration-tests)
- [What the tests prove](#what-the-tests-prove)
- [API surface](#api-surface)
- [Configuration](#configuration)
- [Examples](#examples)
- [Troubleshooting](#troubleshooting)
- [Related documentation](#related-documentation)

## Usage

Run the real-Alice integration tests (requires Alice installation):

```bash
EATME_REAL_ALICE=1 ALICE_HOME=/opt/alice3 \
  cargo test -p eatme-alice --test real_ast_grading -- --nocapture
```

Run only the parser unit tests (no Alice required):

```bash
cargo test -p eatme-alice --test real_ast_grading -- parser
```

Run all `eatme-alice` tests (real-Alice tests skip automatically when the
environment variable is absent):

```bash
cargo test -p eatme-alice
```

## Environment gate

| Variable | Required value | Effect |
| --- | --- | --- |
| `EATME_REAL_ALICE` | `1` | Enables the real-Alice integration tests. Any other value or absence causes the tests to skip. |
| `ALICE_HOME` | Path to built Alice checkout | The Alice checkout directory containing starter projects. Required when `EATME_REAL_ALICE=1`. |

The gate is a runtime `std::env::var` check, not a compile-time `cfg`
attribute. This means:

- `cargo test -p eatme-alice` always compiles the test file.
- The test binary always includes all real-Alice grading tests.
- Each test body returns early when the gate is not satisfied.
- CI workflows that set `EATME_REAL_ALICE=1` on self-hosted runners with Alice
  installations get the full integration validation.
- Parser unit tests are **not gated** — they run unconditionally.

## A3P file format

Alice projects are saved as `.a3p` files, which are ZIP archives containing XML
files that describe the scene graph, procedures, and code. The relevant XML
files for AST extraction are:

| ZIP entry | Content |
| --- | --- |
| `*.xml` (program entries) | Alice XML containing `<userMethod>`, `<userFunction>`, `<localDeclarationStatement>`, `<countLoop>`, `<ifElse>`, `<eventListener>`, `<collisionListener>`, `<returnStatement>`, `<expressionStatement>` elements |

The parser does not extract all XML content — it targets only the elements
needed to populate the eatme AST types.

## A3P parser

The `parse_a3p_program()` function reads a `.a3p` file from disk, opens it as
a ZIP archive, locates the program XML entries, and extracts AST constructs
using regex patterns.

### Parser architecture

```text
.a3p file (ZIP)
  │
  ├─ Open with zip crate (read-only, no disk extraction)
  │
  ├─ Find XML entries containing program code
  │
  ├─ For each XML entry:
  │   ├─ extract_procedures(xml) → Vec<Procedure>
  │   │   ├─ Regex: UserMethod → Procedure name + body
  │   │   ├─ Regex: UserParameter → Parameter name + type
  │   │   └─ extract_statements(body_xml) → Vec<Statement>
  │   │       ├─ Regex: ExpressionStatement → MethodCall
  │   │       ├─ Regex: CountLoop → CountLoop
  │   │       ├─ Regex: ConditionalStatement → IfElse
  │   │       ├─ Regex: EventListener → EventListener
  │   │       ├─ Regex: CollisionListener → CollisionListener
  │   │       ├─ Regex: FunctionInvocation → FunctionCall
  │   │       ├─ Regex: ReturnStatement → ReturnStatement
  │   │       ├─ Regex: LocalDeclarationStatement → VariableDeclaration
  │   │       └─ Regex: AssignmentExpression → VariableAssignment
  │   │
  │   ├─ extract_functions(xml) → Vec<Function>
  │   │   ├─ Regex: UserFunction → Function name + return_type + body
  │   │   └─ extract_statements(body_xml) → Vec<Statement>  (reuses same fn)
  │   │
  │   └─ extract_variable_declarations(xml) → Vec<VariableDeclaration>
  │       └─ Regex: LocalDeclarationStatement (scene-level) → name + type
  │
  └─ Assemble Program { procedures, functions, variable_declarations }
```

### Regex patterns

The parser uses Rust's `regex` crate (Thompson NFA — immune to catastrophic
backtracking). Patterns are compiled once into a lazy `RegexCache`:

| Pattern name | Alice XML element | AST output |
| --- | --- | --- |
| `user_method` | `<userMethod>` | `Procedure` with name |
| `user_parameter` | `<parameter>` inside method | `Parameter` { name, param_type } |
| `expression_statement` | `<expressionStatement>` with method call | `Statement::MethodCall` |
| `count_loop` | `<countLoop>` | `Statement::CountLoop` |
| `conditional` | `<conditionalStatement>` | `Statement::IfElse` |
| `event_listener` | `<eventListener>` | `Statement::EventListener` |
| `collision_listener` | `<collisionListener>` | `Statement::CollisionListener` |
| `function_invocation` | `<functionInvocation>` | `Statement::FunctionCall` |
| `return_statement` | `<returnStatement>` | `Statement::ReturnStatement` |
| `local_declaration` | `<localDeclarationStatement>` | `Statement::VariableDeclaration` |
| `assignment_expression` | `<assignmentExpression>` | `Statement::VariableAssignment` |
| `user_function` | `<userFunction>` | `Function` with name + return_type |

### Safety constraints

- **Read-only ZIP access**: Files are read into memory; nothing is extracted to
  disk.
- **Size cap**: Decompressed content is capped at 50 MB to prevent zip bombs.
- **No unsafe code**: All parsing uses safe Rust.
- **Lenient extraction**: Missing XML elements produce empty collections, not
  panics. The parser uses `unwrap_or_default()` and `Option` returns throughout.

### Return type

```rust
fn parse_a3p_program(a3p_path: &Path) -> anyhow::Result<Program>
```

Returns a fully populated `Program` struct. If the file cannot be opened, is
not a valid ZIP, or contains no recognizable XML, returns an error.

## Parser unit tests

Parser unit tests verify regex extraction from XML snippets. These tests run
unconditionally on every `cargo test` — they do not require Alice or
`EATME_REAL_ALICE`.

### Test inventory

| Test | What it verifies |
| --- | --- |
| `parser_extracts_procedure_name` | `extract_procedures` finds procedure names from `<userMethod>` XML |
| `parser_extracts_procedure_parameters` | `extract_procedures` populates `Procedure.parameters` from `<parameter>` XML |
| `parser_extracts_method_call` | `extract_statements` produces `MethodCall` from `<expressionStatement>` XML |
| `parser_extracts_count_loop` | `extract_statements` produces `CountLoop` from `<countLoop>` XML |
| `parser_extracts_if_else` | `extract_statements` produces `IfElse` from `<conditionalStatement>` XML |
| `parser_extracts_event_listener` | `extract_statements` produces `EventListener` from `<eventListener>` XML |
| `parser_extracts_collision_listener` | `extract_statements` produces `CollisionListener` from `<collisionListener>` XML |
| `parser_extracts_function_call` | `extract_statements` produces `FunctionCall` from `<functionInvocation>` XML |
| `parser_extracts_return_statement` | `extract_statements` produces `ReturnStatement` from `<returnStatement>` XML |
| `parser_extracts_variable_declaration` | `extract_statements` produces `VariableDeclaration` from `<localDeclarationStatement>` XML |
| `parser_extracts_variable_assignment` | `extract_statements` produces `VariableAssignment` from `<assignmentExpression>` XML |
| `parser_extracts_function` | `extract_functions` produces `Function` from `<userFunction>` XML |
| `parser_empty_xml_returns_empty_program` | Empty XML input produces `Program` with empty collections |
| `parser_malformed_xml_does_not_panic` | Truncated or malformed XML returns empty results without panicking |

### Running parser tests

```bash
cargo test -p eatme-alice --test real_ast_grading -- parser
```

## Integration tests

Integration tests feed parsed `.a3p` programs through each grading pipeline and
verify the report structure. These are gated behind `EATME_REAL_ALICE=1`.

### Test inventory

| Test | Starter project | Grading pipeline | What it verifies |
| --- | --- | --- | --- |
| `real_alice_loops_grading` | `amazonMinimum.a3p` | `grade_loops_and_conditionals` | Parser produces a `Program`, report has 7 steps, precondition steps are correct, lesson steps reflect actual AST content |
| `real_alice_events_grading` | `amazonMinimum.a3p` | `grade_events_and_collision` | Report has 7 steps, event/collision step status matches parsed AST content |
| `real_alice_functions_grading` | `amazonMinimum.a3p` | `grade_functions` | Report has 8 steps, function step status matches parsed AST (expected: `blocked` — amazonMinimum has no user functions) |
| `real_alice_variables_grading` | `amazonMinimum.a3p` | `grade_variables` | Report has 8 steps, variable step status matches parsed AST |
| `real_alice_parameters_grading` | `amazonMinimum.a3p` | `grade_parameters` | Report has 7 steps, parameter step status matches parsed AST (expected: `blocked` — amazonMinimum has no parameterized procedures) |
| `real_alice_a3p_parses_without_error` | `amazonMinimum.a3p` | (parser only) | `parse_a3p_program` returns `Ok`, program has at least one procedure |
| `real_alice_a3p_round_trip` | `amazonMinimum.a3p` | (serde only) | Parsed program survives JSON serialize → deserialize round-trip |

### Test assertions

Each integration test asserts:

1. **Parser success**: `parse_a3p_program` returns `Ok(program)`.
2. **Non-empty program**: At least one procedure exists in the parsed program.
3. **Report step count**: Each grading report has the expected number of steps.
4. **Step ordering**: Step names appear in the documented dependency order.
5. **Schema version**: `report.schema_version == "eatme.assets/grading/v1"`.
6. **Lesson name**: `report.lesson` matches the expected lesson identifier.
7. **Status coherence**: If a step is `blocked`, all downstream steps are also
   `blocked`. If `not-yet-tested`, downstream steps evaluate independently.

The tests assert **structure** — not exact content. The amazonMinimum project
may contain different constructs depending on the Alice version. The tests
verify that the grading pipeline processes the parsed program without panicking
and produces a coherent report.

### Starter project location

The tests locate starter projects at:

```text
${ALICE_HOME}/alice-ide/src/main/resources/starter-projects/amazonMinimum.a3p
```

If the file does not exist at this path, the test reports a clear error with the
expected path and the value of `ALICE_HOME`.

## What the tests prove

### Parser unit tests prove

- Regex patterns correctly extract AST constructs from known XML snippets.
- Missing or malformed XML produces empty results without panicking.
- All 11 Alice XML element types map to the correct eatme AST types.

### Integration tests prove

- Real `.a3p` files from a built Alice checkout can be parsed into eatme AST.
- All five grading pipelines accept the parsed program without panicking.
- Grading reports have the correct structure (step count, ordering, schema).
- The parsed AST survives JSON serialization and deserialization.
- The regex parser handles real-world Alice XML (not just test snippets).

### What the tests do NOT prove

- The tests do **not** launch Alice or drive the Alice UI.
- The tests do **not** execute lesson steps (place object, edit procedure, etc.).
- The tests do **not** verify that Alice produces correct XML for a given lesson.
- The tests do **not** assess creative quality or pedagogical outcomes.

## API surface

The integration test uses the following public APIs:

### From `eatme-assets`

```rust
use eatme_assets::{
    // Loops
    grade_loops_and_conditionals, LoopsGradingInput,
    // Events
    grade_events_and_collision, EventsGradingInput,
    // Functions
    grade_functions, FunctionsGradingInput,
    // Variables
    grade_variables, VariablesGradingInput,
    // Parameters
    grade_parameters, ParametersGradingInput,
    // Shared types
    GradingReport, StepStatus,
};
```

### From `eatme-core`

```rust
use eatme_core::ast::{
    Program, Procedure, Function, Parameter, Statement,
    VariableDeclaration,
};
```

### Test-internal (not public)

```rust
// A3P parser — private to the test file
fn parse_a3p_program(a3p_path: &Path) -> anyhow::Result<Program>;
fn extract_procedures(xml: &str) -> Vec<Procedure>;
fn extract_functions(xml: &str) -> Vec<Function>;
fn extract_statements(xml: &str) -> Vec<Statement>;
fn extract_variable_declarations(xml: &str) -> Vec<VariableDeclaration>;
```

The parser functions are test-internal. They are not part of any crate's public
API. If the parser proves stable and useful, it could be promoted to a crate
module in a future iteration.

## Configuration

### Environment variables

| Variable | Required | Default | Purpose |
| --- | --- | --- | --- |
| `EATME_REAL_ALICE` | For integration tests | (unset) | Gate for real-Alice tests. Must be `1` to enable. |
| `ALICE_HOME` | For integration tests | (none) | Path to built Alice checkout. |
| `TMPDIR` | Recommended | `/tmp` | Avoids Unix socket path length errors in deep worktrees. |

### Dev-dependencies

The test file requires two additional dev-dependencies in
`crates/eatme-alice/Cargo.toml`:

| Crate | Purpose |
| --- | --- |
| `zip` | Read `.a3p` ZIP archives |
| `regex` | Extract AST constructs from Alice XML |

These are dev-dependencies only — they are never included in release binaries.

### Workspace Cargo.toml

The `zip` and `regex` crates are also declared in the workspace
`[workspace.dependencies]` section to ensure version consistency:

```toml
[workspace.dependencies]
zip = { version = "0.6", default-features = false, features = ["deflate"] }
regex = "1"
```

## Examples

### Run all real-Alice grading tests

```bash
export ALICE_HOME=/opt/alice3
EATME_REAL_ALICE=1 cargo test -p eatme-alice --test real_ast_grading -- --nocapture
```

### Run only the parser unit tests (no Alice needed)

```bash
cargo test -p eatme-alice --test real_ast_grading -- parser
```

### Run a single integration test

```bash
EATME_REAL_ALICE=1 ALICE_HOME=/opt/alice3 \
  cargo test -p eatme-alice --test real_ast_grading -- real_alice_functions_grading
```

### Inspect parsed AST from a starter project

The integration tests print the parsed program to stdout when run with
`--nocapture`. To see the full AST:

```bash
EATME_REAL_ALICE=1 ALICE_HOME=/opt/alice3 \
  cargo test -p eatme-alice --test real_ast_grading -- \
  real_alice_a3p_parses_without_error --nocapture 2>&1 | head -50
```

### Run all eatme-alice tests (real tests auto-skip)

```bash
cargo test -p eatme-alice
```

Output includes the real-Alice tests with skip messages:

```text
test real_alice_a3p_parses_without_error ... ok (skipped: EATME_REAL_ALICE not set)
test real_alice_a3p_round_trip ... ok (skipped: EATME_REAL_ALICE not set)
test real_alice_loops_grading ... ok (skipped: EATME_REAL_ALICE not set)
test real_alice_events_grading ... ok (skipped: EATME_REAL_ALICE not set)
test real_alice_functions_grading ... ok (skipped: EATME_REAL_ALICE not set)
test real_alice_variables_grading ... ok (skipped: EATME_REAL_ALICE not set)
test real_alice_parameters_grading ... ok (skipped: EATME_REAL_ALICE not set)
```

### Verify the full quality gate

```bash
TMPDIR=/tmp ./scripts/quality-gates.sh
```

The quality gate runs all non-gated tests including the parser unit tests.
Real-Alice integration tests are skipped in the quality gate because
`EATME_REAL_ALICE` is not set.

## Troubleshooting

### Tests skip with "EATME_REAL_ALICE not set"

Set the environment variable to exactly `1`:

```bash
export EATME_REAL_ALICE=1
```

Values like `true`, `yes`, or empty string do not activate the tests.

### "File not found: .../amazonMinimum.a3p"

Verify that `ALICE_HOME` points to a built Alice checkout and that the starter
project exists:

```bash
ls "${ALICE_HOME}/alice-ide/src/main/resources/starter-projects/amazonMinimum.a3p"
```

If the file is missing, the Alice checkout may need to be built first:

```bash
cd "${ALICE_HOME}" && mvn package -pl alice-ide -am -DskipTests
```

### Parser returns empty program

The `.a3p` file may have an unexpected XML structure. Run with `--nocapture` to
see the raw XML being parsed:

```bash
EATME_REAL_ALICE=1 ALICE_HOME=/opt/alice3 \
  cargo test -p eatme-alice --test real_ast_grading -- \
  real_alice_a3p_parses_without_error --nocapture
```

If the XML structure has changed in a newer Alice version, the regex patterns
may need updating. Parser unit tests will continue to pass (they use known XML
snippets), but integration tests may produce empty programs.

### "zip error: invalid Zip archive"

The `.a3p` file may be corrupted or not actually a ZIP archive. Verify with:

```bash
file "${ALICE_HOME}/alice-ide/src/main/resources/starter-projects/amazonMinimum.a3p"
# Expected: Zip archive data
```

### Module too long (quality gate failure)

The test file must stay at or below 500 lines. Expected sizes:

| File | Expected lines | Limit |
| --- | --- | --- |
| `crates/eatme-alice/tests/real_ast_grading.rs` | ~450 | 500 |

If the file exceeds 500 lines, extract the parser into a
`real_ast_grading/parser.rs` submodule and the integration tests into
`real_ast_grading/integration.rs`.

### Compile error: "unresolved import `zip`" or "unresolved import `regex`"

The `zip` and `regex` crates must be listed as dev-dependencies in
`crates/eatme-alice/Cargo.toml`:

```toml
[dev-dependencies]
zip = { workspace = true }
regex = { workspace = true }
```

And in the workspace `Cargo.toml`:

```toml
[workspace.dependencies]
zip = { version = "0.6", default-features = false, features = ["deflate"] }
regex = "1"
```

## Related documentation

- [Functions Grading Report](functions-grading.md) — functions lesson grading
  that this test exercises.
- [Variables Grading Report](variables-grading.md) — variables lesson grading
  that this test exercises.
- [Parameters Grading Report](parameters-grading.md) — parameters lesson
  grading that this test exercises.
- [Loops and Conditionals Grading Report](loops-and-conditionals-grading.md) —
  loops lesson grading that this test exercises.
- [Events and Collision Grading Report](events-and-collision-grading.md) —
  events lesson grading that this test exercises.
- [Deterministic Real-Alice Smoke Test](deterministic-real-alice-smoke-test.md)
  — the launch-smoke real-Alice test that this feature complements.
- [Alice Integration](alice-integration.md) — CLI commands for Alice discovery,
  packaging, and launch smoke.
- [Validation and Quality Gates](validation-quality-gates.md) — the 500-line
  module size gate and other quality checks.
- [Outside-in Alice Test Modules](outside-in-alice-test-modules.md) — Rust test
  module layout and authoring workflow.
