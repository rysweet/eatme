# Scenario authoring

Eatme scenarios are the editable source for Alice mission intent. Authors edit
YAML in `assets/scenarios/eatme/`, validate it, and then refresh the generated
runner files.

## Authoring rules

1. Edit eatme scenarios first.
2. Keep scenario ids stable and filename-aligned.
3. Describe visible learner or harness evidence, not private implementation
   details.
4. State unsupported behavior explicitly instead of allowing silent skips.
5. Validate assets before committing.
6. Regenerate or check generated runner files after scenario changes.

## Asset locations

Editable eatme scenarios:

```text
assets/scenarios/eatme/
```

Generated runner files:

```text
assets/scenarios/gadugi/
```

Persona crews:

```text
assets/personas/
```

## Scenario categories

| Category | Purpose |
| --- | --- |
| `real-alice-launch-smoke` | Baseline repeatable Alice desktop smoke |
| Alice lesson smoke scenarios | Scenario-labeled launch readiness for Alice.org-grounded lesson scenarios |
| Instructor agentic flows | Instructor-facing mission prompts, acceptance probes, and rubrics |

Lesson smoke scenarios route through:

```bash
EATME_REAL_ALICE=1 cargo run -q -p eatme-cli -- alice launch-smoke \
  --alice-home "${ALICE_HOME}" \
  --scenario <scenario-id> \
  --run-id local-<scenario-id> \
  --json
```

Instructor agentic flows stay at the prompt, acceptance-probe, and rubric
boundary. They do not own Alice desktop launch internals.

## Required scenario shape

Eatme scenario assets use `eatme.scenario/v1`:

```yaml
schema_version: eatme.scenario/v1
id: building-a-scene-first-world
title: Building a Scene First World
kind: alice_lesson_smoke
owner: eatme
purpose: >-
  Prove that the lesson-specific smoke scenario launches through the real Alice
  desktop harness and records scenario-labeled evidence.
launcher:
  command: alice launch-smoke
  scenario: building-a-scene-first-world
real_alice:
  gated_by: EATME_REAL_ALICE=1
smoke_ready:
  evidence:
    - manifest_assertions
    - captured_logs
    - screenshot_or_window_evidence
    - scenario_id
acceptance_criteria:
  - given: Alice launch smoke dependencies are available
    when: the scenario is launched through eatme
    then: the manifest identifies the selected scenario id
steps:
  - id: launch-smoke
    command: >-
      EATME_REAL_ALICE=1 cargo run -q -p eatme-cli -- alice launch-smoke
      --alice-home ${ALICE_HOME}
      --scenario building-a-scene-first-world
      --json
    evidence:
      - manifest scenario_id equals building-a-scene-first-world
      - manifest assertions all pass
timeouts:
  scenario_seconds: 1800
  launch_seconds: 900
artifacts:
  manifest: runs/building-a-scene-first-world/${RUN_ID}/manifest.json
  screenshot: runs/building-a-scene-first-world/${RUN_ID}/screenshots/startup.png
  log: runs/building-a-scene-first-world/${RUN_ID}/alice.log
unsupported_policy: >-
  If host graphics, Java, Maven, or the real Alice gate are missing, fail loudly
  rather than substituting a mocked runtime.
```

## Validation workflow

Validate all assets:

```bash
cargo run -q -p eatme-cli -- assets validate --json
```

Validate the scenario being edited:

```bash
cargo run -q -p eatme-cli -- assets validate \
  --path assets/scenarios/eatme/<scenario-id>.yaml \
  --json
```

Check generated runner files:

```bash
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
```

If the check fails because generated files are stale, regenerate them:

```bash
cargo run -q -p eatme-cli -- assets generate-gadugi --json
```

See [Generated Asset Consistency](generated-asset-consistency.md) for the
`scenario_asset_count` source of truth and the add, remove, and rename workflow
for generated files. When removing or renaming an editable scenario, delete the
old generated Gadugi file too; check mode compares expected targets but does not
prune orphaned files.

## Evidence language

Good scenario evidence is observable:

- run summary identifies the scenario id
- repeatable launch assertions pass
- screenshot artifact is non-empty
- log artifact exists and has a digest
- learner predicts a visible behavior before running
- student reflection names expected versus actual behavior
- instructor rubric checks concept evidence, process, creativity, and reflection

Avoid brittle evidence:

- exact UI coordinates
- private class names unless the CLI run summary owns them
- screenshots judged only for visual polish
- one-path-only instructions that prevent learner choice
- silent fallback behavior when prerequisites are missing

## Technical terms versus reader wording

Some editable scenarios and generated files intentionally retain technical names
such as `kind: alice_real_ui_action_contract`, `ui-action-contract.json`,
`adapter`, `schema_version`, and `manifest`. Use those names when referring to
exact fields, files, command output, or compatibility names. In reader-facing
guides, translate the same boundary to plain wording such as first-lesson
readiness evidence, first-action evidence, generated runner files, and run
summary evidence.

## Plain evidence wording for Alice lesson scenarios

When writing reader-facing documentation or scenario prose for real Alice lesson
scenarios, describe the finished evidence boundary exactly:

| Claim | Acceptable wording |
| --- | --- |
| Launch smoke | "records scenario-labeled run summary, log, window, screenshot, and assertion evidence" |
| Student action path | "records first-action evidence for first object placement, procedure/code edit, run-world, and save-project boundaries when each boundary can be proven" |
| Instructor remix | "produces teacher plan, student handout, exit ticket, review prompts, and remix notes" |
| Boundary | "not full UI automation, not creative assessment, and not learner-world grading" |

Do not write that the launch smoke completes a lesson, clicks through the Alice
UI, evaluates a creative project, or grades a learner's world unless a separate
scenario owns that evidence and validation path.

For the instructor/student readiness states, JSON fields, and unsupported-action
wording, see [Lesson Session Readiness](lesson-session-readiness.md).
