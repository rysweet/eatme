# Alice Objects-First Full Path Reference

This reference defines the command, scenario asset, configuration, evidence
manifest, hook API, Rust integration surface, and validation rules for
`alice-objects-first-full-path`.

For step-by-step usage, see
[Alice Objects-First Full Path](alice-objects-first-full-path.md).

## Contents

- [Scenario identity](#scenario-identity)
- [CLI contract](#cli-contract)
- [Configuration](#configuration)
- [Full-path contract](#full-path-contract)
- [Evidence manifest](#evidence-manifest)
- [Required artifacts](#required-artifacts)
- [Alice-side hook API](#alice-side-hook-api)
- [Movement procedure contract](#movement-procedure-contract)
- [Save and reopen persistence assertions](#save-and-reopen-persistence-assertions)
- [Rust integration surface](#rust-integration-surface)
- [Gadugi adapter contract](#gadugi-adapter-contract)
- [External validation states](#external-validation-states)
- [Path, output, and issue safety](#path-output-and-issue-safety)

## Scenario identity

| Field | Value |
| --- | --- |
| Scenario id | `alice-objects-first-full-path` |
| Title | Alice Objects-First Full Path |
| Kind | `alice_objects_first_full_path` |
| Canonical source | `assets/scenarios/eatme/alice-objects-first-full-path.yaml` |
| Generated adapter | `assets/scenarios/gadugi/alice-objects-first-full-path.yaml` |
| Primary command | `eatme alice objects-first-full-path` |
| Primary runner | `eatme_alice::run_launch_smoke` with `LaunchSmokeScenario::new("alice-objects-first-full-path")` |
| Primary Alice target | `$ALICE_HOME` |
| Comparable prototype target | `$ALICE_WEB_PROTOTYPE_HOME` |

The canonical eatme scenario owns the ordered phase list, expected hook names,
required evidence, acceptance criteria, artifact names, and unsupported behavior
policy. The Gadugi adapter is generated from the canonical scenario and is not
hand-edited for mission intent.

## CLI contract

Installed binary:

```bash
eatme alice objects-first-full-path \
  --alice-home "$ALICE_HOME" \
  --run-id local-alice-objects-first-full-path \
  --runs-dir runs \
  --timeout 900 \
  --json \
  --no-memory \
  --offline-package
```

Source checkout:

```bash
cargo run -q -p eatme-cli -- alice objects-first-full-path \
  --alice-home "$ALICE_HOME" \
  --run-id local-alice-objects-first-full-path \
  --runs-dir runs \
  --timeout 900 \
  --json \
  --no-memory \
  --offline-package
```

Options:

| Option | Required | Description |
| --- | --- | --- |
| `--alice-home <path>` | Yes | Alice checkout. May also come from `ALICE_HOME`. |
| `--run-id <id>` | Yes | Stable evidence directory name for this run. |
| `--runs-dir <path>` | No | Root for run artifacts. Defaults to `runs`. |
| `--starter-project <path>` | No | Project to open when new-project creation is unavailable. Relative paths resolve from `--alice-home`. |
| `--timeout <seconds>` | No | Full-path timeout. Use `900` or higher for real desktop runs. |
| `--json` | No | Print machine-readable command output. |
| `--no-memory` | No | Disable memory writes for the run. |
| `--offline-package` | No | Package Alice with Maven offline mode before execution. |

Exit behavior:

| Exit | Meaning |
| --- | --- |
| `0` | Every required phase and persistence assertion passed. |
| non-zero | A prerequisite, hook, artifact, phase, external validation, or persistence assertion failed or blocked the full path. |

`alice launch-smoke --scenario alice-objects-first-full-path` is not the public
entrypoint. The discoverable command is `alice objects-first-full-path`, and it
binds the canonical scenario automatically.

## Configuration

| Name | Required | Contract |
| --- | --- | --- |
| `EATME_REAL_ALICE=1` | Yes | Enables real Alice execution for this non-baseline scenario. |
| `ALICE_HOME=/path/to/RabbitHole` | Yes unless `--alice-home` is set | RabbitHole Alice checkout. |
| `NODE_OPTIONS=--max-old-space-size=32768` | For Node-backed checks/adapters | Required for generated Gadugi adapters, Node-backed asset checks, or surrounding Node-based agent tooling; not required by the Rust command itself. |
| `ALICE_WEB_PROTOTYPE_HOME=/path/to/alice-web-prototype` | For TypeScript validation | TypeScript prototype checkout. |
| `GH_TOKEN` or existing `gh` auth | For issue filing | Used only to file sanitized product-gap issues when product support is claimed but invalid. |

The command records the effective configuration in `manifest.json` after
redacting tokens, credentials, environment dumps, and unrelated host paths.

## Full-path contract

The run writes a versioned top-level contract:

```json
{
  "schema_version": "eatme.alice-objects-first-full-path/v1",
  "scenario_id": "alice-objects-first-full-path",
  "status": "passed",
  "workflow_phases": [
    {"id": "project-open", "status": "passed"},
    {"id": "object-placement", "status": "passed"},
    {"id": "object-transform", "status": "passed"},
    {"id": "procedure-edit", "status": "passed"},
    {"id": "run-world", "status": "passed"},
    {"id": "project-save", "status": "passed"},
    {"id": "project-reopen", "status": "passed"},
    {"id": "persistence-assertions", "status": "passed"}
  ]
}
```

Accepted phase ids and required proof:

| Phase id | Required proof |
| --- | --- |
| `project-open` | A new project was created when deterministic creation is available, or the configured starter project was opened through Alice launch infrastructure when creation is unavailable. The manifest records the selected mode. |
| `object-placement` | A named object was added and visible in the world. |
| `object-transform` | The same object has changed position, rotation, or scale. |
| `procedure-edit` | A procedure contains executable movement for the same object. |
| `run-world` | The world ran after the movement edit and produced accepted run evidence. |
| `project-save` | The edited project was saved as a non-empty `.a3p` under the run directory. |
| `project-reopen` | The saved `.a3p` from the same run was reopened, not the starter project or a fresh project. |
| `persistence-assertions` | Reopened object, transform, movement procedure, saved artifact, and run evidence match the pre-reopen state. |

Later phases are not marked passed when an earlier phase is blocked.

## Evidence manifest

`manifest.json` uses the standard Alice run manifest plus the full-path section:

| Field | Meaning |
| --- | --- |
| `schema_version` | Manifest schema version. |
| `command.argv` | Redacted command invocation. Must include `alice objects-first-full-path`. |
| `scenario_id` | Must equal `alice-objects-first-full-path`. |
| `run_id` | Caller-supplied run id. |
| `alice_home` | Redacted Alice target metadata. |
| `scenario.path` | Relative path to the canonical scenario copy in the run directory. |
| `logs` | Relative paths and digests for stdout, stderr, Alice log, and display logs. |
| `objects_first_full_path` | Versioned full-path contract and typed phase results. |
| `workflow_phases[]` | Ordered phase summaries for consumers that read the generic manifest surface. |
| `project_state.before_save` | Relative path to pre-save project-state evidence. |
| `project_state.after_save` | Relative path to post-save project-state evidence. |
| `reopen_verification.path` | Relative path to reopen verification evidence. |
| `persistence_assertions.path` | Relative path to final assertion evidence. |
| `screenshots[]` | Optional screenshot or snapshot artifacts with explicit `present` or `unavailable` status. |
| `external_validation` | RabbitHole and TypeScript validation results. |
| `product_issue_candidates[]` | Sanitized issue-ready product gaps. |
| `follow_up_workstreams[]` | Follow-up default-workflow workstreams started for product gaps. |
| `failure_category` | `null` on pass; otherwise the blocked or invalid phase category. |

Consumers must use `objects_first_full_path`, `persistence_assertions`, and
`failure_category` as the source of truth. Logs, stdout, stderr, and screenshots
are supporting evidence only.

## Required artifacts

All artifact paths are relative to:

```text
runs/alice-objects-first-full-path/<run-id>/
```

| Artifact | Schema version | Required contents |
| --- | --- | --- |
| `command.json` | `eatme.command-invocation/v1` | Redacted argv, cwd label, selected scenario, and run id. |
| `scenario.yaml` | `eatme.scenario/v1` | Copy of the canonical scenario used for the run. |
| `project-open/project-open.json` | `eatme.alice-project-open/v1` | Project mode, title, and opened project artifact. |
| `project-open/opened-project.a3p` | n/a | Non-empty project artifact created or opened by the project-open phase. |
| `object-placement/object-placement.json` | `eatme.alice-object-placement-result/v1` | Object id, label, visibility proof, and project artifact after placement. |
| `object-placement/placed-project.a3p` | n/a | Non-empty project artifact after visible object placement. |
| `object-transform/object-transform.json` | `eatme.alice-object-transform/v1` | Object id, before transform, after transform, and changed transform fields. |
| `object-transform/transformed-object-project.a3p` | n/a | Non-empty project artifact after the same object has a changed transform. |
| `procedure-edit/procedure-edit.json` | `eatme.alice-procedure-edit-result/v1` | Procedure id, target object id, movement operation, movement amount, and executable status. |
| `procedure-edit/edited-project.a3p` | n/a | Non-empty project artifact after executable movement is added to the procedure. |
| `run-world/run-world.json` | `eatme.alice-run-world-result/v1` | Run trigger, target object id, movement observation, and run artifact. |
| `project-save/project-save.json` | `eatme.alice-project-save-result/v1` | Save selector, source edited project, saved `.a3p`, and save artifact metadata. |
| `project-save/pre-save-project-state.json` | `eatme.alice-project-state/v1` | Object, transform, procedure, and run state immediately before save. |
| `project-save/post-save-project-state.json` | `eatme.alice-project-state/v1` | Saved-project state immediately after save. |
| `project-reopen/project-reopen.json` | `eatme.alice-project-reopen-result/v1` | Source saved project, reopened project artifact, and reopen selector. |
| `project-reopen/reopen-verification.json` | `eatme.alice-reopen-verification/v1` | Proof that reopen loaded the saved artifact from this run. |
| `project-reopen/persistence-assertions.json` | `eatme.alice-persistence-assertions/v1` | Final assertion list and pass/fail status. |

Optional screenshots or snapshots are recorded with explicit status. Missing
screenshot tooling is not fatal unless the scenario marks screenshots required.

## Alice-side hook API

Eatme accepts only the expected local hook names below. Hook paths are resolved
relative to `ALICE_HOME/tools/`; absolute hook paths from callers are rejected.

| Phase | Hook |
| --- | --- |
| Object placement source evidence | `eatme-place-object` |
| Movement procedure edit | `eatme-edit-procedure` |
| World run | `eatme-run-world` |
| Project save | `eatme-save-project` |
| Project reopen | `eatme-reopen-project` |

Project create/open uses the existing Alice launch infrastructure. Object
transform validation is implemented as an internal `launch_object_transform`
probe over structured project state; it does not expand the Alice hook allowlist.

### Object placement hook

```bash
tools/eatme-place-object \
  --project runs/alice-objects-first-full-path/local/project-open/opened-project.a3p \
  --object-id bunny \
  --object-label "Bunny" \
  --transform "move x 1.0; rotate y 15" \
  --evidence-dir runs/alice-objects-first-full-path/local/object-placement \
  --json
```

The hook prints `eatme.alice-object-placement-result/v1` JSON. It must identify a
visible object, write an updated project artifact, and provide enough state for
eatme to derive the `object-transform` phase.

The internal object-transform probe consumes `object-placement/placed-project.a3p`
and writes `object-transform/object-transform.json` plus
`object-transform/transformed-object-project.a3p`.

### Procedure edit hook

```bash
tools/eatme-edit-procedure \
  --project runs/alice-objects-first-full-path/local/object-transform/transformed-object-project.a3p \
  --procedure world.myFirstMethod \
  --object-id bunny \
  --movement-operation move \
  --movement-direction forward \
  --movement-amount 1.0 \
  --movement-units meters \
  --evidence-dir runs/alice-objects-first-full-path/local/procedure-edit \
  --json
```

The hook prints `eatme.alice-procedure-edit-result/v1` JSON and writes
`edited-project.a3p`.

### Run-world hook

```bash
tools/eatme-run-world \
  --project runs/alice-objects-first-full-path/local/procedure-edit/edited-project.a3p \
  --object-id bunny \
  --procedure world.myFirstMethod \
  --evidence-dir runs/alice-objects-first-full-path/local/run-world \
  --json
```

The hook prints `eatme.alice-run-world-result/v1` JSON and records that the
world ran after the accepted movement edit.

### Save hook

```bash
tools/eatme-save-project \
  --project runs/alice-objects-first-full-path/local/procedure-edit/edited-project.a3p \
  --save-selector alice.save-project-default \
  --evidence-dir runs/alice-objects-first-full-path/local/project-save \
  --json
```

The hook prints `eatme.alice-project-save-result/v1` JSON and writes
`saved-project.a3p`. `save_selector` names the save affordance or backend path to
exercise; it is not a procedure id.

### Reopen hook

```bash
tools/eatme-reopen-project \
  --saved-project runs/alice-objects-first-full-path/local/project-save/saved-project.a3p \
  --reopen-selector alice.open-saved-project-default \
  --object-id bunny \
  --evidence-dir runs/alice-objects-first-full-path/local/project-reopen \
  --json
```

The hook prints `eatme.alice-project-reopen-result/v1` JSON and writes
`reopened-project.a3p`. The reopen result must point to the same saved project
artifact produced by the save hook in the same run. `reopen_selector` names the
open/reopen affordance or backend path to exercise; it is not a procedure id.

## Movement procedure contract

`procedure-edit/procedure-edit.json` must prove executable movement:

```json
{
  "schema_version": "eatme.alice-procedure-edit-result/v1",
  "status": "edited",
  "procedure_id": "world.myFirstMethod",
  "target_object_id": "bunny",
  "movement_operation": "move",
  "movement_direction": "forward",
  "movement_amount": 1.0,
  "movement_units": "meters",
  "executable": true,
  "edited_project_artifact": "edited-project.a3p"
}
```

Validation rules:

| Field | Rule |
| --- | --- |
| `target_object_id` | Must match the placed and transformed object id. |
| `movement_operation` | Must name executable movement such as `move`, `turn`, or `roll`. |
| `movement_amount` | Must be numeric and non-zero. |
| `executable` | Must be `true`. |
| `edited_project_artifact` | Must resolve under `procedure-edit/` and be non-empty. |

Comment-only edits, placeholder procedure notes, unattached code blocks, zero
movement, and movement for a different object are rejected.

## Save and reopen persistence assertions

`project-reopen/persistence-assertions.json` is the final acceptance artifact:

```json
{
  "schema_version": "eatme.alice-persistence-assertions/v1",
  "status": "passed",
  "scenario_id": "alice-objects-first-full-path",
  "source_saved_project_artifact": "project-save/saved-project.a3p",
  "assertions": [
    {"id": "same-saved-project-reopened", "status": "passed"},
    {"id": "object-identity-persisted", "status": "passed"},
    {"id": "object-transform-persisted", "status": "passed"},
    {"id": "movement-procedure-persisted", "status": "passed"},
    {"id": "run-world-completion-recorded", "status": "passed"}
  ]
}
```

Required assertions:

| Assertion id | Acceptance rule |
| --- | --- |
| `same-saved-project-reopened` | Reopen source resolves to the same canonical file as `project-save/saved-project.a3p`. |
| `object-identity-persisted` | Reopened project contains the same object id and label. |
| `object-transform-persisted` | Reopened transform matches the post-save transform state. |
| `movement-procedure-persisted` | Reopened procedure still contains executable movement for the same object. |
| `run-world-completion-recorded` | Run-world evidence exists and belongs to the saved project state. |

Any failed assertion makes the command fail.

## Rust integration surface

The public Rust surface for v1 remains the existing Alice launch orchestration.
The CLI command constructs the full-path scenario and delegates to
`run_launch_smoke`; no separate public full-path module is part of this
contract.

```rust
use eatme_alice::{run_launch_smoke, LaunchSmokeOptions, LaunchSmokeScenario};

let manifest = run_launch_smoke(&LaunchSmokeOptions {
    alice_home: "/path/to/RabbitHole".into(),
    run_id: "local-alice-objects-first-full-path".into(),
    runs_dir: "runs".into(),
    scenario: LaunchSmokeScenario::new("alice-objects-first-full-path"),
    timeout_seconds: 900,
    json: true,
    no_memory: true,
    offline_package: true,
})?;

assert_eq!(manifest.scenario_id, "alice-objects-first-full-path");
assert!(manifest.failure_category.is_none());
```

Public types reused by the command:

| Type | Purpose |
| --- | --- |
| `LaunchSmokeOptions` | Inputs for Alice home, run id, evidence root, timeout, package mode, memory mode, JSON output, and selected scenario. |
| `LaunchSmokeScenario` | Scenario id and optional starter-project override. |
| `LaunchSmokeManifest` | Top-level launch manifest extended with full-path artifacts and failure category. |

Internal implementation modules own the typed full-path work:
`launch_ui_action_contract`, `launch_object_placement`, `launch_object_transform`,
`launch_edit_procedure`, `launch_run_world`, `launch_save_project`, and
`launch_reopen_project`.

The command determines success from the `objects_first_full_path` manifest
section and persistence assertions, not from launch success, logs, screenshots,
or stale blocker classifications.

## Gadugi adapter contract

The generated adapter runs the same command shape and checks:

1. `assets validate` accepts the canonical scenario.
2. `assets generate-gadugi --check --json` confirms the adapter is current.
3. `alice objects-first-full-path` exits successfully.
4. `manifest.json` has `scenario_id: alice-objects-first-full-path`.
5. `objects_first_full_path.workflow_phases[]` are all `passed`.
6. `project-reopen/persistence-assertions.json` has `status: passed`.
7. `failure_category` is `null`.

The adapter must not substitute `alice launch-smoke`, hand-edit phase evidence,
or infer success from stdout alone.

## External validation states

External validation results use these states:

| State | Meaning |
| --- | --- |
| `present` | The target claims and proves the comparable behavior. |
| `unsupported` | The target does not claim the behavior; the gap is explicit and not treated as success. |
| `blocked` | The target claims the behavior but cannot produce valid evidence. |
| `invalid` | The target returned malformed, unsafe, oversized, or out-of-run evidence. |
| `issue_filed` | A sanitized product-gap issue was filed for claimed but broken support. |

RabbitHole Alice is the primary target and must prove the full path. The
TypeScript prototype records comparable behavior where supported. Claimed but
broken TypeScript support is surfaced as a product gap instead of being skipped.

## Path, output, and issue safety

Validation rejects:

- malformed JSON;
- oversized hook output;
- unexpected hook names;
- absolute artifact paths;
- parent traversal such as `..`;
- symlink escapes from the run directory;
- empty required artifacts;
- artifact paths outside `runs/alice-objects-first-full-path/<run-id>/`;
- phase success without required structured evidence.

Logs, manifests, issue bodies, and screenshot metadata redact secrets, tokens,
credentials, environment values, SSH keys, browser/session data, and sensitive
host paths.

Product-gap issue bodies include only scenario id, run id, target, phase,
expected behavior, observed behavior, and sanitized artifact names. Raw local
evidence directories, screenshots, logs, and absolute machine paths are not
attached.
