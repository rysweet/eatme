# Alice Objects-First World Specification

`alice-objects-first-world` is the implementation specification for a planned
Alice learning workflow. The feature must prove that a learner can create or
open a small world, add one visible object, change it, make it move, run the
world, save the project, reopen the saved project, and verify the reopened state.

This page is not a runnable user guide until the implementation lands. Treat it
as the contract for the scenario asset, generated Gadugi adapter, Alice workflow
coordinator, evidence files, and tests that need to be built. For field names,
asset validation rules, and artifact schemas, see
[Alice Objects-First World Reference](alice-objects-first-world-reference.md).

## Contents

- [Implementation status](#implementation-status)
- [What the workflow must prove](#what-the-workflow-must-prove)
- [Target command contract](#target-command-contract)
- [Configuration](#configuration)
- [Implementation validation checklist](#implementation-validation-checklist)
- [Review the target evidence](#review-the-target-evidence)
- [Learner journey represented by the scenario](#learner-journey-represented-by-the-scenario)
- [TypeScript prototype coverage](#typescript-prototype-coverage)
- [Blocked workflow behavior](#blocked-workflow-behavior)

## Implementation status

The documentation defines the feature to build. The implementation is complete
only when these repository surfaces exist and pass validation:

| Surface | Required result |
| --- | --- |
| Canonical scenario asset | `assets/scenarios/eatme/alice-objects-first-world.yaml` exists and validates. |
| Generated Gadugi adapter | `assets/scenarios/gadugi/alice-objects-first-world.yaml` is generated from the canonical asset. |
| CLI entrypoint | A clear full-workflow command exists, preferably `alice run-objects-first-world`. |
| Alice coordinator | `eatme-alice` owns a workflow module that runs each phase in order. |
| Evidence validation | Runtime validation rejects launch-only, unsafe, missing, or out-of-run evidence. |
| Tests | Asset, adapter, coordinator, evidence, and failure-path tests cover the contract. |

Until those surfaces exist, do not describe this scenario as available to
students or instructors.

## What the workflow must prove

The workflow follows the objects-first Alice learning path:

1. Create a new Alice project, or open a prepared starter project when project
   creation is not available to automation.
2. Add one visible object to the world.
3. Change the object's position, rotation, or scale so the change can be
   observed.
4. Edit a procedure so the same object moves when the world runs.
5. Run the world and record the visible result.
6. Save the edited project under the run evidence directory.
7. Reopen that saved project artifact.
8. Verify the reopened project still contains the object, transform, movement
   procedure, and same-run saved artifact link.

A run only passes when each phase has its own accepted evidence. Opening Alice
or recording a launch manifest by itself does not satisfy this scenario.

## Target command contract

The preferred command surface for the feature is a dedicated full-workflow
command:

```bash
export NODE_OPTIONS=--max-old-space-size=32768
export ALICE_HOME=/home/azureuser/src/alice

EATME_REAL_ALICE=1 cargo run -q -p eatme-cli -- alice run-objects-first-world \
  --alice-home "${ALICE_HOME}" \
  --run-id local-alice-objects-first-world \
  --runs-dir runs \
  --timeout 900 \
  --json \
  --no-memory \
  --offline-package
```

If implementation keeps `alice launch-smoke --scenario alice-objects-first-world`
for compatibility, that path must dispatch to the same full-workflow
coordinator. It must not pass on launch-only evidence.

Use a prepared starter project only when Alice project creation is not exposed to
automation:

```bash
EATME_REAL_ALICE=1 cargo run -q -p eatme-cli -- alice run-objects-first-world \
  --alice-home "${ALICE_HOME}" \
  --starter-project core/resources/target/distribution/application/starter-projects/africa.a3p \
  --run-id local-alice-objects-first-world-starter \
  --runs-dir runs \
  --timeout 900 \
  --json \
  --no-memory \
  --offline-package
```

The run report must record whether the project was created or opened from a
starter. Both paths must continue through the same object, transform, procedure,
run, save, reopen, and persisted-state checks.

## Configuration

| Setting | Target value |
| --- | --- |
| Scenario id | `alice-objects-first-world` |
| Canonical scenario asset | `assets/scenarios/eatme/alice-objects-first-world.yaml` |
| Generated Gadugi adapter | `assets/scenarios/gadugi/alice-objects-first-world.yaml` |
| Planned Rust coordinator | `eatme_alice::objects_first_workflow` |
| Primary Alice target | `/home/azureuser/src/alice` |
| Node memory setting | `NODE_OPTIONS=--max-old-space-size=32768` |
| Real Alice gate | `EATME_REAL_ALICE=1` |
| Evidence root | `runs/alice-objects-first-world/<run-id>/` |

The TypeScript web prototype target is optional and future-facing. Do not require
`ALICE_WEB_PROTOTYPE_HOME` for this workflow until a prototype adapter is
implemented and documented as conditional coverage.

## Implementation validation checklist

After the scenario asset exists, validate it directly:

```bash
cargo run -q -p eatme-cli -- assets validate \
  --path assets/scenarios/eatme/alice-objects-first-world.yaml \
  --json
```

After the generated adapter exists, check freshness:

```bash
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
```

After the coordinator and CLI entrypoint exist, run the full workflow against
RabbitHole Alice:

```bash
export NODE_OPTIONS=--max-old-space-size=32768
export ALICE_HOME=/home/azureuser/src/alice

EATME_REAL_ALICE=1 cargo run -q -p eatme-cli -- alice run-objects-first-world \
  --alice-home "${ALICE_HOME}" \
  --run-id local-alice-objects-first-world \
  --runs-dir runs \
  --timeout 900 \
  --json \
  --no-memory \
  --offline-package
```

## Review the target evidence

A complete run writes this evidence layout:

```text
runs/alice-objects-first-world/local-alice-objects-first-world/
|-- manifest.json
|-- alice.log
|-- window-list.txt
|-- screenshots/
|   |-- object-visible.png
|   |-- object-transformed.png
|   `-- world-ran.png
|-- project-open/
|   `-- project-open.json
|-- object-placement/
|   `-- object-placement.json
|-- object-transform/
|   `-- object-transform.json
|-- procedure-edit/
|   |-- procedure-edit.json
|   `-- edited-project.a3p
|-- run-world/
|   `-- run-world.json
|-- project-save/
|   |-- project-save.json
|   `-- saved-project.a3p
`-- project-reopen/
    |-- project-reopen.json
    `-- persisted-state.json
```

Review the scenario id and final status:

```bash
jq '{scenario_id, passed, failure_category}' \
  runs/alice-objects-first-world/local-alice-objects-first-world/manifest.json
```

Review the persisted state:

```bash
jq . \
  runs/alice-objects-first-world/local-alice-objects-first-world/project-reopen/persisted-state.json
```

The persisted state is accepted only when it shows the same learner object, saved
transform, movement procedure, and saved project artifact from the same run.

## Learner journey represented by the scenario

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

The procedure evidence must describe executable movement. Placeholder text or a
comment that does not move the object is not accepted.

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

## TypeScript prototype coverage

Prototype coverage is future and conditional. Do not document
`ALICE_WEB_PROTOTYPE_HOME` or a `ts_prototype_adapter` test as required workflow
steps until the adapter exists.

When prototype support is added, the adapter should report each comparable phase
as one of:

| State | Meaning |
| --- | --- |
| `present` | The prototype claims and proves the comparable behavior. |
| `unsupported` | The prototype does not claim the behavior. This is documented, not filed as a product bug. |
| `blocked` | The prototype claims the behavior, but the adapter cannot collect valid evidence. |
| `invalid` | The prototype returned malformed, unsafe, or out-of-run evidence. |

Unsupported prototype behavior is an explicit gap, not a silent pass.

## Blocked workflow behavior

The scenario fails closed when required proof is missing or malformed. A blocked
run reports the missing phase and writes sanitized issue-ready details without
raw screenshots, secrets, full environment dumps, or local evidence files.

File a product issue when RabbitHole Alice or a supported prototype claims a
workflow phase but the phase cannot produce valid evidence. Link the issue to the
run id and scenario id, then start follow-up implementation work for that product
defect.
