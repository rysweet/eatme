# [PLANNED] Alice QA persona-to-scenario coverage

This document defines the planned `workshop-facilitator-live-studio` coverage
slice. It is a build contract for promoting an Alice persona-crew scenario marker
into a standalone editable scenario plus a generated Gadugi adapter.

The feature will connect workshop facilitator personas from
`assets/personas/alice-user-crew.yaml` to a canonical
`kind: instructor_agentic_flow` scenario under `assets/scenarios/eatme/`. The
scenario and generated adapter will give instructors and agents reviewable
workshop evidence without changing Rust code.

## Contents

- [Planned asset contract](#planned-asset-contract)
- [Build workflow](#build-workflow)
- [Scenario asset API](#scenario-asset-api)
- [Generated adapter API](#generated-adapter-api)
- [Configuration](#configuration)
- [Review workflow](#review-workflow)
- [Boundaries](#boundaries)

## Planned asset contract

The persona crew owns the workshop coverage marker. The feature is complete only
when the marker has a committed canonical scenario and a generated adapter:

| Asset | Implementation role | Purpose |
| --- | --- | --- |
| `assets/personas/alice-user-crew.yaml` | Source input | Defines the `workshop-facilitator-live-studio` marker and related persona ids. |
| `assets/scenarios/eatme/workshop-facilitator-live-studio.yaml` | Planned canonical asset | Editable eatme scenario for workshop facilitation coverage. |
| `assets/scenarios/gadugi/workshop-facilitator-live-studio.yaml` | Planned generated asset | Gadugi adapter generated from the canonical scenario. |

Do not treat the persona-crew marker by itself as a runnable standalone
scenario. The marker becomes reviewable scenario coverage only after the
canonical eatme scenario and generated Gadugi adapter are committed.

The canonical scenario will explicitly connect these personas:

| Role | Persona ids |
| --- | --- |
| Instructors | `workshop-facilitator`, `studio-facilitator` |
| Students | `creative-storyteller`, `collaborative-peer-mentor`, `reflective-debugger` |

The scenario covers a short live studio workshop where participants build one
small Alice artifact, test it, request help with a visible stuck signal, exchange
peer feedback, and share a tiny outcome before the session ends.

## Build workflow

Implement the coverage slice in this order:

1. Add the canonical scenario:

   ```text
   assets/scenarios/eatme/workshop-facilitator-live-studio.yaml
   ```

2. Validate the new scenario:

   ```bash
   cargo run -q -p eatme-cli -- assets validate \
     --path assets/scenarios/eatme/workshop-facilitator-live-studio.yaml \
     --json
   ```

3. Regenerate the Gadugi adapter:

   ```bash
   cargo run -q -p eatme-cli -- assets generate-gadugi --json
   ```

4. Check the generated adapter is fresh:

   ```bash
   cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
   ```

5. Validate the full persona and scenario inventory:

   ```bash
   cargo run -q -p eatme-cli -- assets validate --json
   ```

Definition of done:

| Requirement | Evidence |
| --- | --- |
| Canonical scenario exists | `assets/scenarios/eatme/workshop-facilitator-live-studio.yaml` is committed. |
| Generated adapter exists | `assets/scenarios/gadugi/workshop-facilitator-live-studio.yaml` is committed and generated from the canonical scenario. |
| Asset validation passes | `cargo run -q -p eatme-cli -- assets validate --json` reports success. |
| Adapter generation is clean | `cargo run -q -p eatme-cli -- assets generate-gadugi --check --json` reports no changed adapters. |
| Reviewable workshop evidence is encoded | Scenario includes `agentic_test_prompt`, `acceptance_probes`, `rubric`, `avoid`, `steps`, and `artifacts`. |

The planned scenario is an instructor agentic flow. It validates editable asset
coverage, prompts, probes, rubrics, and review outputs. It does not need a real
Alice desktop launch to run the agentic review.

## Scenario asset API

`workshop-facilitator-live-studio.yaml` will use the existing
`eatme.scenario/v1` schema.

| Field | Value or contract |
| --- | --- |
| `schema_version` | `eatme.scenario/v1` |
| `id` | `workshop-facilitator-live-studio` |
| `title` | `Workshop Facilitator Live Studio` |
| `kind` | `instructor_agentic_flow` |
| `owner` | `eatme` |
| `resource_basis` | Alice resource grounding for lessons, classroom use, and export/share work. |
| `purpose` | Describes editable workshop facilitation evidence without claiming desktop automation or automated assessment. |
| `personas.instructors` | `workshop-facilitator`, `studio-facilitator` |
| `personas.students` | `creative-storyteller`, `collaborative-peer-mentor`, `reflective-debugger` |
| `agentic_flow.focus` | Workshop facilitation with checkpoints, help signals, recovery moves, and share-out evidence. |
| `agentic_flow.prompt_source` | `assets/scenarios/eatme/workshop-facilitator-live-studio.yaml#agentic_test_prompt` |
| `agentic_flow.non_coder_editable` | `resource_basis`, `purpose`, `agentic_test_prompt`, `acceptance_criteria`, `acceptance_probes`, `rubric`, `avoid` |
| `agentic_flow.expected_outputs` | `workshop_plan`, `checkpoint_board`, `helper_roles`, `recovery_moves`, `showcase_notes` |
| `agentic_test_prompt` | Prompt for producing a short Alice workshop facilitation plan. |
| `acceptance_criteria` | Given/when/then checks for timeboxing, mixed readiness, participant progress, help signals, checkpoint decisions, and reviewable evidence. |
| `acceptance_probes` | Plain-language checks applied to the agent output. |
| `rubric` | Criteria for workshop readiness, participant progress evidence, help/recovery behavior, and share-out quality. |
| `avoid` | Brittle or overclaimed outputs the scenario rejects. |
| `steps` | Asset validation plus instructor agentic review. |
| `timeouts.agentic_seconds` | Agentic review timeout, matching existing instructor flow conventions. |
| `artifacts` | Logical `agentic://` output names for instructor-facing artifacts. |
| `unsupported_policy` | Fails visibly when required editable evidence is unavailable. |

Required agentic outputs are named so a reviewer can inspect them without
opening Rust code:

```text
workshop_plan
checkpoint_board
helper_roles
recovery_moves
showcase_notes
```

The scenario keeps evidence observable. Good outputs name timeboxed milestones,
minimum runnable artifact expectations, helper roles, stuck signals, checkpoint
decisions, peer feedback, and final share criteria.

## Generated adapter API

The generated Gadugi adapter will live at:

```text
assets/scenarios/gadugi/workshop-facilitator-live-studio.yaml
```

It must be generated from the canonical eatme scenario. Do not hand-edit it to
change mission intent.

The adapter contract is:

| Adapter section | Contract |
| --- | --- |
| `agents.eatme-cli-agent` | Runs repository CLI validation. |
| `agents.instructor-qa-agent` | Runs the instructor agentic review against the canonical scenario. |
| `steps.Validate editable Alice instructor assets` | Expects `assets validate --json` to pass and include `workshop-facilitator-live-studio`. |
| `steps.Run instructor agentic QA review` | Supplies the scenario prompt and acceptance probes to the agentic reviewer. |
| `assertions.Assets Validate` | Fails if editable assets do not validate. |
| `assertions.Instructor Agentic Review Covers Probes` | Fails if the agentic review does not satisfy the scenario probes. |
| `metadata.source_eatme_asset` | Points back to `assets/scenarios/eatme/workshop-facilitator-live-studio.yaml`. |

Regenerate the adapter after editing the canonical scenario:

```bash
cargo run -q -p eatme-cli -- assets generate-gadugi --json
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
```

## Configuration

Run asset commands from the repository root when possible.

Set `NODE_OPTIONS` for repository-wide quality workflows that invoke Node-based
tooling:

```bash
export NODE_OPTIONS=--max-old-space-size=32768
```

The Rust asset validation and Gadugi generation commands do not require Node,
but keeping the variable exported is safe for the full quality workflow.

Generated Gadugi adapters also support:

| Variable | Required | Purpose |
| --- | --- | --- |
| `EATME_REPO` | Optional | Runs eatme CLI commands from a specific repository root. |
| `RUN_ID` | Optional | Overrides the default generated run id when a runner uses one. |
| `ALICE_HOME` | No for this scenario | Only needed when a separate launch-smoke scenario starts real Alice. |
| `EATME_REAL_ALICE=1` | No for this scenario | Only needed when a separate real Alice launch path is exercised. |

## Review workflow

Use this workflow after the planned scenario asset exists.

1. Choose the scenario:

   ```bash
   export SCENARIO_ID=workshop-facilitator-live-studio
   ```

2. Validate the editable scenario:

   ```bash
   cargo run -q -p eatme-cli -- assets validate \
     --path "assets/scenarios/eatme/${SCENARIO_ID}.yaml" \
     --json
   ```

3. Ask the instructor QA agent to use the scenario prompt:

   ```text
   Use assets/scenarios/eatme/workshop-facilitator-live-studio.yaml.
   Produce the expected workshop_plan, checkpoint_board, helper_roles,
   recovery_moves, and showcase_notes. Address every acceptance_probe.
   ```

4. Review the output against the scenario probes:

   | Probe area | Evidence to look for |
   | --- | --- |
   | Minimum runnable artifact | Participants can finish one small Alice result before optional extensions. |
   | Mixed readiness | Late arrivals, setup failures, and different experience levels have recovery moves. |
   | Help system | Stuck signals and helper roles make support visible without taking over. |
   | Checkpoint decisions | Instructor can pause, pair, demo, extend, or move to share-out based on evidence. |
   | Share-out | Participants show a tiny artifact and one reflection or peer feedback item. |

5. Check generated adapter freshness:

   ```bash
   cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
   ```

6. If the canonical scenario changed, regenerate the adapter and review the diff:

   ```bash
   cargo run -q -p eatme-cli -- assets generate-gadugi --json
   git --no-pager diff -- assets/scenarios/eatme/${SCENARIO_ID}.yaml \
     assets/scenarios/gadugi/${SCENARIO_ID}.yaml
   ```

## Boundaries

This feature will prove a small, explicit coverage connection between the Alice
crew persona inventory and an editable scenario asset. It is not exhaustive for
every Alice persona or every classroom situation.

The workshop scenario reviews facilitation plans, participant evidence, and
agentic outputs. It does not drive every Alice screen, automatically score
creative work, grade saved learner worlds, or replace instructor judgment.

Use a separate real Alice launch-smoke scenario when the evidence needed is
desktop startup, manifest, log, window, screenshot, or deterministic assertion
evidence. Do not treat a launch manifest as proof that participants completed or
understood a workshop activity.
