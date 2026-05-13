# Step block composition

Step blocks are shared YAML fragments that eliminate duplicate preflight and
launch-smoke steps across generated Gadugi adapters. The generator embeds step
blocks at compile time and substitutes placeholders during generation, so the
generated adapter output remains self-contained.

## Contents

- [Problem](#problem)
- [How it works](#how-it-works)
- [Step block files](#step-block-files)
- [Placeholders](#placeholders)
- [Discovery exclusion](#discovery-exclusion)
- [Usage](#usage)
- [Adding a new step block](#adding-a-new-step-block)
- [Examples](#examples)
- [Constraints](#constraints)

## Problem

Before step block composition, every generated Gadugi adapter inlined the same
Validate Assets and Check Dependencies steps. With 50 scenarios, that meant 50
copies of identical YAML fragments. Adding a new shared step (such as a lesson
verification hook) required editing the Rust generator function, not a
declarative template.

Step blocks solve this by extracting shared steps into YAML files that the
generator treats as the single source of truth for repeated step patterns.

## How it works

```text
assets/scenarios/gadugi/step-blocks/
├── preflight-steps.yaml          # validate-assets + check-dependencies
└── preflight-alice-steps.yaml    # validate-assets + check-dependencies + discover-alice
```

The generator uses `include_str!()` to embed these files at compile time. During
adapter generation, it reads the embedded template, replaces placeholders with
scenario-specific values, and emits the result as part of the generated adapter
YAML. The final generated `.yaml` files are identical to what the generator
would have produced by inlining the steps directly — no runtime file I/O, no
new dependencies.

```text
┌──────────────────────────┐
│  step-blocks/*.yaml      │  YAML template with {{placeholders}}
│  (source of truth)       │
└──────────┬───────────────┘
           │ include_str!() at compile time
           ▼
┌──────────────────────────┐
│  gadugi.rs generator     │  .replace("{{run-id}}", &run_id)
│  (Rust, compile-time)    │  .replace("{{expected-scenario-asset-count}}", &count)
└──────────┬───────────────┘
           │ generates
           ▼
┌──────────────────────────┐
│  gadugi/*.yaml           │  Self-contained adapter (no references)
│  (generated output)      │
└──────────────────────────┘
```

## Step block files

### `preflight-steps.yaml`

Contains the standard preflight steps used by most generated adapters:

```yaml
- name: Validate Assets
  agent: eatme-cli-agent
  action: execute_command
  params:
    command: |-
      cd "${EATME_REPO:-.}"
      export RUN_ID="${RUN_ID:-{{run-id}}}"
      cargo run -q -p eatme-cli -- assets validate --json
  expect:
    exit_code: 0
    stdout_contains:
    - '"passed": true'
    - '"scenario_asset_count": {{expected-scenario-asset-count}}'
  timeout: 60000
- name: Check Dependencies
  agent: eatme-cli-agent
  action: execute_command
  params:
    command: |-
      cd "${EATME_REPO:-.}"
      export RUN_ID="${RUN_ID:-{{run-id}}}"
      cargo run -q -p eatme-cli -- deps check --json
  expect:
    exit_code: 0
    stdout_contains:
    - '"all_required_available": true'
  timeout: 60000
```

### `preflight-alice-steps.yaml`

Extends the preflight with a Discover Alice step for scenarios that need Alice
discovery before launch:

```yaml
- name: Validate Assets
  agent: eatme-cli-agent
  action: execute_command
  params:
    command: |-
      cd "${EATME_REPO:-.}"
      export RUN_ID="${RUN_ID:-{{run-id}}}"
      cargo run -q -p eatme-cli -- assets validate --json
  expect:
    exit_code: 0
    stdout_contains:
    - '"passed": true'
    - '"scenario_asset_count": {{expected-scenario-asset-count}}'
  timeout: 60000
- name: Check Dependencies
  agent: eatme-cli-agent
  action: execute_command
  params:
    command: |-
      cd "${EATME_REPO:-.}"
      export RUN_ID="${RUN_ID:-{{run-id}}}"
      cargo run -q -p eatme-cli -- deps check --json
  expect:
    exit_code: 0
    stdout_contains:
    - '"all_required_available": true'
  timeout: 60000
- name: Discover Alice
  agent: eatme-cli-agent
  action: execute_command
  params:
    command: |-
      cd "${EATME_REPO:-.}"
      export RUN_ID="${RUN_ID:-{{run-id}}}"
      cargo run -q -p eatme-cli -- alice discover --alice-home ${ALICE_HOME} --json
  expect:
    exit_code: 0
    stdout_contains:
    - '"alice_ide_jar_exists": true'
  timeout: 60000
```

## Placeholders

Step block templates use `{{placeholder}}` syntax. The generator replaces each
placeholder with the scenario-specific value before emitting the adapter YAML.

| Placeholder | Replaced with | Example value |
| --- | --- | --- |
| `{{run-id}}` | `gadugi-<scenario-id>` | `gadugi-real-alice-launch-smoke` |
| `{{expected-scenario-asset-count}}` | Discovered scenario asset count | `101` |

Placeholders are plain string substitution — no template engine, no expression
evaluation. If a placeholder is not replaced, it appears literally in the
output, which causes the byte-equality check to fail and surfaces the error
immediately.

## Discovery exclusion

The `step-blocks/` directory lives under `assets/scenarios/gadugi/` but its
contents are not scenario assets. The discovery function in `discovery.rs`
excludes directories named `step-blocks` so that:

1. Step block files do not inflate `scenario_asset_count`.
2. Step block files are not validated as standalone scenario assets.
3. Existing asset counts and generated adapter expectations are unchanged.

The exclusion is directory-name-based: any directory named `step-blocks` under
the scenario root is skipped during recursive YAML discovery.

## Usage

Step blocks are consumed automatically by the generator. No CLI flags or
configuration changes are needed.

### Validate and regenerate (unchanged workflow)

The workflow for scenario authors is identical to before step blocks were added:

```bash
# Validate all assets
cargo run -q -p eatme-cli -- assets validate --json

# Regenerate adapters
cargo run -q -p eatme-cli -- assets generate-gadugi --json

# Check freshness
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
```

### Verify step blocks are embedded correctly

Run the existing test suite. The byte-equality tests in `gadugi_tests.rs`
verify that generated adapter output has not changed:

```bash
cargo test -p eatme-assets
```

If a step block template is malformed or a placeholder is misspelled, the
generated output changes and the byte-equality tests fail.

## Adding a new step block

1. Create a new YAML file under `assets/scenarios/gadugi/step-blocks/`.
2. Use `{{placeholder}}` syntax for scenario-specific values.
3. In `gadugi.rs`, add an `include_str!()` constant for the new file.
4. Call `.replace()` for each placeholder in the generator function.
5. Insert the expanded steps at the correct position in the generated adapter.
6. Regenerate adapters and verify byte-equality:

   ```bash
   cargo run -q -p eatme-cli -- assets generate-gadugi --json
   cargo test -p eatme-assets
   cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
   ```

7. Commit the step block file, generator change, and regenerated adapters
   together.

### Example: adding a lesson verification hook step

If a future lesson verification hook needs to run after preflight but before
launch, add it to a new step block or extend `preflight-steps.yaml`:

```yaml
- name: Verify Lesson Prerequisite
  agent: eatme-cli-agent
  action: execute_command
  params:
    command: |-
      cd "${EATME_REPO:-.}"
      export RUN_ID="${RUN_ID:-{{run-id}}}"
      cargo run -q -p eatme-cli -- lesson verify-prerequisite --scenario {{scenario-id}} --json
  expect:
    exit_code: 0
    stdout_contains:
    - '"prerequisite_met": true'
  timeout: 60000
```

Then update the generator to handle the new placeholder and regenerate.

## Examples

### Before step blocks (25+ copies of identical steps)

Each generated adapter contained inline preflight steps. Changing the Validate
Assets step text required modifying `gadugi.rs` Rust code:

```rust
// In gadugi.rs — old approach
fn generated_step(scenario: &EatmeScenarioAsset, step: &EatmeScenarioStep, ...) -> GeneratedStep {
    let command = repository_command(step.command.trim(), run_id);
    GeneratedStep {
        name: step_title(&step.id),
        agent: "eatme-cli-agent".into(),
        action: "execute_command".into(),
        params: BTreeMap::from([("command".into(), command)]),
        // ... inline per-step logic
    }
}
```

### After step blocks (single source of truth)

The generator reads the step block template once and substitutes placeholders:

```rust
// In gadugi.rs — step block approach
const PREFLIGHT_STEPS: &str = include_str!("../../../assets/scenarios/gadugi/step-blocks/preflight-steps.yaml");
const PREFLIGHT_ALICE_STEPS: &str = include_str!("../../../assets/scenarios/gadugi/step-blocks/preflight-alice-steps.yaml");

fn preflight_yaml(run_id: &str, expected_count: usize) -> String {
    PREFLIGHT_STEPS
        .replace("{{run-id}}", run_id)
        .replace("{{expected-scenario-asset-count}}", &expected_count.to_string())
}
```

### Generated output is unchanged

The generated adapter YAML is byte-identical before and after the step block
refactor. Compare the Validate Assets step in any generated adapter:

```yaml
- name: Validate Assets
  agent: eatme-cli-agent
  action: execute_command
  params:
    command: |-
      cd "${EATME_REPO:-.}"
      export RUN_ID="${RUN_ID:-gadugi-real-alice-launch-smoke}"
      cargo run -q -p eatme-cli -- assets validate --json
  expect:
    exit_code: 0
    stdout_contains:
    - '"passed": true'
    - '"scenario_asset_count": 101'
  timeout: 60000
```

This output is identical whether the step was generated from inline Rust code
or from the `preflight-steps.yaml` step block.

## Constraints

| Constraint | Rationale |
| --- | --- |
| Compile-time embedding only | No runtime file I/O; works in scratch-root test environments |
| Plain string substitution | No template engine dependency; zero new crate dependencies |
| Byte-identical output | Existing byte-equality tests catch any template drift |
| Discovery exclusion for `step-blocks/` | Step blocks are not scenario assets and must not affect `scenario_asset_count` |
| Instructor adapters out of scope | Instructor agentic flows use `--path` validation and different step shapes; preflight step blocks do not apply |
| No changes to CLI interface | Step blocks are a generator-internal concern; no new flags or commands |
