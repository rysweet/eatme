# Scenario-link generated runners

Scenario-link generated runners connect editable eatme scenarios to generated
Gadugi scenario files without broadening what the executable evidence checks.

Use this guide when you author scenario links, regenerate Gadugi runners, or
review the first-lesson silver thread from prerequisites to bounded evidence to
the next classroom action.

## Contents

- [What the feature provides](#what-the-feature-provides)
- [Quick start](#quick-start)
- [Usage](#usage)
- [CLI reference](#cli-reference)
- [Generated YAML contract](#generated-yaml-contract)
- [Configuration](#configuration)
- [Examples](#examples)
- [Authoring tutorial](#authoring-tutorial)
- [Review checklist](#review-checklist)

## What the feature provides

The scenario-link generated-runner path has one source of truth:

```text
assets/scenarios/eatme/
```

Generated Gadugi runners are derived output:

```text
assets/scenarios/gadugi/
```

The generated runner preserves the canonical scenario's link path:

| Link stage | Source field or file | Generated-runner responsibility |
| --- | --- | --- |
| Prerequisites | `capabilities`, `real_alice.gated_by`, `timeouts`, and step commands | Name required tools, gates, and scenario inputs before execution. |
| Evidence | `purpose`, `smoke_ready`, `acceptance_criteria`, step `evidence`, and `unsupported_policy` | Run eatme commands and check the emitted JSON, manifest, and artifact markers named by the scenario. |
| Boundary | `unsupported_policy` and bounded acceptance criteria | Preserve explicit non-claims instead of converting a readiness path into a completion claim. |
| Follow-on path | `agentic_follow_on`, handoff scenarios, and docs links | Route the reader to the next bounded action or human review step. |

Eatme owns Alice desktop launch behavior, dependency checks, virtual display
setup, Java process lifecycle, screenshots, logs, manifests, and validation
reports. Gadugi runners invoke eatme commands and inspect their output.

## Quick start

Run from the repository root:

```bash
cargo run -q -p eatme-cli -- assets validate --json
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
```

If generated runners are stale after editing canonical scenarios, regenerate and
check again:

```bash
cargo run -q -p eatme-cli -- assets generate-gadugi --json
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
```

## Usage

### Author scenario links

Write scenario links in canonical eatme YAML, not in generated Gadugi YAML.

Use canonical fields for the reader path:

```yaml
schema_version: eatme.scenario/v1
id: first-lessons-real-ui-actions
purpose: >-
  Record first-lesson readiness evidence for the real Alice path while keeping
  unsupported UI action boundaries explicit.
real_alice:
  gated_by: EATME_REAL_ALICE=1
steps:
  - id: launch-first-lesson-readiness
    command: >-
      EATME_REAL_ALICE=1 cargo run -q -p eatme-cli -- alice launch-smoke
      --alice-home ${ALICE_HOME}
      --scenario first-lessons-real-ui-actions
      --json
    evidence:
      - manifest scenario_id equals first-lessons-real-ui-actions
      - readiness fields name checked and unchecked action boundaries
unsupported_policy: >-
  This scenario records bounded first-lesson readiness evidence. It is not full
  UI automation, not creative assessment, and not learner-world grading.
```

Then regenerate Gadugi runners from that canonical source.

### Review generated runners

Review generated Gadugi YAML as derivative output. Do not hand-edit generated
descriptions, command expectations, or scenario counts to make one runner pass.

If a generated runner has the wrong prompt, evidence wording, command shape, or
boundary note, change the canonical eatme scenario or generator policy and run:

```bash
cargo run -q -p eatme-cli -- assets generate-gadugi --json
```

### Keep claims bounded

Scenario-link generated runners may claim only what their commands check.

| Do say | Do not say unless separately checked |
| --- | --- |
| "records first-lesson readiness evidence" | "completes the first lesson" |
| "checks manifest-level launch evidence" | "claims visible rendering correctness" |
| "keeps unsupported UI action boundaries explicit" | "fully automates the Alice UI" |
| "routes to instructor handoff or human review" | "grades learner worlds" |
| "names creative work for review" | "performs creative assessment" |

## CLI reference

### `assets validate --json`

Validates the committed persona and scenario asset inventory.

```bash
cargo run -q -p eatme-cli -- assets validate --json
```

Use this before trusting a generated runner, because check mode compares
generated targets but validation checks the full scenario inventory.

### `assets validate --path <asset> --json`

Validates one canonical scenario or one generated Gadugi scenario.

```bash
cargo run -q -p eatme-cli -- assets validate \
  --path assets/scenarios/eatme/first-lessons-real-ui-actions.yaml \
  --json
```

Use path validation while authoring a scenario, then run full validation before
review.

### `assets generate-gadugi --check --json`

Compares committed generated Gadugi runners with the output the generator would
produce from canonical eatme scenarios.

```bash
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
```

Check mode does not write files. It fails when an expected generated target is
missing or stale.

### `assets generate-gadugi --json`

Writes generated Gadugi runners under `assets/scenarios/gadugi/`.

```bash
cargo run -q -p eatme-cli -- assets generate-gadugi --json
```

The command does not edit canonical eatme scenarios and does not prune obsolete
Gadugi files after a canonical scenario is removed or renamed.

### `assets generate-gadugi --root <path> --check --json`

Runs the generator against a repository root that is not the current directory.

```bash
cargo run -q -p eatme-cli -- assets generate-gadugi \
  --root /path/to/eatme \
  --check \
  --json
```

## Generated YAML contract

Each generated Gadugi runner is deterministic output from a canonical eatme
scenario. Generated Gadugi YAML does not include a top-level `schema_version`;
traceability comes from generated metadata and the canonical eatme source file.
A generated file includes these top-level fields:

| Field | Contract |
| --- | --- |
| `name` | Uses a human-readable generated name derived from the canonical scenario title. |
| `description` | States the canonical source and the bounded evidence scope. |
| `version` | Identifies the generated runner version. |
| `config` | Sets timeout, retry, and parallel-execution policy from the canonical scenario. |
| `environment` | Names required and optional runtime variables. Real Alice runners require variables such as `ALICE_HOME` and `EATME_REAL_ALICE`; generated runners use `EATME_REPO` and, for CLI launch paths, `RUN_ID` when supplied. |
| `agents` | Defines the system or agentic runner that executes the generated steps. |
| `steps` | Invokes eatme commands instead of duplicating Alice desktop automation. |
| `expect` | Checks exit codes and output markers that belong to the bounded scenario contract. |
| `assertions` | Names the generated pass/fail checks for the runner's steps. |
| `metadata` | Records `source_eatme_asset`, `generated_by`, tags, priority, author, and test type. `metadata.source_eatme_asset` and `metadata.tags` trace the generated runner back to the canonical eatme scenario id and source file. |

Generated descriptions follow this shape:

```text
Gadugi-compatible CLI scenario generated from <source-scenario>. Alice desktop launch behavior remains owned by eatme; <bounded evidence scope>.<boundary note>
```

For `first-lessons-real-ui-actions`, the generated description identifies
first-lesson readiness evidence and keeps the explicit limits: not full UI
automation, not creative assessment, and not learner-world grading.

For `starter-project-open-save-export-preflight`, the generated description
identifies starter-world and readiness-gap artifacts without claiming
save/reopen/export coverage, visible rendering correctness, lesson completion,
or complete Alice coverage.

## Configuration

### Repository root

Run commands from the repository root when possible. Use `--root` for generator
checks from another directory.

Generated runners support `EATME_REPO` at runtime:

```bash
EATME_REPO=/path/to/eatme gadugi run assets/scenarios/gadugi/first-lessons-real-ui-actions.yaml
```

When `EATME_REPO` is not set, generated runner commands use the current
directory.

### Real Alice scenarios

Real Alice scenarios require `ALICE_HOME` and, for non-baseline lesson-labeled
runs, `EATME_REAL_ALICE=1`:

```bash
export ALICE_HOME=/path/to/alice
export EATME_REAL_ALICE=1
```

The real-Alice gate prevents a lesson-labeled scenario from silently substituting
a mocked or incomplete runtime.

### Run identifiers

Set `RUN_ID` when you need a stable artifact directory:

```bash
RUN_ID=local-first-lessons gadugi run assets/scenarios/gadugi/first-lessons-real-ui-actions.yaml
```

When `RUN_ID` is not set, generated commands use their default run-id behavior.

### Node memory for repository workflows

The Rust validation and generator commands do not require Node. Repository-wide
quality workflows may invoke Node-based tooling, so keep the repository
preference available when running those workflows:

```bash
export NODE_OPTIONS=--max-old-space-size=32768
```

## Examples

### Check the first-lesson generated runner

```bash
cargo run -q -p eatme-cli -- assets validate \
  --path assets/scenarios/gadugi/first-lessons-real-ui-actions.yaml \
  --json
```

This validates the generated runner as a scenario asset. It does not claim full
UI automation, rendering correctness, grading, creative assessment, or lesson
completion.

### Run the first-lesson readiness path through eatme

```bash
EATME_REAL_ALICE=1 cargo run -q -p eatme-cli -- alice launch-smoke \
  --alice-home "${ALICE_HOME}" \
  --scenario first-lessons-real-ui-actions \
  --run-id local-first-lessons-real-ui-actions \
  --json
```

Use the resulting JSON and artifacts as readiness evidence for the named
scenario. Treat explicit UI action failure categories as part of the bounded
contract until the corresponding user-like action evidence exists.

### Check generated runner freshness before review

```bash
cargo run -q -p eatme-cli -- assets validate --json
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
```

Both commands must pass before a scenario-link change is ready for review.

## Authoring tutorial

Use this workflow when adding or changing a scenario-link path.

1. Edit the canonical scenario under `assets/scenarios/eatme/`.
2. Put prerequisite, evidence, boundary, and follow-on wording in the canonical
   scenario fields.
3. Validate the edited scenario:

   ```bash
   cargo run -q -p eatme-cli -- assets validate \
     --path assets/scenarios/eatme/first-lessons-real-ui-actions.yaml \
     --json
   ```

4. Regenerate generated runners:

   ```bash
   cargo run -q -p eatme-cli -- assets generate-gadugi --json
   ```

5. Validate the full inventory:

   ```bash
   cargo run -q -p eatme-cli -- assets validate --json
   ```

6. Confirm generated output is reproducible:

   ```bash
   cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
   ```

7. Review generated diffs only as derivative output. If wording is wrong, return
   to the canonical scenario or generator policy instead of hand-editing the
   generated file.

## Review checklist

Before review, confirm:

- Scenario links are authored in `assets/scenarios/eatme/`.
- Generated Gadugi runners are fresh and reproducible.
- Generated descriptions name the source scenario and bounded evidence scope.
- First-lesson wording says readiness evidence, not lesson completion.
- UI action gaps fail loudly instead of silently passing.
- Docs and generated runners avoid claims about full UI automation, rendering
  correctness, grading, creative assessment, Save completion, lesson completion,
  or broad Alice compatibility unless an executable check covers that exact
  claim.
