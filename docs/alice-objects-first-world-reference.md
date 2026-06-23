# Alice Objects-First World Reference

This reference defines the planned scenario asset, target command, workflow phase
contracts, evidence files, manifest fields, and safety rules for
`alice-objects-first-world`.

For the learner workflow and implementation checklist, see
[Alice Objects-First World Specification](alice-objects-first-world.md).

## Contents

- [Scenario identity](#scenario-identity)
- [Scenario YAML contract](#scenario-yaml-contract)
- [Configuration](#configuration)
- [Target command contract](#target-command-contract)
- [Workflow phase contracts](#workflow-phase-contracts)
- [Evidence contract](#evidence-contract)
- [Manifest fields](#manifest-fields)
- [Rust API](#rust-api)
- [TypeScript prototype adapter](#typescript-prototype-adapter)
- [Path and issue safety](#path-and-issue-safety)

## Scenario identity

| Field | Target value |
| --- | --- |
| Scenario id | `alice-objects-first-world` |
| Title | Alice Objects-First World |
| Kind | `alice_objects_first_workflow` |
| Canonical source | `assets/scenarios/eatme/alice-objects-first-world.yaml` |
| Generated adapter | `assets/scenarios/gadugi/alice-objects-first-world.yaml` |
| Primary runner | `eatme_alice::objects_first_workflow` |
| Primary Alice target | `$ALICE_HOME` |

The canonical eatme scenario owns mission wording, required steps, required
evidence, acceptance criteria, artifact names, and unsupported behavior. The
Gadugi adapter must be generated from the canonical scenario and must not be
hand-edited for mission intent.

## Scenario YAML contract

The implementation must add a canonical scenario asset using the repository's
`eatme.scenario/v1` shape, plus objects-first validation fields that the asset
validator enforces when `kind: alice_objects_first_workflow`.

```yaml
schema_version: eatme.scenario/v1
id: alice-objects-first-world
title: Alice Objects-First World
kind: alice_objects_first_workflow
owner: eatme
tags:
  - alice
  - desktop
  - objects-first
  - save-reopen
  - persisted-state
launcher:
  command: alice run-objects-first-world
  scenario: alice-objects-first-world
real_alice:
  gated_by: EATME_REAL_ALICE=1
personas:
  students:
    - curious-novice
    - reflective-debugger
capabilities:
  required:
    - rust-cli
    - java-21
    - maven
    - xvfb
    - xdpyinfo
    - wmctrl
    - xwininfo
    - xdotool
    - screenshot-tool
workflow_phases:
  - project-open
  - object-placement
  - object-transform
  - procedure-edit
  - run-world
  - project-save
  - project-reopen
  - persisted-state
smoke_ready:
  evidence:
    - project_open_evidence
    - object_placement_evidence
    - object_transform_evidence
    - procedure_movement_evidence
    - run_world_evidence
    - saved_project_artifact
    - reopened_project_evidence
    - persisted_state_evidence
steps:
  - id: validate-assets
    command: cargo run -q -p eatme-cli -- assets validate --json
    evidence:
      - stdout JSON has passed=true
  - id: check-dependencies
    command: cargo run -q -p eatme-cli -- deps check --json
    evidence:
      - stdout JSON has all_required_available=true
  - id: run-objects-first-world
    command: >-
      EATME_REAL_ALICE=1 cargo run -q -p eatme-cli -- alice run-objects-first-world
      --alice-home ${ALICE_HOME}
      --run-id ${RUN_ID}
      --runs-dir runs
      --timeout 900
      --json
      --no-memory
      --offline-package
    evidence:
      - manifest scenario_id equals alice-objects-first-world
      - manifest workflow_phases contains every required phase in order
      - persisted-state evidence links to project-save/saved-project.a3p
artifacts:
  manifest: runs/alice-objects-first-world/${RUN_ID}/manifest.json
  persisted_state: runs/alice-objects-first-world/${RUN_ID}/project-reopen/persisted-state.json
  saved_project: runs/alice-objects-first-world/${RUN_ID}/project-save/saved-project.a3p
unsupported_policy: >-
  Missing host tools, missing EATME_REAL_ALICE=1, unsupported Alice project
  creation, unavailable deterministic object placement, missing movement edit,
  missing run-world proof, missing save proof, missing reopen proof, or unsafe
  artifact paths must fail loudly. Do not substitute launch-only evidence.
```

Validation rules to add with the implementation:

| Rule | Requirement |
| --- | --- |
| Filename alignment | The canonical filename must be `alice-objects-first-world.yaml`. |
| Kind-specific tags | `alice`, `desktop`, `objects-first`, `save-reopen`, and `persisted-state` must be present. |
| Phase coverage | `workflow_phases` must contain the eight required phases exactly once and in order. |
| Step reference | A `run-objects-first-world` step must invoke the full workflow command or a compatibility command that dispatches to the same coordinator. |
| Evidence coverage | `smoke_ready.evidence`, `steps[].evidence`, and `artifacts` must reference persisted-state and same-run saved-project proof. |
| Boundary wording | `unsupported_policy` must reject launch-only proof and silent fallback behavior. |

## Configuration

| Name | Required | Description |
| --- | --- | --- |
| `EATME_REAL_ALICE=1` | Yes for execution | Enables real Alice desktop execution for non-baseline scenarios. |
| `ALICE_HOME=$ALICE_HOME` | Yes for RabbitHole validation | Points to the RabbitHole Alice checkout. |
| `NODE_OPTIONS=--max-old-space-size=32768` | Yes for Node-backed wrappers | Gives Node-backed wrappers enough memory. |
| `--run-id <id>` | Yes | Names the evidence directory for this run. |
| `--runs-dir <path>` | Optional | Defaults to `runs`. |
| `--starter-project <path>` | Optional | Opens a prepared starter project when Alice project creation is not available. |
| `--timeout <seconds>` | Optional | Upper bound for the scenario run. Use `900` or higher for real desktop runs. |
| `--offline-package` | Optional | Packages Alice with Maven offline mode before running. |
| `--no-memory` | Optional | Disables memory writes for the run. |

Evidence artifacts must use paths relative to the run evidence directory. Any
artifact path that is absolute, escapes with `..`, points through a symlink
escape, or resolves outside the run directory is rejected.

## Target command contract

Preferred command:

```bash
export NODE_OPTIONS=--max-old-space-size=32768
export ALICE_HOME=$ALICE_HOME

EATME_REAL_ALICE=1 cargo run -q -p eatme-cli -- alice run-objects-first-world \
  --alice-home "${ALICE_HOME}" \
  --run-id local-alice-objects-first-world \
  --runs-dir runs \
  --timeout 900 \
  --json \
  --no-memory \
  --offline-package
```

The command exits successfully only when every required workflow phase is
accepted. It exits with failure when the run launches Alice but does not prove
the full workflow.

If `alice launch-smoke --scenario alice-objects-first-world` remains supported,
it must be a compatibility path that dispatches to this same coordinator and
inherits the same pass/fail rules.

## Workflow phase contracts

The coordinator runs these phases in order and stops on the first missing
prerequisite. It does not mark later phases as passed when an earlier phase is
blocked.

| Phase id | Required proof |
| --- | --- |
| `project-open` | A new project was created, or the configured starter project was opened. |
| `object-placement` | A named visible object was added to the world. |
| `object-transform` | The object position, rotation, or scale changed from its initial state. |
| `procedure-edit` | A procedure contains executable movement for the named object. |
| `run-world` | The world ran after the movement edit and produced visible run evidence. |
| `project-save` | The edited project was saved under the current run evidence directory. |
| `project-reopen` | The saved project artifact was reopened, not the starter project. |
| `persisted-state` | The reopened project still contains the object, transform, and movement procedure. |

This contract does not invent Alice-side `tools/eatme-*` commands for every
phase. The implementation may use a Rust coordinator, existing documented
save/reopen hooks, desktop automation, Alice APIs, or a combination of those
mechanisms, but each phase must produce the evidence schemas below.

## Evidence contract

All evidence is written under:

```text
runs/alice-objects-first-world/<run-id>/
```

Required artifacts:

| Artifact | Schema version | Required contents |
| --- | --- | --- |
| `manifest.json` | `eatme.alice-run-manifest/v1` | Scenario id, run id, Alice target, artifact list, phase statuses, assertions, and failure category. |
| `project-open/project-open.json` | `eatme.alice-project-open/v1` | `status`, `mode` (`created` or `opened_starter`), `project_artifact`, and project title. |
| `object-placement/object-placement.json` | `eatme.alice-object-placement/v1` | `status`, `object_id`, `object_label`, visibility proof, and scene or project diff artifact. |
| `object-transform/object-transform.json` | `eatme.alice-object-transform/v1` | `status`, `object_id`, before transform, after transform, and transform kind. |
| `procedure-edit/procedure-edit.json` | `eatme.alice-procedure-edit/v1` | `status`, `procedure_id`, `object_id`, movement intent, movement operation, and edited project artifact. |
| `run-world/run-world.json` | `eatme.alice-run-world/v1` | `status`, run trigger, object movement observation, screenshot or window evidence, and log artifact. |
| `project-save/project-save.json` | `eatme.alice-project-save/v1` | `status`, source edited project, saved project artifact, and save proof. |
| `project-reopen/project-reopen.json` | `eatme.alice-project-reopen/v1` | `status`, source saved project artifact, reopened project artifact, and reopen proof. |
| `project-reopen/persisted-state.json` | `eatme.alice-persisted-state/v1` | `status`, object state, transform state, procedure movement state, and same-run artifact links. |

`persisted-state.json` is the final acceptance artifact. It must contain:

```json
{
  "schema_version": "eatme.alice-persisted-state/v1",
  "status": "present",
  "scenario_id": "alice-objects-first-world",
  "object": {
    "id": "bunny",
    "visible": true
  },
  "transform": {
    "changed": true,
    "after": {
      "position": [1.0, 0.0, 0.0]
    }
  },
  "procedure": {
    "id": "world.myFirstMethod",
    "movement_operation": "move",
    "target_object_id": "bunny"
  },
  "source_saved_project_artifact": "project-save/saved-project.a3p"
}
```

The object id, transform, and procedure movement must match the evidence from
earlier phases. A reopened project that points to a different saved artifact or
to the starter project is rejected.

## Manifest fields

The scenario extends the existing Alice run manifest with workflow evidence:

| Field | Meaning |
| --- | --- |
| `scenario_id` | Must equal `alice-objects-first-world`. |
| `run_id` | Caller-supplied run id. |
| `alice_home` | Alice checkout used for the run. |
| `workflow_phases[]` | Ordered phase statuses from project open through persisted-state verification. |
| `artifacts[]` | Relative artifact paths and content digests for required evidence. |
| `persisted_state.path` | Relative path to `project-reopen/persisted-state.json`. |
| `product_issue_candidates[]` | Sanitized issue-ready summaries for claimed product behavior that blocked the workflow. |
| `failure_category` | `null` on pass; otherwise the blocked or invalid phase category. |

Consumers must use `workflow_phases[]`, `persisted_state`, and
`failure_category` instead of inferring success from process startup or
screenshots alone.

## Rust API

The planned workflow coordinator is exposed by `eatme-alice`:

```rust
use eatme_alice::objects_first_workflow::{
    run_objects_first_workflow, ObjectsFirstWorkflowOptions,
};

let report = run_objects_first_workflow(ObjectsFirstWorkflowOptions {
    alice_home: "$ALICE_HOME".into(),
    run_id: "local-alice-objects-first-world".into(),
    runs_dir: "runs".into(),
    starter_project: None,
    timeout_seconds: 900,
    no_memory: true,
    offline_package: true,
})?;

assert!(report.passed);
```

Important public types:

| Type | Purpose |
| --- | --- |
| `ObjectsFirstWorkflowOptions` | Inputs for Alice home, run id, evidence root, timeout, package mode, memory mode, and optional starter project. |
| `ObjectsFirstWorkflowReport` | Top-level scenario result, phase list, artifact list, persisted-state summary, and product issue candidates. |
| `WorkflowPhaseEvidence` | One phase status with required artifact references and failure category. |
| `PersistedStateEvidence` | Parsed object, transform, and procedure movement state from the reopened project. |

## LookingGlass adapter

LookingGlass coverage is future and conditional. Do not require
`LOOKINGGLASS_HOME` or document a LookingGlass adapter test as part of
the runnable workflow until that adapter exists.

When implemented, adapter outputs must use these states:

| State | Meaning |
| --- | --- |
| `present` | The prototype claims and proves the comparable behavior. |
| `unsupported` | The prototype does not claim the behavior. This is documented, not filed as a product bug. |
| `blocked` | The prototype claims the behavior, but the adapter cannot collect valid evidence. |
| `invalid` | The prototype returned malformed, unsafe, or out-of-run evidence. |

Reopen and transform gaps are product issues only when the TypeScript port claims
those behaviors. Otherwise they remain explicit unsupported gaps in the adapter
result.

## Path and issue safety

The workflow rejects evidence that escapes the run directory. It also rejects
evidence that contains secrets, browser/session credentials, SSH keys, raw
environment dumps, or unrelated desktop captures.

When a product bug blocks the workflow, issue-ready output includes only:

- scenario id;
- run id;
- blocked phase;
- target name (`RabbitHole Alice` or supported prototype);
- display-safe summary;
- expected behavior;
- observed behavior;
- sanitized artifact names, not raw artifact contents.

Do not attach local evidence directories, raw screenshots, logs with secrets, or
absolute machine paths to product issues.
