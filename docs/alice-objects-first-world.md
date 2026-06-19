# Alice Objects-First World

`alice-objects-first-world` is the Alice learning workflow that proves a
learner can build, run, save, and reopen a small world with an object that
moves.

Use this page when you want to run or review the full workflow. For field
names, hook contracts, and artifact schemas, see
[Alice Objects-First World Reference](alice-objects-first-world-reference.md).

## Contents

- [What the workflow does](#what-the-workflow-does)
- [Quick start](#quick-start)
- [Configuration](#configuration)
- [Run the workflow](#run-the-workflow)
- [Review the evidence](#review-the-evidence)
- [Tutorial: the learner journey](#tutorial-the-learner-journey)
- [TypeScript prototype check](#typescript-prototype-check)
- [When the workflow is blocked](#when-the-workflow-is-blocked)

## What the workflow does

The workflow follows the objects-first Alice learning path:

1. Create a new Alice project, or open the prepared starter project when project
   creation is not available.
2. Add one visible object to the world.
3. Change the object's position or transform so the change can be observed.
4. Edit a procedure so the object moves when the world runs.
5. Run the world and record the visible result.
6. Save the project.
7. Reopen the saved project.
8. Verify the reopened project still contains the object, transform, procedure
   movement, and saved run evidence.

A run only passes when each step has its own evidence. Opening Alice by itself
does not satisfy this scenario.

## Quick start

Run from the eatme repository root:

```bash
export NODE_OPTIONS=--max-old-space-size=32768
export ALICE_HOME=/home/azureuser/src/alice

EATME_REAL_ALICE=1 cargo run -q -p eatme-cli -- alice launch-smoke \
  --alice-home "${ALICE_HOME}" \
  --scenario alice-objects-first-world \
  --run-id local-alice-objects-first-world \
  --runs-dir runs \
  --timeout 900 \
  --json \
  --no-memory \
  --offline-package
```

The command name is shared with the existing Alice runner. This scenario is not
accepted unless the full object, procedure, run, save, reopen, and persisted
state evidence exists.

## Configuration

| Setting | Value |
| --- | --- |
| Scenario id | `alice-objects-first-world` |
| Canonical scenario asset | `assets/scenarios/eatme/alice-objects-first-world.yaml` |
| Gadugi adapter | `assets/scenarios/gadugi/alice-objects-first-world.yaml` |
| Primary Alice target | `/home/azureuser/src/alice` |
| TypeScript prototype target | `/home/azureuser/src/alice-web-prototype` |
| Node memory setting | `NODE_OPTIONS=--max-old-space-size=32768` |
| Real Alice gate | `EATME_REAL_ALICE=1` |
| Evidence root | `runs/alice-objects-first-world/<run-id>/` |

Set `ALICE_HOME` to the RabbitHole Alice checkout before running the full
workflow. Set `ALICE_WEB_PROTOTYPE_HOME` only when running the TypeScript
prototype check.

## Run the workflow

Validate the scenario asset:

```bash
cargo run -q -p eatme-cli -- assets validate \
  --path assets/scenarios/eatme/alice-objects-first-world.yaml \
  --json
```

Check the generated Gadugi adapter:

```bash
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
```

Run the workflow against RabbitHole Alice:

```bash
export NODE_OPTIONS=--max-old-space-size=32768
export ALICE_HOME=/home/azureuser/src/alice

EATME_REAL_ALICE=1 cargo run -q -p eatme-cli -- alice launch-smoke \
  --alice-home "${ALICE_HOME}" \
  --scenario alice-objects-first-world \
  --run-id local-alice-objects-first-world \
  --runs-dir runs \
  --timeout 900 \
  --json \
  --no-memory \
  --offline-package
```

Use a prepared starter project only when Alice project creation is not exposed
to automation:

```bash
EATME_REAL_ALICE=1 cargo run -q -p eatme-cli -- alice launch-smoke \
  --alice-home "${ALICE_HOME}" \
  --scenario alice-objects-first-world \
  --starter-project core/resources/target/distribution/application/starter-projects/africa.a3p \
  --run-id local-alice-objects-first-world-starter \
  --runs-dir runs \
  --timeout 900 \
  --json \
  --no-memory \
  --offline-package
```

The run report records whether the project was created or opened. Both paths
must continue through the same object, transform, procedure, run, save, reopen,
and persisted-state checks.

## Review the evidence

A complete run writes this evidence layout:

```text
runs/alice-objects-first-world/local-alice-objects-first-world/
├── manifest.json
├── alice.log
├── window-list.txt
├── screenshots/
│   ├── object-visible.png
│   ├── object-transformed.png
│   └── world-ran.png
├── project-open/
│   └── project-open.json
├── object-placement/
│   └── object-placement.json
├── object-transform/
│   └── object-transform.json
├── procedure-edit/
│   ├── procedure-edit.json
│   └── edited-project.a3p
├── run-world/
│   └── run-world.json
├── project-save/
│   ├── project-save.json
│   └── saved-project.a3p
└── project-reopen/
    ├── project-reopen.json
    └── persisted-state.json
```

Review the scenario id and final status:

```bash
jq '{scenario_id, passed, failure_category}' \
  runs/alice-objects-first-world/local-alice-objects-first-world/manifest.json
```

Review the persisted state:

```bash
jq '.persisted_state' \
  runs/alice-objects-first-world/local-alice-objects-first-world/project-reopen/persisted-state.json
```

The persisted state is accepted only when it shows the same learner object, the
saved transform, the movement procedure, and the reopened project artifact from
the same run.

## Tutorial: the learner journey

This is the learner-facing activity the scenario represents.

### Start the world

Create a new Alice world. If the classroom build only supports opening a prepared
project, open the provided starter world.

Name the project:

```text
Objects First World
```

### Add an object

Add one visible character or prop. The object must appear in the world and must
have a stable name in the evidence report.

Example:

```text
Object: bunny
Starting position: center of the scene
```

### Change the object

Move, rotate, or resize the object so the change can be seen before the world
runs.

Example:

```text
Change: move bunny 1 meter to the right and rotate it toward the camera
```

### Make the object move

Edit the world procedure so the object moves when the world runs.

Example learner intent:

```text
When the world starts, bunny moves forward 1 meter.
```

The procedure evidence must describe movement. Placeholder text or a comment
that does not move the object is not accepted.

### Run and observe

Run the world. Record what changed on screen.

Example observation:

```text
Bunny started on the right side of the scene, then moved forward when the world
ran.
```

### Save, reopen, and check

Save the project, close or reset the Alice session, then reopen the saved
project. The reopened project must still show:

- the named object;
- the transform made before running;
- the movement in the procedure;
- the saved project artifact used for the reopen step.

## TypeScript prototype check

The TypeScript prototype check runs the comparable pieces that the prototype
supports:

```bash
export NODE_OPTIONS=--max-old-space-size=32768
export ALICE_WEB_PROTOTYPE_HOME=/home/azureuser/src/alice-web-prototype

cargo test -p eatme-alice --test ts_prototype_adapter -- --ignored
```

The prototype check records supported workflow pieces such as object creation,
visible transform state, or procedure movement when those features exist in the
prototype. Unsupported reopen or transform behavior is documented as an
unsupported gap only when the prototype does not claim that feature.

## When the workflow is blocked

The scenario fails closed when required proof is missing or malformed. A blocked
run reports the missing phase and writes sanitized issue-ready details without
raw screenshots, secrets, full environment dumps, or local evidence files.

File a product issue when RabbitHole Alice or the TypeScript prototype claims to
support a workflow phase but the phase cannot produce valid evidence. Link the
issue to the run id and scenario id, then start follow-up default-workflow work
for that product defect.
