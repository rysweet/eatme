# Generated asset consistency

Generated asset consistency keeps canonical eatme scenario assets, generated
Gadugi adapters, and validation reports aligned.

The feature has one rule: counts and adapter expectations are derived from the
scenario files that are actually present under `assets/scenarios/`. They are not
manually chosen values.

## Contents

- [Source of truth](#source-of-truth)
- [Usage](#usage)
- [CLI reference](#cli-reference)
- [Configuration](#configuration)
- [Examples](#examples)
- [Authoring tutorial](#authoring-tutorial)
- [Strict validation behavior](#strict-validation-behavior)
- [Real UI action contract](#real-ui-action-contract)

## Source of truth

Canonical eatme scenarios live in:

```text
assets/scenarios/eatme/
```

Gadugi scenario assets live in:

```text
assets/scenarios/gadugi/
```

Most Gadugi scenario assets are generated adapters whose matching source files
live under `assets/scenarios/eatme/`. A small number may be hand-authored
regression assets, such as validation CLI contract tests. Hand-authored Gadugi
regression assets are still scenario assets and still contribute to validation
counts, but they are not generated adapter targets.

The `scenario_asset_count` value is derived by discovering every `.yaml` and
`.yml` scenario file under:

```text
assets/scenarios/
```

Discovery is recursive and deterministic. Directories named `step-blocks` are
excluded — files inside them are generator inputs, not scenario assets. The
count includes every scenario asset validated by eatme: canonical eatme
scenarios, generated Gadugi adapters, and any hand-authored Gadugi regression
scenarios. See [Step Block Composition](step-block-composition.md) for the
template format and discovery exclusion details.

The current committed inventory has 115 scenario YAML files:

| Scenario asset type | Count |
| --- | --- |
| Canonical eatme scenarios | 57 |
| Generated Gadugi adapters | 57 |
| Hand-authored Gadugi regression scenarios | 1 |

CLI-backed generated adapters use that discovered count in their validation
expectations:

```yaml
expect:
  exit_code: 0
  stdout_contains:
    - '"passed": true'
    - '"scenario_asset_count": 115'
```

Instructor generated adapters run `assets validate --path <scenario> --json` so
their id check comes from the single source scenario, not from the full inventory
report.

When scenario assets are added, removed, or renamed, the generated adapters must
be regenerated so committed expectations match the discovered inventory. For
removals and renames, remove the obsolete generated adapter file as part of the
same change. The generator compares and rewrites expected adapter targets; it
does not prune or report orphaned Gadugi files that no longer have a canonical
source.

## Usage

Validate all persona and scenario assets:

```bash
cargo run -q -p eatme-cli -- assets validate --json
```

Check that committed Gadugi adapters match generator output:

```bash
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
```

Regenerate adapters after changing canonical scenarios:

```bash
cargo run -q -p eatme-cli -- assets generate-gadugi --json
```

Run both checks before opening or merging a change that edits files under
`assets/scenarios/`:

```bash
cargo run -q -p eatme-cli -- assets validate --json
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
```

## CLI reference

### `assets validate --json`

Validates the full committed asset inventory from the repository root.

```bash
cargo run -q -p eatme-cli -- assets validate --json
```

Output fields:

| Field | Meaning |
| --- | --- |
| `schema_version` | Validation report schema: `eatme.assets/validation/v1` |
| `asset_path` | Root path that was validated |
| `passed` | `true` only when no validation errors were found |
| `instructor_count` | Instructor personas discovered from the persona crew asset |
| `student_count` | Student personas discovered from the persona crew asset |
| `core_scenario_count` | Core persona-crew scenarios discovered from the persona asset |
| `creative_scenario_count` | Creative persona-crew scenarios discovered from the persona asset |
| `scenario_asset_count` | Discovered `.yaml` and `.yml` scenario assets under `assets/scenarios/` |
| `errors` | Validation errors that must be fixed before the asset set is trusted |
| `warnings` | Non-blocking diagnostics |

Exit behavior:

| Result | Exit code | Output contract |
| --- | --- | --- |
| Valid inventory | `0` | JSON report with `passed: true` |
| Semantic validation failure | Non-zero | JSON report with `passed: false` and populated `errors` |
| Malformed YAML or unknown schema field | Non-zero | Failure is surfaced with parse context; the invalid file is not accepted |

### `assets validate --path <asset> --json`

Validates one persona crew or scenario asset.

```bash
cargo run -q -p eatme-cli -- assets validate \
  --path assets/scenarios/eatme/building-a-scene-first-world.yaml \
  --json
```

Scenario output fields:

| Field | Meaning |
| --- | --- |
| `schema_version` | Scenario validation report schema: `eatme.assets/scenario-validation/v1` |
| `asset_path` | Scenario file that was validated |
| `asset_kind` | `eatme` for canonical scenarios, `gadugi` for Gadugi scenario assets |
| `id` | Scenario id or generated Gadugi scenario name |
| `passed` | `true` only when the scenario has no validation errors |
| `step_count` | Number of executable scenario steps |
| `assertion_count` | Number of acceptance criteria or Gadugi assertions |
| `errors` | Blocking scenario validation errors |
| `warnings` | Non-blocking diagnostics |

### `assets generate-gadugi --check --json`

Checks whether committed Gadugi adapters are fresh without writing files.

```bash
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
```

Output fields:

| Field | Meaning |
| --- | --- |
| `schema_version` | Generation report schema: `eatme.assets/gadugi-adapter-generation/v1` |
| `root` | Repository root used for discovery |
| `generated_count` | Number of adapter targets the generator would produce from canonical eatme scenarios |
| `checked_count` | Number of generated adapter targets compared in check mode |
| `changed` | Expected adapter target paths that are stale, missing, or would be rewritten |
| `passed` | `true` only when committed adapters exactly match generated output |
| `errors` | Blocking freshness errors |

Exit behavior:

| Result | Exit code | Output contract |
| --- | --- | --- |
| Expected adapter targets are fresh | `0` | JSON report with `passed: true` and empty `changed` |
| Expected adapter target is stale or missing | Non-zero | JSON report with `passed: false`, changed path, and regeneration guidance |

Check mode does not validate extra Gadugi YAML files that are not expected
generated targets. Run `assets validate --json` to validate the full scenario
inventory, and manually delete obsolete generated adapters when a canonical
scenario is removed or renamed.

### `assets generate-gadugi --json`

Regenerates Gadugi adapters in place from canonical eatme scenarios.

```bash
cargo run -q -p eatme-cli -- assets generate-gadugi --json
```

The command writes expected generated adapter targets under
`assets/scenarios/gadugi/`. It does not rewrite canonical eatme scenarios, and
it does not delete obsolete generated adapters.

## Configuration

### Repository root

Run commands from the repository root when possible. Use `--root` for
`generate-gadugi` when running from another directory:

```bash
cargo run -q -p eatme-cli -- assets generate-gadugi \
  --root /path/to/eatme \
  --check \
  --json
```

Generated Gadugi adapters support `EATME_REPO` at runtime:

```bash
EATME_REPO=/path/to/eatme gadugi run assets/scenarios/gadugi/building-a-scene-first-world.yaml
```

When `EATME_REPO` is not set, generated adapter commands use the current
directory.

### Runtime environment

Generated adapters declare runtime variables in their `environment` block.
Common variables are:

| Variable | Required when | Purpose |
| --- | --- | --- |
| `ALICE_HOME` | Adapter launches or checks real Alice | Points to the Alice checkout |
| `EATME_REAL_ALICE=1` | Non-baseline real Alice scenarios | Prevents mocked or accidental lesson-labeled launches |
| `RUN_ID` | Optional for generated adapters | Overrides the default generated run id |
| `EATME_REPO` | Optional for generated adapters | Runs eatme commands from a specific repository root |
| `NODE_OPTIONS=--max-old-space-size=32768` | Repository quality workflows that invoke Node-based tooling | Gives Node subprocesses the expected memory ceiling |

The Rust asset validation and generator commands do not require Node. Keeping
`NODE_OPTIONS` exported is safe for repository-wide quality workflows.

## Examples

### Valid generated count

For the current 115-file inventory, validation output
includes:

```json
{
  "schema_version": "eatme.assets/validation/v1",
  "passed": true,
  "scenario_asset_count": 115,
  "errors": []
}
```

CLI-backed generated Gadugi adapters for that same inventory expect the same
count:

```yaml
stdout_contains:
  - '"passed": true'
  - '"scenario_asset_count": 115'
```

### Stale adapter check

If the scenario inventory changes and adapters are not regenerated, check mode
reports expected generated adapter targets whose committed expectations no
longer match. When a new canonical scenario has no generated adapter yet, check
mode reports the new target as missing:

```json
{
  "schema_version": "eatme.assets/gadugi-adapter-generation/v1",
  "passed": false,
  "changed": [
    "assets/scenarios/gadugi/building-a-scene-first-world.yaml",
    "assets/scenarios/gadugi/new-scenario.yaml"
  ],
  "errors": [
    "assets/scenarios/gadugi/building-a-scene-first-world.yaml is stale; regenerate with `eatme assets generate-gadugi`",
    "assets/scenarios/gadugi/new-scenario.yaml is missing or unreadable: ..."
  ]
}
```

The fix is to regenerate and re-run check mode:

```bash
cargo run -q -p eatme-cli -- assets generate-gadugi --json
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
```

## Authoring tutorial

Use this workflow when adding, removing, or renaming a canonical scenario.

1. Edit or add the canonical scenario under `assets/scenarios/eatme/`.
2. Validate the single scenario:

   ```bash
   cargo run -q -p eatme-cli -- assets validate \
     --path assets/scenarios/eatme/building-a-scene-first-world.yaml \
     --json
   ```

3. Regenerate Gadugi adapters:

   ```bash
   cargo run -q -p eatme-cli -- assets generate-gadugi --json
   ```

4. Validate the full inventory:

   ```bash
   cargo run -q -p eatme-cli -- assets validate --json
   ```

5. Confirm generated output is reproducible:

   ```bash
   cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
   ```

6. Commit the canonical scenario change and the generated adapter change
   together.

For removals and renames, also delete the obsolete generated adapter under
`assets/scenarios/gadugi/`. `generate-gadugi` does not delete files that no
longer have a canonical source, and check mode compares only the adapter targets
that should be generated from the remaining canonical scenarios.

Do not hand-edit `scenario_asset_count` in generated adapters. A hand edit may
make one file pass locally while leaving the generator and committed output out
of sync.

## Strict validation behavior

Scenario and persona YAML use strict schemas. Unknown top-level fields and
unknown nested fields are rejected instead of ignored. This applies to canonical
eatme scenarios, Gadugi scenario assets, and persona crew assets.

This fails because `unknown_field` is not part of `eatme.scenario/v1`:

```yaml
schema_version: eatme.scenario/v1
id: strict-test
title: Strict Test
purpose: Catch bad edits.
unknown_field: should-fail
```

This also fails because `href` is not a valid `resource_basis` field:

```yaml
schema_version: eatme.scenario/v1
id: strict-test
title: Strict Test
purpose: Catch bad edits.
resource_basis:
  - name: Resource
    href: https://example.invalid
```

Missing required semantic fields also fail validation. For example, lesson smoke
and real UI action scenarios must define non-empty capability, adapter,
launcher, evidence, timeout, artifact, and persona references appropriate for
their kind.

## Real UI action contract

`first-lessons-real-ui-actions` is an explicit real Alice UI action contract.
Its generated adapter is not a passing UI automation run. The adapter expects
eatme to launch real Alice, collect deterministic evidence, and fail loudly with
a UI action failure category such as:

```json
{
  "scenario_id": "first-lessons-real-ui-actions",
  "failure_category": "ui_action_automation_unimplemented"
}
```

When the Alice-side object placement hook proves placement, the category can
advance to `ui_action_remaining_steps_unimplemented`; that still is not a full
UI automation pass.

The generated Gadugi adapter preserves that contract with `expect.exit_code: 1`
for the launch step and required output markers for:

```text
real_alice_execution_evidence
specific_alice_window_detected
place_object_ui_action
edit_procedure_ui_action
run_world_ui_action
save_project_ui_action
ui_action_artifact_captured
ui_action_contract
```

Do not change the generated adapter to expect a successful launch until the
eatme harness actually implements object placement, procedure editing, world
running, project saving, and UI action artifact capture.
