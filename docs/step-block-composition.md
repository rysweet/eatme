# Step block composition

Step blocks extract duplicated expected-output patterns from the gadugi adapter
generator into reusable YAML templates. Instead of hardcoding the same
`validate-assets`, `check-dependencies`, and `launch-smoke` stdout expectations
across 25+ generated adapters, the generator reads shared step-block templates
and substitutes per-scenario values at generation time.

Generated gadugi YAML is still self-contained — step blocks are inlined during
generation, not referenced at runtime.

## Contents

- [Motivation](#motivation)
- [Directory layout](#directory-layout)
- [Template format](#template-format)
- [Placeholder substitution](#placeholder-substitution)
- [Discovery exclusion](#discovery-exclusion)
- [API reference](#api-reference)
- [Configuration](#configuration)
- [Examples](#examples)
- [Adding a new step block](#adding-a-new-step-block)
- [Editing policy](#editing-policy)
- [Related documentation](#related-documentation)

## Motivation

Before step blocks, every gadugi adapter's `validate-assets` step hardcoded:

```rust
vec!["\"passed\": true".into(), format!("\"scenario_asset_count\": {count}")]
```

The `check-dependencies` step hardcoded:

```rust
vec!["\"all_required_available\": true".into()]
```

And the `launch-smoke` base frame hardcoded the scenario id and evidence
opener, with conditional `failure_category` logic per scenario kind:

```rust
let mut expected = vec![format!("\"scenario_id\": \"{}\"", scenario.id)];
if scenario.kind == "alice_real_ui_action_contract" {
    expected.push("\"failure_category\":".into());
} else {
    expected.push("\"failure_category\": null".into());
}
expected.push("\"real_alice_execution_evidence\": {".into());
```

These patterns were duplicated in every generated adapter — 25+ copies of
identical strings. When the expected evidence format changed, every hardcoded
site had to change in lockstep.

Step blocks move these patterns into YAML templates that the generator reads
once and substitutes per scenario. The generated output is byte-identical; only
the source of truth has moved from Rust string literals to shared YAML files.

## Directory layout

Step-block templates live beside the generated gadugi adapters, in a dedicated
`step-blocks` subdirectory:

```text
assets/scenarios/gadugi/
├── step-blocks/
│   ├── alice-preflight.yaml
│   └── alice-launch-smoke.yaml
├── building-a-scene-first-world.yaml
├── real-alice-launch-smoke.yaml
└── ...
```

The `step-blocks/` directory is excluded from scenario asset discovery. Files
inside it are not validated as scenario assets and do not contribute to
`scenario_asset_count`.

## Template format

Each step-block template is a YAML list of step definitions. A step definition
captures the expected-stdout patterns for a single gadugi adapter step.

### alice-preflight.yaml

```yaml
# Shared preflight step-block for gadugi adapter generation.
# Used by: crates/eatme-assets/src/gadugi.rs (include_str!)
- id: validate-assets
  expect_stdout:
    - '"passed": true'
    - '"scenario_asset_count": {{scenario-asset-count}}'
  timeout_ms: 60000

- id: check-dependencies
  expect_stdout:
    - '"all_required_available": true'
  timeout_ms: 60000
```

### alice-launch-smoke.yaml

```yaml
# Shared launch-smoke base frame for gadugi adapter generation.
# Used by: crates/eatme-assets/src/gadugi.rs (include_str!)
- id: launch-smoke
  expect_stdout:
    - '"scenario_id": "{{scenario-id}}"'
    - '"real_alice_execution_evidence": {'
  timeout_ms: 900000
```

Templates define only the **expected-output patterns** and timeout. The actual
command, agent, step name, and assertion shape are still computed by the
generator from the canonical eatme scenario data.

## Placeholder substitution

Templates use `{{placeholder}}` syntax. The generator substitutes these at
generation time with per-scenario values:

| Placeholder | Source | Example value |
| --- | --- | --- |
| `{{scenario-asset-count}}` | Discovered scenario asset inventory count | `93` |
| `{{scenario-id}}` | `EatmeScenarioAsset.id` | `building-a-scene-first-world` |

Substitution uses `str::replace()` — no format-string injection, no runtime
file I/O. The template content is embedded at compile time via `include_str!()`.

## Discovery exclusion

The `collect_yaml_paths()` function in `discovery.rs` must skip directories
named `step-blocks` during recursive scenario asset discovery. The current
implementation recurses into all subdirectories; step-block composition adds a
guard that skips `step-blocks/`:

```rust
fn collect_yaml_paths(root: &Path, paths: &mut Vec<PathBuf>) -> Result<()> {
    // ...
    for entry in fs::read_dir(root)? {
        let path = entry?.path();
        if path.is_dir() {
            if path.file_name().and_then(|n| n.to_str()) == Some("step-blocks") {
                continue;  // step-block templates are generator inputs, not scenario assets
            }
            collect_yaml_paths(&path, paths)?;
        }
        // ...
    }
}
```

This keeps the `scenario_asset_count` stable. Step-block templates are
generator inputs, not scenario assets.

For test environments that use `scratch_root()` without a `step-blocks/`
directory, the exclusion is a safe no-op — the directory simply does not exist
and nothing is skipped.

## API reference

### Rust internals

The generator in `crates/eatme-assets/src/gadugi.rs` exposes no new public API.
Step-block consumption is internal:

| Item | Purpose |
| --- | --- |
| `ALICE_PREFLIGHT_TEMPLATE` | `include_str!()` constant embedding `alice-preflight.yaml` |
| `ALICE_LAUNCH_SMOKE_TEMPLATE` | `include_str!()` constant embedding `alice-launch-smoke.yaml` |
| `StepBlockEntry` | Deserialization struct: `id`, `expect_stdout: Vec<String>`, `timeout_ms: u64` |
| `parse_step_blocks()` | Parses a step-block YAML string into `Vec<StepBlockEntry>` |

The `timeout_ms` field in templates is informational for documentation and
future use. The generator continues to compute step timeouts via
`step_timeout_ms()` to preserve byte-identical output with the pre-template
generator. Template timeouts may replace computed timeouts in a future change
once the byte-equality baseline is re-established.

The `expected_stdout()` function looks up the step id in the parsed preflight
block entries. If a match is found, it returns the template's `expect_stdout`
patterns with placeholders substituted. If no match is found (e.g., for
`discover-alice`), it falls through to the existing hardcoded logic.

The `validate-assets` template entry covers the common case where the step
command runs full-inventory validation (no `--path` flag). Steps whose command
includes `--path` (e.g., `modified-class-portability`) bypass the template and
continue to use the existing command-content dispatch that returns the scenario
id instead of the asset count.

The `launch_expected_stdout()` function uses the launch-smoke template for the
base frame (`scenario_id` and `real_alice_execution_evidence`) and then appends
scenario-specific evidence entries (screenshots, UI actions, `africa.a3p`,
`passed`) using the existing dynamic logic.

The `failure_category` line is not in the template because its value differs
by scenario kind (`null` for lesson smoke, bare `"failure_category":` for
`alice_real_ui_action_contract`). It remains computed by the generator.

> **Design spec deviation**: the original spec listed `failure_category` as
> part of the launch-smoke template. Excluding it is intentional — including a
> conditional value in a static template would add branching logic that defeats
> the simplicity goal.

### Generated output

Step blocks do not change the generated gadugi YAML format. The output is
byte-identical to the pre-step-block generator. Existing byte-equality tests
in `gadugi_tests.rs` enforce this invariant.

## Configuration

Step blocks require no runtime configuration. Templates are compile-time
embedded and have no environment variable, CLI flag, or config file
dependencies.

The only "configuration" is the template file content itself, which lives in
version-controlled YAML under `assets/scenarios/gadugi/step-blocks/`.

## Examples

### How the generator uses alice-preflight.yaml

When generating the `validate-assets` step for `building-a-scene-first-world`
with 93 discovered assets:

1. Generator parses `alice-preflight.yaml` (once, at first use).
2. Finds the entry with `id: validate-assets`.
3. Takes `expect_stdout`: `['"passed": true', '"scenario_asset_count": {{scenario-asset-count}}']`.
4. Substitutes `{{scenario-asset-count}}` → `93`.
5. Returns `['"passed": true', '"scenario_asset_count": 93']`.

The generated adapter step is identical to what the hardcoded logic produced:

```yaml
expect:
  exit_code: 0
  stdout_contains:
    - '"passed": true'
    - '"scenario_asset_count": 93'
```

### How the generator uses alice-launch-smoke.yaml

When generating the `launch-smoke` step for `building-a-scene-first-world`:

1. Generator parses `alice-launch-smoke.yaml` (once, at first use).
2. Finds the entry with `id: launch-smoke`.
3. Takes `expect_stdout`: `['"scenario_id": "{{scenario-id}}"', '"real_alice_execution_evidence": {']`.
4. Substitutes `{{scenario-id}}` → `building-a-scene-first-world`.
5. Inserts `"failure_category": null` (computed, not from template — for
   `alice_real_ui_action_contract` scenarios this is instead bare
   `"failure_category":` without a value).
6. Appends scenario-specific evidence entries from existing dynamic logic
   (screenshots, UI actions, `africa.a3p`, `passed`).

### Instructor agentic flows are unaffected

Step blocks apply only to CLI-backed gadugi adapters (`gadugi.rs`). Instructor
agentic adapters (`gadugi_instructor.rs`) use a different code path with
`--path`-based validation and instructor-specific step patterns. They are
completely out of scope for step-block composition.

## Adding a new step block

1. Create a new YAML file in `assets/scenarios/gadugi/step-blocks/`:

   ```yaml
   # Description of what this step block captures.
   # Used by: crates/eatme-assets/src/gadugi.rs (include_str!)
   - id: my-new-step
     expect_stdout:
       - '"expected_field": "{{placeholder}}"'
     timeout_ms: 60000
   ```

2. Add an `include_str!()` constant in `gadugi.rs`:

   ```rust
   const MY_NEW_STEP_TEMPLATE: &str =
       include_str!("../../../assets/scenarios/gadugi/step-blocks/my-new-step.yaml");
   ```

3. Parse the template in the step-block helper and wire it into the
   appropriate `expected_stdout()` or step-generation logic.

4. Run `cargo test -p eatme-assets` to verify byte-identical output.

5. Run `cargo clippy --all-targets` to verify no warnings.

The step-block file is automatically excluded from scenario asset discovery
because it lives inside `step-blocks/`. No changes to `discovery.rs` are
needed when adding new templates to that directory.

## Editing policy

Step-block templates are generator inputs, not generated outputs. Edit them
directly when the expected-output contract changes.

After editing a step-block template:

1. Regenerate gadugi adapters:

   ```bash
   cargo run -q -p eatme-cli -- assets generate-gadugi --json
   ```

2. Validate the full inventory:

   ```bash
   cargo run -q -p eatme-cli -- assets validate --json
   ```

3. Run tests to confirm byte-equality:

   ```bash
   cargo test -p eatme-assets
   ```

4. Commit the template change and regenerated adapters together.

Do not add runtime-only placeholders or conditional logic to templates.
Templates are static expected-output patterns with simple string substitution.
Keep them flat and predictable.

## Related documentation

- [Gadugi Adapters](gadugi-adapters.md) — adapter boundary, workflow, and
  regeneration instructions
- [Scenario Authoring](scenario-authoring.md) — canonical scenario format,
  validation workflow, and evidence language
- [Generated Asset Consistency](generated-asset-consistency.md) —
  `scenario_asset_count` source of truth and the discovery exclusion that keeps
  step-block templates out of the count
