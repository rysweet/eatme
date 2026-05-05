# Live Studio Workshop Evidence Contract

The `workshop-facilitator-live-studio` feature defines a two-hour Alice live
studio workshop evidence contract where instructors and students both produce
reviewable artifacts. When complete, the canonical scenario and generated
Gadugi adapter must preserve the same prompt, probes, rubric, and expected
evidence. This is an editable instructor agentic flow, not an Alice desktop
automation flow.

This contract is **not full Alice user interface automation, creative
assessment, learner-world grading, or complete Alice coverage**.

## Contents

- [What the completed scenario covers](#what-the-completed-scenario-covers)
- [Usage](#usage)
- [Configuration](#configuration)
- [Asset contract reference](#asset-contract-reference)
- [Generated Gadugi adapter](#generated-gadugi-adapter)
- [Instructor evidence](#instructor-evidence)
- [Student evidence](#student-evidence)
- [Tutorial: run a live studio evidence review](#tutorial-run-a-live-studio-evidence-review)
- [Examples](#examples)
- [Scope limits](#scope-limits)

## What the completed scenario covers

`assets/scenarios/eatme/workshop-facilitator-live-studio.yaml` is the canonical
editable source for live studio workshop facilitation evidence. The feature is
complete only when that source keeps the scenario id, filename, and
`kind: instructor_agentic_flow` stable and the generated Gadugi adapter consumes
the same prompt, probes, rubric, and expected artifacts.

The scenario covers a short classroom studio cycle:

1. Setup and launch readiness.
2. Timeboxed build, run, revise, and share checkpoints.
3. Instructor observation points and intervention cues.
4. Student prompt cards or equivalent student-facing flows.
5. Student-owned Alice action evidence: add or adjust one visible behavior,
   run it, record the observed result, and revise one small choice.
6. Help signals and peer feedback.
7. Revision behavior and reflection.
8. Share-out artifacts.
9. Instructor-facing acceptance probes.

The scenario stays at the evidence boundary. It describes what instructors,
students, and instructor-facing agents must produce and review. It does not
claim to click through Alice, judge creative merit automatically, grade saved
world files, or cover every Alice feature.

## Usage

Use these commands when implementing or reviewing the live studio evidence
contract for a short Alice workshop:

```bash
cargo run -q -p eatme-cli -- assets validate \
  --path assets/scenarios/eatme/workshop-facilitator-live-studio.yaml \
  --json
```

Use the full asset validation before trusting the scenario with the persona crew
and generated adapters:

```bash
cargo run -q -p eatme-cli -- assets validate --json
```

Check that the generated Gadugi adapter matches the editable scenario:

```bash
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
```

Regenerate the adapter after changing the canonical scenario:

```bash
cargo run -q -p eatme-cli -- assets generate-gadugi --json
```

The generated adapter is `assets/scenarios/gadugi/workshop-facilitator-live-studio.yaml`.
Do not hand-edit it for mission intent. Edit the canonical eatme scenario and
regenerate.

Validation and freshness checks are necessary but not sufficient. They prove the
assets are shaped correctly and generated files are reproducible; reviewers must
also confirm that the canonical scenario and generated adapter satisfy the
contract below.

## Configuration

Run commands from the repository root unless a command explicitly accepts
`--root`.

The completed live studio scenario does not require a new external service,
credential, account, or environment variable. The generated Gadugi adapter uses
`EATME_REPO` before it runs eatme commands when the runner needs to point at a
repository checkout:

```bash
export EATME_REPO=/path/to/eatme
```

For direct local CLI use from another directory, pass `--root` to the command
that supports it:

```bash
cargo run -q -p eatme-cli -- assets generate-gadugi \
  --root /path/to/eatme \
  --check \
  --json
```

## Asset contract reference

The target live studio contract is expressed through the canonical scenario
fields.

| Field | Required meaning |
| --- | --- |
| `schema_version` | `eatme.scenario/v1`. |
| `id` | `workshop-facilitator-live-studio`. |
| `kind` | `instructor_agentic_flow`; the scenario is prompt, probe, rubric, and artifact evidence. |
| `resource_basis` | Alice.org resources that ground the workshop plan and student activity. |
| `purpose` | A durable classroom statement that names instructor and student evidence and the explicit scope limits. |
| `personas.instructors` | Includes `workshop-facilitator` and supporting instructor personas that run setup, timing, intervention, and share-out work. |
| `personas.students` | Includes student personas that participate through prompt cards, help signals, peer feedback, revision, reflection, and share-out artifacts. |
| `agentic_flow.instructor_goal` | Asks for a two-hour live studio facilitation packet with setup, timing, checkpoints, interventions, student participation flow, and share-out support. |
| `agentic_flow.expected_outputs` | Names the durable instructor and student evidence artifacts an instructor-facing agent must return. |
| `agentic_test_prompt` | Gives the instructor-facing agent the full workshop task without internal shorthand. |
| `acceptance_criteria` | Uses given/when/then criteria for setup, timing, observation, intervention, checkpoint artifacts, student participation, reflection, and share-out. |
| `acceptance_probes` | Reviewer checks that the output represents both instructor facilitation and student participation evidence. |
| `rubric` | Scores readiness, facilitation, participation, revision, reflection, share-out, and evidence boundary. |
| `avoid` | Rejects presenter-only demos, exact coordinates, private implementation details, unclear shorthand, and overclaims. |
| `steps` | Validates assets and runs the instructor agentic acceptance review. |
| `artifacts` | Declares the instructor and student evidence packet outputs. |
| `unsupported_policy` | Fails visibly when required editable assets or resource basis cannot be read; it does not substitute hidden automation or grading claims. |

There is no separate public application programming interface for the feature.
The supported interface is the editable YAML scenario, the generated Gadugi YAML
adapter, and the existing `eatme-cli` asset commands.

## Generated Gadugi adapter

A generated adapter is correct only when it mirrors the completed canonical
scenario for a Gadugi runner. It does three things:

1. Runs `cargo run -q -p eatme-cli -- assets validate --json`.
2. Presents the scenario prompt and acceptance probes to an instructor-facing
   agent.
3. Requires the expected instructor and student evidence artifacts to appear in
   the agent output.

Generator-owned wording must use explicit terms such as `instructor acceptance
review`, `instructor quality adapter`, or `instructor acceptance agent`. It must
not use unclear internal shorthand in generated evidence.

The adapter must preserve the same scope boundary as the canonical scenario:

> This scenario is not full Alice user interface automation, creative
> assessment, learner-world grading, or complete Alice coverage.

## Instructor evidence

A complete instructor evidence packet contains:

| Artifact | Required contents |
| --- | --- |
| `facilitation_plan` | Two-hour agenda, setup assumptions, readiness checks, timing, helper roles, fallback participation choices, and share-out timing. |
| `timing_plan` | Timeboxed setup, build, run, revise, feedback, reflection, and share-out checkpoints for the two-hour studio. |
| `observation_intervention_guide` | What the facilitator watches for during setup, first build, first run, revision, peer feedback, reflection, and share-out, plus the cues that trigger help, pacing changes, pairing, fallback assets, or a reduced minimum runnable artifact. |
| `participant_checkpoint_board` | Classroom board or equivalent record of each pair's current checkpoint, help signal, visible behavior, revision status, and share-out readiness. |
| `instructor_acceptance_probe_notes` | Reviewer notes that confirm the plan is actionable, student-facing, evidence-based, and within scope. |
| `showcase_notes` | Prompts and timing for a short final share that asks for visible behavior, help or revision evidence, and next small change. |

The instructor evidence is complete when another facilitator can run the session
without guessing the setup sequence, timing, observation points, intervention
thresholds, checkpoint artifacts, or share-out format.

## Student evidence

A complete student evidence packet contains:

| Artifact | Required contents |
| --- | --- |
| `student_prompt_cards` | Plain-language prompts for the minimum runnable artifact, optional extension choices, prediction, run, revise, and share-out. |
| `real_alice_action_evidence_notes` | Student-owned Alice action evidence that records one added or adjusted visible behavior, the run result, and one small revision. |
| `help_signal_board` | Student-facing ways to ask for help without losing ownership of the artifact, such as stuck, needs partner review, setup blocked, or ready for stretch. |
| `peer_feedback_notes` | Feedback that names one observed behavior, one question, and one suggested next change. |
| `revision_reflection_log` | One meaningful revision based on observation, help, or peer feedback, plus student explanation of expected versus actual behavior and what changed. |
| `share_out_artifacts` | The visible behavior, screenshot, description, or saved artifact reference used during the final share. |

Student evidence is not graded by the scenario. The scenario only requires
reviewable participation evidence that an instructor or instructor-facing agent
can inspect.

## Tutorial: run a live studio evidence review

### 1. Confirm the canonical scenario

Open the canonical scenario:

```text
assets/scenarios/eatme/workshop-facilitator-live-studio.yaml
```

Confirm it keeps:

- `id: workshop-facilitator-live-studio`
- `kind: instructor_agentic_flow`
- `owner: eatme`
- explicit instructor and student evidence
- the scope statement that it is not full Alice user interface automation,
  creative assessment, learner-world grading, or complete Alice coverage

### 2. Validate the editable asset

```bash
cargo run -q -p eatme-cli -- assets validate \
  --path assets/scenarios/eatme/workshop-facilitator-live-studio.yaml \
  --json
```

The result is acceptable only when `passed` is `true`.

### 3. Run full asset validation

```bash
cargo run -q -p eatme-cli -- assets validate --json
```

The result is acceptable only when the complete persona and scenario inventory
passes.

### 4. Refresh or check the generated adapter

After editing the canonical scenario:

```bash
cargo run -q -p eatme-cli -- assets generate-gadugi --json
```

Before committing:

```bash
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
```

### 5. Review the instructor-facing output

Apply the acceptance probes to the generated workshop packet. Accept the output
only when it includes both instructor and student evidence:

- setup, timing, observation, intervention, checkpoint, and share-out support
- prompt cards or equivalent student-facing flows, help signals, peer feedback,
  revision behavior, reflection, and share-out artifacts
- explicit refusal to overclaim Alice automation, automated assessment, learner
  grading, or complete Alice coverage

## Examples

### Instructor prompt card

```text
Live studio facilitator card

Goal:
Help each student pair produce one minimum runnable Alice behavior, revise it
from observation or feedback, and share the evidence before the session ends.

Watch for:
- setup blockers before the first build checkpoint
- pairs that have not run the world before the midpoint
- students who can describe polish but not behavior
- students who need a smaller minimum runnable artifact

Intervene when:
- a pair is blocked for more than one checkpoint
- the room is behind the timing plan
- a student is watching instead of building, observing, revising, or reflecting
```

### Student prompt card

```text
Alice live studio card

Build:
Create one visible behavior that another person can observe.

Predict:
Before running, write what you expect the audience to see.

Run and observe:
Run the world and write what actually happened.

Revise:
Change one meaningful thing based on your observation or peer feedback.

Share:
Show the visible behavior, name one help or revision moment, and describe your
next small change.
```

### Peer feedback note

```text
Observed behavior:
The character turned toward the camera after the scene started.

Question:
Should the audience notice the character movement or the background first?

Suggested next change:
Try a shorter wait before the turn, then run again and compare the timing.
```

### Checkpoint board row

```text
Pair: Student pair 4
Checkpoint: Ran the world once
Help signal: Needs peer review
Visible behavior: Rabbit hops toward the tree
Revision target: Adjust timing so the hop starts after the camera settles
Share-out ready: Not yet
```

## Scope limits

The live studio scenario is intentionally narrow.

| Not in scope | Reason |
| --- | --- |
| Full Alice user interface automation | The scenario is an instructor agentic flow, not a desktop automation harness. |
| Creative assessment | Creative judgment remains with the instructor and classroom review. |
| Learner-world grading | The scenario reviews evidence packets, not saved world correctness or private learner files. |
| Complete Alice coverage | The scenario covers one live studio workshop pattern, not every Alice lesson, feature, or workflow. |

If an output claims any of those capabilities, reject it even when the rest of
the packet looks useful.
