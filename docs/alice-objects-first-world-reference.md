# Alice Objects-First World Reference

This reference defines the scenario asset, command contract, configuration,
evidence files, and hook APIs for `alice-objects-first-world`.

For step-by-step use, see
[Alice Objects-First World](alice-objects-first-world.md).

## Contents

- [Scenario identity](#scenario-identity)
- [Configuration](#configuration)
- [Command contract](#command-contract)
- [Evidence contract](#evidence-contract)
- [Manifest fields](#manifest-fields)
- [Alice-side hook API](#alice-side-hook-api)
- [Rust API](#rust-api)
- [TypeScript prototype adapter](#typescript-prototype-adapter)
- [Path and issue safety](#path-and-issue-safety)

## Scenario identity

| Field | Value |
| --- | --- |
| Scenario id | `alice-objects-first-world` |
| Title | Alice Objects-First World |
| Kind | `alice_objects_first_workflow` |
| Canonical source | `assets/scenarios/eatme/alice-objects-first-world.yaml` |
| Generated adapter | `assets/scenarios/gadugi/alice-objects-first-world.yaml` |
| Primary runner | `eatme-alice::objects_first_workflow` |
| Primary Alice target | `/home/azureuser/src/alice` |
| Comparable prototype target | `/home/azureuser/src/alice-web-prototype` |

The canonical eatme scenario owns mission wording, required steps, required
evidence, acceptance criteria, artifact names, and unsupported behavior. The
Gadugi adapter is generated from the canonical scenario and must not be edited by
hand for mission intent.

## Configuration

| Name | Required | Description |
| --- | --- | --- |
| `EATME_REAL_ALICE=1` | Yes for execution | Enables real Alice desktop execution for non-baseline scenarios. |
| `ALICE_HOME=/home/azureuser/src/alice` | Yes for RabbitHole validation | Points to the RabbitHole Alice checkout. |
| `NODE_OPTIONS=--max-old-space-size=32768` | Yes for Node-backed checks | Gives Node-backed wrappers enough memory. |
| `ALICE_WEB_PROTOTYPE_HOME=/home/azureuser/src/alice-web-prototype` | Only for prototype checks | Points to the TypeScript prototype checkout. |
| `--run-id <id>` | Yes | Names the evidence directory for this run. |
| `--runs-dir <path>` | Optional | Defaults to `runs`. |
| `--starter-project <path>` | Optional | Opens a prepared starter project when Alice project creation is not available. |
| `--timeout <seconds>` | Optional | Upper bound for the scenario run. Use `900` or higher for real desktop runs. |
| `--offline-package` | Optional | Packages Alice with Maven offline mode before running. |
| `--no-memory` | Optional | Disables memory writes for the run. |

Evidence artifacts must use paths relative to the run evidence directory. Any
artifact path that is absolute, escapes with `..`, points through a symlink
escape, or resolves outside the run directory is rejected.

## Command contract

Run the scenario:

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

The command exits successfully only when every required workflow phase is
accepted. It exits with failure when the run launches Alice but does not prove
the full workflow.

Required phases:

| Phase id | Required proof |
| --- | --- |
| `project-open` | A new project was created, or the configured starter project was opened. |
| `object-placement` | A named visible object was added to the world. |
| `object-transform` | The object position, rotation, or scale changed from its initial state. |
| `procedure-edit` | A procedure contains movement for the named object. |
| `run-world` | The world ran after the movement edit and produced visible run evidence. |
| `project-save` | The edited project was saved under the current run evidence directory. |
| `project-reopen` | The saved project artifact was reopened, not the starter project. |
| `persisted-state` | The reopened project still contains the object, transform, and movement procedure. |

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
  "source_saved_project_artifact": "../project-save/saved-project.a3p"
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
| `typescript_prototype` | Comparable prototype result when the prototype check is requested. |
| `product_issue_candidates[]` | Sanitized issue-ready summaries for claimed product behavior that blocked the workflow. |
| `failure_category` | `null` on pass; otherwise the blocked or invalid phase category. |

Consumers must use `workflow_phases[]`, `persisted_state`, and
`failure_category` instead of inferring success from process startup or
screenshots alone.

## Alice-side hook API

Eatme invokes deterministic hooks from the Alice checkout when the desktop
workflow reaches each phase. Hook paths are relative to `ALICE_HOME`.

### Project create/open

```bash
tools/eatme-open-project \
  --scenario alice-objects-first-world \
  --starter-project core/resources/target/distribution/application/starter-projects/africa.a3p \
  --evidence-dir runs/alice-objects-first-world/local/project-open \
  --json
```

The hook prints `eatme.alice-project-open/v1` JSON and writes the opened or
created project artifact under the evidence directory.

### Object placement

```bash
tools/eatme-place-object \
  --project runs/alice-objects-first-world/local/project-open/opened-project.a3p \
  --object-id bunny \
  --evidence-dir runs/alice-objects-first-world/local/object-placement \
  --json
```

The hook must prove that the object is visible and connected to the opened
project.

### Object transform

```bash
tools/eatme-transform-object \
  --project runs/alice-objects-first-world/local/object-placement/placed-object-project.a3p \
  --object-id bunny \
  --move-x 1.0 \
  --rotate-y 15 \
  --evidence-dir runs/alice-objects-first-world/local/object-transform \
  --json
```

The hook must record before and after transform values for the same object.

### Procedure edit

```bash
tools/eatme-edit-procedure \
  --project runs/alice-objects-first-world/local/object-transform/transformed-object-project.a3p \
  --procedure world.myFirstMethod \
  --object-id bunny \
  --movement "move forward 1 meter" \
  --evidence-dir runs/alice-objects-first-world/local/procedure-edit \
  --json
```

The hook must record executable movement intent. Placeholder text, comments, or
unattached procedure notes are rejected.

### Run world

```bash
tools/eatme-run-world \
  --project runs/alice-objects-first-world/local/procedure-edit/edited-project.a3p \
  --object-id bunny \
  --evidence-dir runs/alice-objects-first-world/local/run-world \
  --json
```

The hook must record that the world ran after the procedure edit and that the
named object movement was observed or captured through accepted evidence.

### Save project

```bash
tools/eatme-save-project \
  --project runs/alice-objects-first-world/local/procedure-edit/edited-project.a3p \
  --evidence-dir runs/alice-objects-first-world/local/project-save \
  --json
```

The hook must write `saved-project.a3p` under `project-save/`.

### Reopen project

```bash
tools/eatme-reopen-project \
  --saved-project runs/alice-objects-first-world/local/project-save/saved-project.a3p \
  --object-id bunny \
  --evidence-dir runs/alice-objects-first-world/local/project-reopen \
  --json
```

The hook must reopen the saved project artifact and write
`persisted-state.json`. The persisted-state proof must reference the same saved
project artifact from the same run.

## Rust API

The workflow coordinator is exposed by `eatme-alice`:

```rust
use eatme_alice::objects_first_workflow::{
    run_objects_first_workflow, ObjectsFirstWorkflowOptions,
};

let report = run_objects_first_workflow(ObjectsFirstWorkflowOptions {
    alice_home: "/home/azureuser/src/alice".into(),
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

The coordinator runs phases in order and stops on the first missing prerequisite.
It does not mark later phases as passed when an earlier phase is blocked.

## TypeScript prototype adapter

The prototype adapter validates comparable behavior against the TypeScript port
when the port exposes the needed feature.

```bash
export NODE_OPTIONS=--max-old-space-size=32768
export ALICE_WEB_PROTOTYPE_HOME=/home/azureuser/src/alice-web-prototype

cargo test -p eatme-alice --test ts_prototype_adapter -- --ignored
```

Adapter outputs use these states:

| State | Meaning |
| --- | --- |
| `present` | The prototype claims and proves the comparable behavior. |
| `unsupported` | The prototype does not claim the behavior. This is documented, not filed as a product bug. |
| `blocked` | The prototype claims the behavior, but the adapter cannot collect valid evidence. |
| `invalid` | The prototype returned malformed, unsafe, or out-of-run evidence. |

Reopen and transform gaps are product issues only when the TypeScript port
claims those behaviors. Otherwise they remain explicit unsupported gaps in the
adapter result.

## Path and issue safety

The workflow rejects evidence that escapes the run directory. It also rejects
evidence that contains secrets, browser/session credentials, SSH keys, raw
environment dumps, or unrelated desktop captures.

When a product bug blocks the workflow, issue-ready output includes only:

- scenario id;
- run id;
- blocked phase;
- target name (`RabbitHole Alice` or `TypeScript prototype`);
- display-safe summary;
- expected behavior;
- observed behavior;
- sanitized artifact names, not raw artifact contents.

Do not attach local evidence directories, raw screenshots, logs with secrets, or
absolute machine paths to product issues.
