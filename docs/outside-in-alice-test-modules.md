# Outside-in Alice test modules

The outside-in Alice expansion tests are split into small Rust modules so the
asset contract tests stay readable and every Rust source file remains within the
500-line module-size gate.

## Contents

- [Usage](#usage)
- [Module layout](#module-layout)
- [Internal helper API](#internal-helper-api)
- [Configuration](#configuration)
- [Examples](#examples)
- [Authoring workflow](#authoring-workflow)
- [Maintenance checklist](#maintenance-checklist)

## Usage

Run the focused outside-in Alice expansion tests with:

```bash
cargo test -p eatme-assets outside_in_alice_expansion_tests
```

Run the full `eatme-assets` crate tests with:

```bash
cargo test -p eatme-assets
```

Run the repository quality gate when a change touches test wiring, assets,
generated adapters, or evidence wording:

```bash
./scripts/quality-gates.sh
```

The quality gate includes `cargo fmt --check`, clippy, workspace tests, coverage,
and the Rust module-size check. The module-size check requires each Rust source
file under `crates/` to be 500 lines or fewer.

## Module layout

The parent harness remains wired through `crates/eatme-assets/src/lib.rs`:

```rust
#[cfg(test)]
mod outside_in_alice_expansion_tests;
```

The parent module owns shared constants, path helpers, YAML loading helpers, and
assertion helpers. Focused child modules own the tests:

| File | Responsibility |
| --- | --- |
| `crates/eatme-assets/src/outside_in_alice_expansion_tests.rs` | Parent harness with shared helpers and child module declarations. |
| `crates/eatme-assets/src/outside_in_alice_expansion_tests/scenario_contracts.rs` | Scenario structure, persona coverage, real-Alice gate, and honest-boundary contract tests. |
| `crates/eatme-assets/src/outside_in_alice_expansion_tests/first_lesson_evidence.rs` | First-lesson readiness evidence tests and wording checks. |
| `crates/eatme-assets/src/outside_in_alice_expansion_tests/gadugi_adapters.rs` | Asset inventory, validation, and generated Gadugi adapter freshness tests. |
| `crates/eatme-assets/src/outside_in_alice_expansion_tests/asset_validation.rs` | Validation API regression tests for persona/scenario compatibility failures. |

The split is mechanical. It does not change asset behavior, readiness claims,
assertion language, generated adapter expectations, or validation semantics.

## Internal helper API

These helpers are test-only implementation details of the parent module. Child
modules import them with `super::` and should keep visibility as narrow as Rust
allows.

| Helper | Purpose |
| --- | --- |
| `EXPECTED_SCENARIO_ASSET_COUNT` | Expected full scenario YAML inventory count for the expansion asset contract. |
| `TARGET_SCENARIOS` | Outside-in Alice expansion scenarios and their required instructor/student personas. |
| `FIRST_LESSON_SMOKE_READY_EVIDENCE_COUNT` | Stable count for `first-lessons-real-ui-actions` `smoke_ready.evidence`. |
| `FIRST_LESSON_REQUIRED_SMOKE_READY_EVIDENCE` | Required first-lesson evidence identifiers that must remain explicit. |
| `repository_root()` | Resolves the repository root from the crate manifest directory. |
| `scenario_path(root, collection, id)` | Builds a canonical or Gadugi scenario YAML path. |
| `read_eatme_scenario(path)` | Reads and parses an `EatmeScenarioAsset`. |
| `assert_contains_all(label, text, needles)` | Fails with explicit missing-evidence language when required snippets are absent. |
| `assert_not_contains_any(label, text, needles)` | Fails when forbidden wording appears. |
| `forbidden_internal_shorthand()` | Returns internal shorthand terms that must not leak into portable docs or assets. |
| `normalize_whitespace(text)` | Normalizes text before evidence-language comparisons. |

No helper is part of the public crate API. The split keeps helper scope inside
the test module tree instead of promoting test internals to `pub(crate)`.

## Configuration

The Rust contract tests do not require real Alice desktop execution and do not
require Node. They read committed assets and docs, run the asset validator, and
compare generated Gadugi adapters in check mode.

| Setting | Required for these tests | Purpose |
| --- | --- | --- |
| `NODE_OPTIONS=--max-old-space-size=32768` | No | Safe to export for repository-wide quality workflows that invoke Node-based tooling. |
| `EATME_REAL_ALICE=1` | No | Required only for real Alice launch-smoke runs, not for these Rust contract tests. |
| `ALICE_HOME` | No | Required only when running real Alice launch commands. |

Use the repository root as the working directory for the commands in this guide.

## Examples

### Run only the first-lesson evidence contract tests

```bash
cargo test -p eatme-assets outside_in_alice_expansion_tests::first_lesson_evidence
```

The first-lesson tests keep the evidence boundary explicit. In particular,
`first_lesson_evidence_contracts_stay_explicit_and_honest` continues to assert
that the student, instructor, launch, and documentation contracts include the
same honest readiness language:

```text
first-lessons-real-ui-actions
instructor-lesson-materials-remix
real-alice-launch-smoke
preflight automation scenario readiness evidence only
not full UI automation
not creative assessment
not learner-world grading
not production readiness
not lesson completion
```

### Check generated adapter freshness without writing files

```bash
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
```

The `gadugi_adapters` test module uses the same generator check mode from Rust.
It verifies that each target expansion scenario has both a canonical eatme asset
and a generated Gadugi adapter, and that generated adapters are fresh.

### Confirm module sizes

```bash
find crates -name '*.rs' -not -path '*/target/*' -exec wc -l {} + \
  | awk '$2 != "total" && $1 > 500 { print; bad=1 } END { exit bad }'
```

The parent harness and each child module are independent Rust source files, so
adding a new test to one area does not force unrelated contracts into the same
500-line budget.

## Authoring workflow

Use this workflow when adding or moving an outside-in Alice expansion contract
test.

1. Pick the child module that owns the contract boundary:

   | Test concern | Add it to |
   | --- | --- |
   | Scenario kind, launcher, personas, honest boundary wording | `scenario_contracts.rs` |
   | First-lesson readiness evidence and explicit no-overclaim language | `first_lesson_evidence.rs` |
   | Asset inventory count, validation pass/fail, generated adapter freshness | `gadugi_adapters.rs` |
   | Persona crew validation failures against scenario asset ids | `asset_validation.rs` |

2. Reuse the parent helpers through `super::` instead of duplicating path,
   parsing, or wording-normalization code.

3. Preserve assertion messages and evidence snippets when moving existing tests.
   For the first-lesson readiness evidence contract, keep
   `first_lesson_evidence_contracts_stay_explicit_and_honest` semantically
   intact. Do not weaken required snippets, forbidden snippets, or failure
   messages to make a wording change pass.

4. Keep public claims honest. Launch-smoke evidence proves scenario-labeled
   startup evidence. It does not prove full UI automation, creative assessment,
   learner-world grading, production readiness, lesson completion, complete
   end-to-end lesson execution, or broad Alice compatibility.

5. Run the focused tests and the module-size check:

   ```bash
   cargo test -p eatme-assets outside_in_alice_expansion_tests
   find crates -name '*.rs' -not -path '*/target/*' -exec wc -l {} + \
     | awk '$2 != "total" && $1 > 500 { print; bad=1 } END { exit bad }'
   ```

## Maintenance checklist

Before merging a change that touches these tests:

| Check | Command |
| --- | --- |
| Format Rust files | `cargo fmt --check` |
| Run focused expansion tests | `cargo test -p eatme-assets outside_in_alice_expansion_tests` |
| Run all asset crate tests | `cargo test -p eatme-assets` |
| Validate assets | `cargo run -q -p eatme-cli -- assets validate --json` |
| Check generated adapters | `cargo run -q -p eatme-cli -- assets generate-gadugi --check --json` |
| Enforce Rust module size | `./scripts/quality-gates.sh` |

Use `./scripts/quality-gates.sh` when a single command should run the repository
Rust quality gate.
