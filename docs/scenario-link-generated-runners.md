# Scenario-link generated runners

Scenario-link generated runners connect editable eatme scenarios to generated
Gadugi scenario files without broadening what the executable evidence checks.

Use this guide when you author scenario links, regenerate Gadugi runners, or
review the first-lesson silver thread from prerequisites to bounded evidence to
the next classroom action.

## Contents

- [What the feature provides](#what-the-feature-provides)
- [Scenario-link model](#scenario-link-model)
- [Quick start](#quick-start)
- [Usage](#usage)
- [CLI reference](#cli-reference)
- [Data contract reference](#data-contract-reference)
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

## Scenario-link model

A scenario link is the traceable path from a canonical scenario to a generated
runner and then to the next human review or bounded generated-runner step. It is
not a new runtime layer. The editable scenario supplies intent and boundaries;
the generated runner supplies reproducible execution steps; documentation
explains how to interpret the evidence without adding claims.

Use these surfaces together:

| Surface | Location | Reader contract |
| --- | --- | --- |
| Canonical scenario | `assets/scenarios/eatme/<scenario-id>.yaml` | The editable source for prerequisites, audience, evidence, artifacts, unsupported behavior, and follow-on path. |
| Generated runner | `assets/scenarios/gadugi/<scenario-id>.yaml` | Deterministic output that calls eatme commands and checks emitted markers. |
| Evidence artifacts | `runs/<scenario-id>/<RUN_ID>/` | Manifest, log, window list, screenshot, and scenario-specific artifacts produced by the eatme command. |
| Documentation | `docs/` | Human-readable guidance for authoring, running, reviewing, and interpreting the evidence. |

The first-lesson path uses this chain:

```text
first-lessons-real-ui-actions
  -> generated Gadugi runner
  -> launch-smoke evidence and ui-action-contract.json
  -> lesson readiness interpretation
  -> instructor-student-launch-evidence-handoff
```

The chain keeps the same boundary at every step: setup, launch, handoff, and
classroom-support readiness evidence only. It does not become full UI
automation, visible rendering correctness, grading, creative assessment,
complete lesson execution, or broad Alice compatibility.

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
| "routes to instructor handoff, human review, or a bounded generated-runner step" | "grades learner worlds" |
| "names creative work for review" | "performs creative assessment" |

### Connect first-lesson evidence to the classroom handoff

Use `first-lessons-real-ui-actions` for the executable first-lesson readiness
evidence and `instructor-student-launch-evidence-handoff` for the editable
instructor/student follow-on prompt.

The executable scenario produces evidence such as:

- the selected scenario id in the run summary
- launch manifest, log, window list, and startup screenshot references
- Alice window detection and safe activation probes
- save and Run shortcut dispatch probes when their preconditions are met
- `ui-action-contract.json` with explicit action expectations and blockers

The instructor handoff scenario turns those evidence inputs into classroom
materials:

- `real_alice_evidence_handoff_card`
- `instructor_readiness_note`
- `student_action_prompt`

The handoff scenario stays at the agentic acceptance boundary. It can ask a
student to record one Alice action, the visible result, and one revision. It
does not certify that eatme automated the action or graded the result.

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

## Data contract reference

### Generator-rendered canonical fields

Scenario-link generated runners render these canonical eatme scenario fields into
Gadugi YAML:

| Field | Purpose |
| --- | --- |
| `id` | Stable scenario id. The filename, generated runner name, metadata tags, command arguments, and default run id are derived from it. |
| `title` | Human-readable scenario name used in generated runner names. |
| `kind` | Selects the runner shape and metadata test type, such as `alice_real_ui_action_contract`, `alice_lesson_smoke`, or `instructor_agentic_flow`. |
| `purpose` and `unsupported_policy` | Feed generated boundary wording when the scenario declares the source limits that the generator recognizes. |
| `real_alice.gated_by` | Adds required real-Alice environment gates such as `EATME_REAL_ALICE`. |
| `steps[].command` | Provides the eatme command the generated runner executes. |
| `steps[].evidence` | Supplies launch-output markers that the generated runner checks for bounded evidence scenarios. |
| `agentic_flow.focus` and `agentic_flow.expected_outputs` | Feed instructor-agentic tags and expected output markers. |
| `agentic_test_prompt` and `acceptance_probes` | Feed instructor-agentic runner prompts and probe text. |
| `timeouts` | Supplies scenario, launch, and agentic timeout values used by generated runners. |

### Validated canonical fields

These canonical fields are part of the editable source contract and validation
surface. They are not necessarily copied into generated Gadugi YAML:

| Field | Purpose |
| --- | --- |
| `schema_version` | Identifies the editable scenario schema. Scenario-link runners consume `eatme.scenario/v1`. |
| `resource_basis` | Names external or local resources the scenario is grounded in. |
| `capabilities.required` and `capabilities.optional` | Lists host tools and optional probes needed before execution. |
| `smoke_ready.evidence` | Names the evidence markers the scenario expects from a launch or readiness path. |
| `acceptance_criteria` | Defines observable Given/When/Then checks and explicit unsupported-action outcomes. |
| `agentic_follow_on` | Routes follow-on instructor/student work without converting it into launch evidence. |
| `artifacts` | Names manifest, screenshot, log, window-list, and scenario-specific artifact paths. |
| `unsupported_policy` | Preserves fail-loud behavior and the exact claims the scenario does not make. |

### Generated runner fields

Generated Gadugi runner files expose this stable contract to external runners:

| Field | Purpose |
| --- | --- |
| `name` | Human-readable generated runner name. |
| `description` | Generated boundary statement that names the source scenario and evidence scope. |
| `config.timeout` | Scenario-level timeout in milliseconds, derived from canonical timeouts. |
| `environment.requires` | Required environment variables, such as `ALICE_HOME` and `EATME_REAL_ALICE` for real Alice paths. |
| `environment.optional` | Optional variables. CLI launch runners expose `RUN_ID` and `EATME_REPO`; instructor-agentic runners expose `EATME_REPO`. |
| `agents` | System or agentic runner definitions used by Gadugi. |
| `steps[].params.command` | Shell command that changes to `EATME_REPO` when supplied, may set a default `RUN_ID`, and invokes eatme. |
| `steps[].expect` | Exit-code and output-marker expectations for each generated step. |
| `assertions` | Named pass/fail checks that bind to generated steps. |
| `metadata.source_eatme_asset` | Relative path back to the canonical eatme scenario. |
| `metadata.generated_by` | Generator identity. |
| `metadata.tags` | Searchable scenario and feature tags, including the source scenario id. |
| `metadata.test_type` | Runner category, such as `ui-action-contract` or `instructor-agentic-flow`. |

Generated runners are a compatibility API for Gadugi consumers. Change the
canonical scenario or generator policy when this contract needs new wording,
commands, or checks.

### Shell quoting safety

Generated shell commands quote every shell-expanded launch argument. The
generator replaces unquoted `--alice-home ${ALICE_HOME}` with
`--alice-home "${ALICE_HOME}"` and unquoted `--run-id ${RUN_ID}` with
`--run-id "${RUN_ID}"`. This prevents word-splitting and globbing when paths
contain spaces or special characters.

The quoting contract is enforced by the
`generated_launch_commands_quote_environment_argument_expansions` regression
test, which asserts that every generated launch command includes the quoted form
and rejects the unquoted form.

Generated CLI launch runners that export `RUN_ID` with a default value also
declare `RUN_ID` under `environment.optional`. The
`generated_runners_declare_run_id_optional_when_commands_export_it` test
enforces this contract across all generated runners that contain
`export RUN_ID=`.

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

### Produce an instructor handoff from first-lesson evidence

After a first-lesson run produces manifest, log, window-list, screenshot, and
`ui-action-contract.json` evidence, use the instructor handoff scenario as the
next classroom-support step:

```bash
cargo run -q -p eatme-cli -- assets validate \
  --path assets/scenarios/eatme/instructor-student-launch-evidence-handoff.yaml \
  --json
```

The generated Gadugi runner for the handoff validates editable assets and then
asks the instructor acceptance agent for:

```text
real_alice_evidence_handoff_card
instructor_readiness_note
student_action_prompt
```

Use those outputs to separate environment readiness from student project
behavior. Keep launch evidence, action-contract blockers, and classroom
observation in separate sections so students are not shown a false completion
claim.

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
- Generated shell commands quote `${ALICE_HOME}` and `${RUN_ID}`.
- Generated runners that export `RUN_ID` declare it under `environment.optional`.
- First-lesson wording says readiness evidence, not lesson completion.
- UI action gaps fail loudly instead of silently passing.
- Docs and generated runners avoid claims about full UI automation, rendering
  correctness, grading, creative assessment, Save completion, lesson completion,
  or broad Alice compatibility unless an executable check covers that exact
  claim.
