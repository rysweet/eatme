# Alice Objects-First Full Path

`alice-objects-first-full-path` is the executable Alice workflow that proves a
complete objects-first user path without manual intervention.

Use this page to run the command and review the evidence it writes. For field
names, hook schemas, Rust types, and validation rules, see
[Alice Objects-First Full Path Reference](alice-objects-first-full-path-reference.md).

## Contents

- [What the command does](#what-the-command-does)
- [Quick start](#quick-start)
- [Configuration](#configuration)
- [Run the full path](#run-the-full-path)
- [Review the evidence](#review-the-evidence)
- [Tutorial: verify persistence from a run](#tutorial-verify-persistence-from-a-run)
- [External Alice validation](#external-alice-validation)
- [Blocked product support](#blocked-product-support)

## What the command does

The command drives the real Alice target through this ordered path:

1. Create a new project when deterministic creation is available, or open the
   configured starter project when creation is unavailable.
2. Add one named visible object.
3. Transform that object so its state changes before the world runs.
4. Edit a movement procedure that references the same object.
5. Run the world and record that the object movement was exercised.
6. Save the project under the run evidence directory.
7. Reopen the saved project artifact.
8. Verify that the reopened project still contains the object, transform,
   movement procedure, and prior run evidence.

Opening Alice or producing launch logs is not enough. The run passes only when
the final persistence assertions pass.

## Quick start

Run from the eatme repository root:

```bash
export NODE_OPTIONS=--max-old-space-size=32768
export ALICE_HOME=/path/to/RabbitHole
export LOOKINGGLASS_HOME=/absolute/path/to/LookingGlass

EATME_REAL_ALICE=1 eatme alice objects-first-full-path \
  --alice-home "${ALICE_HOME}" \
  --run-id local-alice-objects-first-full-path \
  --runs-dir runs \
  --timeout 900 \
  --json \
  --no-memory \
  --offline-package
```

When running from source, use the Cargo wrapper:

```bash
EATME_REAL_ALICE=1 cargo run -q -p eatme-cli -- alice objects-first-full-path \
  --alice-home "${ALICE_HOME}" \
  --run-id local-alice-objects-first-full-path \
  --runs-dir runs \
  --timeout 900 \
  --json \
  --no-memory \
  --offline-package
```

The command uses the canonical scenario:

```text
assets/scenarios/eatme/alice-objects-first-full-path.yaml
```

The generated Gadugi adapter is:

```text
assets/scenarios/gadugi/alice-objects-first-full-path.yaml
```

## Configuration

| Setting | Value |
| --- | --- |
| Command | `eatme alice objects-first-full-path` |
| Scenario id | `alice-objects-first-full-path` |
| Canonical scenario asset | `assets/scenarios/eatme/alice-objects-first-full-path.yaml` |
| Generated Gadugi adapter | `assets/scenarios/gadugi/alice-objects-first-full-path.yaml` |
| Primary Alice target | `$ALICE_HOME` |
| LookingGlass target | `$LOOKINGGLASS_HOME` |
| Required real-Alice gate | `EATME_REAL_ALICE=1` |
| Node-backed adapter/check memory setting | `NODE_OPTIONS=--max-old-space-size=32768` |
| Evidence root | `runs/alice-objects-first-full-path/<run-id>/` |

Every path written to the manifest is relative to the run evidence directory.
Absolute paths, parent traversal, symlink escapes, and artifacts outside the run
directory are rejected.

Set `NODE_OPTIONS` when the run is wrapped by Node-backed checks, generated
Gadugi adapters, or agent tooling. The Rust command itself does not require Node.

## Run the full path

Validate the scenario asset before a run:

```bash
cargo run -q -p eatme-cli -- assets validate \
  --path assets/scenarios/eatme/alice-objects-first-full-path.yaml \
  --json
```

Check that the generated Gadugi adapter is current:

```bash
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
```

Run the full path against RabbitHole Alice:

```bash
export NODE_OPTIONS=--max-old-space-size=32768
export ALICE_HOME=/path/to/RabbitHole

EATME_REAL_ALICE=1 cargo run -q -p eatme-cli -- alice objects-first-full-path \
  --alice-home "${ALICE_HOME}" \
  --run-id local-alice-objects-first-full-path \
  --runs-dir runs \
  --timeout 900 \
  --json \
  --no-memory \
  --offline-package
```

Open a prepared starter project instead of creating a new project:

```bash
EATME_REAL_ALICE=1 cargo run -q -p eatme-cli -- alice objects-first-full-path \
  --alice-home "${ALICE_HOME}" \
  --starter-project core/resources/target/distribution/application/starter-projects/africa.a3p \
  --run-id local-alice-objects-first-full-path-starter \
  --runs-dir runs \
  --timeout 900 \
  --json \
  --no-memory \
  --offline-package
```

The manifest records whether the project was created or opened from a starter.
Both modes must continue through object placement, transform, movement procedure,
run-world, save, reopen, and persistence verification.

## Review the evidence

A complete run writes this layout:

```text
runs/alice-objects-first-full-path/local-alice-objects-first-full-path/
├── manifest.json
├── command.json
├── scenario.yaml
├── alice.log
├── xvfb.log
├── output/
│   ├── stdout.txt
│   └── stderr.txt
├── screenshots/
│   ├── object-visible.png
│   ├── object-transformed.png
│   └── world-ran.png
├── project-open/
│   ├── project-open.json
│   └── opened-project.a3p
├── object-placement/
│   ├── object-placement.json
│   └── placed-project.a3p
├── object-transform/
│   ├── object-transform.json
│   └── transformed-object-project.a3p
├── procedure-edit/
│   ├── procedure-edit.json
│   └── edited-project.a3p
├── run-world/
│   └── run-world.json
├── project-save/
│   ├── project-save.json
│   ├── pre-save-project-state.json
│   ├── post-save-project-state.json
│   └── saved-project.a3p
└── project-reopen/
    ├── project-reopen.json
    ├── reopened-project.a3p
    ├── reopen-verification.json
    └── persistence-assertions.json
```

Review the final status:

```bash
jq '{scenario_id, passed, failure_category}' \
  runs/alice-objects-first-full-path/local-alice-objects-first-full-path/manifest.json
```

Review the ordered phase results:

```bash
jq '.objects_first_full_path.workflow_phases[] | {id, status, artifact}' \
  runs/alice-objects-first-full-path/local-alice-objects-first-full-path/manifest.json
```

Review the persistence assertions:

```bash
jq '.assertions' \
  runs/alice-objects-first-full-path/local-alice-objects-first-full-path/project-reopen/persistence-assertions.json
```

The source of truth is the structured contract in `manifest.json`,
`project-save/*project-state.json`, `project-reopen/reopen-verification.json`,
and `project-reopen/persistence-assertions.json`. Logs and screenshots support
the evidence; they do not replace the structured assertions.

## Tutorial: verify persistence from a run

Use this tutorial after a local run completes.

### 1. Confirm the command and scenario

```bash
jq '{command: .command.argv, scenario_id}' \
  runs/alice-objects-first-full-path/local-alice-objects-first-full-path/manifest.json
```

Expected result:

```json
{
  "command": [
    "...",
    "alice",
    "objects-first-full-path",
    "..."
  ],
  "scenario_id": "alice-objects-first-full-path"
}
```

The command array must include `alice objects-first-full-path`; extra wrapper
arguments are allowed when running through Cargo.

### 2. Confirm the same object appears in every phase

```bash
jq '{
  placed: .objects_first_full_path.object_placement.object_id,
  transformed: .objects_first_full_path.object_transform.object_id,
  procedure_target: .objects_first_full_path.procedure_edit.target_object_id,
  reopened: .objects_first_full_path.reopen_verification.object_id
}' \
  runs/alice-objects-first-full-path/local-alice-objects-first-full-path/manifest.json
```

All four values must match.

### 3. Confirm the movement procedure is executable

```bash
jq '{
  procedure_id,
  target_object_id,
  movement_operation,
  movement_amount,
  executable
}' \
  runs/alice-objects-first-full-path/local-alice-objects-first-full-path/procedure-edit/procedure-edit.json
```

`executable` must be `true`. Comment-only edits are rejected.

### 4. Confirm save and reopen used the same artifact

```bash
jq '{
  saved: .source_saved_project_artifact,
  reopened_from: .reopened_from_saved_project_artifact
}' \
  runs/alice-objects-first-full-path/local-alice-objects-first-full-path/project-reopen/reopen-verification.json
```

Both values must refer to the same `project-save/saved-project.a3p` artifact from
the current run.

### 5. Confirm persistence assertions passed

```bash
jq '.assertions[] | select(.status != "passed")' \
  runs/alice-objects-first-full-path/local-alice-objects-first-full-path/project-reopen/persistence-assertions.json
```

The command should print no rows. Any row means the full path did not pass.

## External Alice validation

The command validates applicable behavior against:

| Target | Path | Purpose |
| --- | --- | --- |
| RabbitHole Alice | `$ALICE_HOME` | Primary executable desktop target. |
| LookingGlass | `$LOOKINGGLASS_HOME` | Comparable web-port behavior where supported. |

RabbitHole validation is required for the full path. TypeScript validation records
`present`, `unsupported`, `blocked`, or `invalid` for comparable hooks and state
outputs. Unsupported behavior is explicit in the manifest; claimed behavior that
cannot produce valid evidence becomes a product-gap issue candidate.

## Blocked product support

Missing or malformed required support is not silently skipped. A blocked run
writes:

```text
manifest.json
product-issues/<target>/<phase>.json
follow-up-workstreams.json
```

Issue-ready entries include the scenario id, run id, target, blocked phase,
expected behavior, observed behavior, and sanitized artifact names. They do not
include raw screenshots, secrets, full environment dumps, credentials, or local
absolute paths.

When `gh` authentication is available, eatme files sanitized product-gap issues
for RabbitHole or TypeScript behavior that is claimed but incompatible with the
contract. The manifest records filed issue URLs and follow-up default-workflow
workstreams.
