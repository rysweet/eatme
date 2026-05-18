# Import/export workflow

The import/export workflow integration test validates the full `.a3p`
save/load/export round-trip against a real Alice desktop session. It exercises
the core Save→Open→Export silver thread: open a starter project, save it after
modification, reopen and verify persistence, then export to NetBeans project
format and verify the exported Ant `build.xml` exists.

The test is gated behind `EATME_REAL_ALICE=1` so CI and developer machines
without Alice desktop dependencies skip it automatically.

## Contents

- [Evidence boundary](#evidence-boundary)
- [Usage](#usage)
- [Environment gate](#environment-gate)
- [What the test proves](#what-the-test-proves)
- [Workflow phases](#workflow-phases)
- [Export hook API](#export-hook-api)
- [Evidence directories](#evidence-directories)
- [Configuration](#configuration)
- [Rust API](#rust-api)
- [Examples](#examples)
- [Troubleshooting](#troubleshooting)
- [Non-claims](#non-claims)
- [Related documentation](#related-documentation)

## Evidence boundary

The import/export workflow is layered on top of the save/reopen readiness
contract.

| Layer | What it proves | What it does not imply |
| --- | --- | --- |
| Launch smoke | Alice starts on a virtual display, process is alive, screenshot captured, no fatal logs. | Save, reopen, export, full UI automation, or lesson completion. |
| Save artifact proof | A deterministic save affordance produced a non-empty saved `.a3p` and save evidence. | Semantic project change, full Save completion, grading, creative assessment, or first-lesson completion. |
| Reopen artifact proof | A deterministic reopen affordance reopened the saved `.a3p` and state verification passed. | Visible rendering correctness, broad Alice compatibility, or creative quality. |
| Export artifact proof | A deterministic export affordance produced a NetBeans project and the Ant `build.xml` exists on disk. | Build correctness, NetBeans IDE compatibility, full project portability, or lesson completion. |

The readiness sequence is:

1. Open the bundled starter project through the Alice launch path.
2. Pass all 6 core launch-smoke assertions.
3. Save the edited project through the deterministic save hook.
4. Reopen the saved `.a3p` through the deterministic reopen hook.
5. Verify reopened state matches the bounded selector.
6. Export the saved project to NetBeans format through the deterministic export
   hook.
7. Verify the exported `build.xml` exists on disk in the export evidence
   directory.

Each phase depends on the previous phase. Export proof is blocked until accepted
reopen proof exists. Reopen proof is blocked until accepted save proof exists.

## Usage

Run the import/export workflow integration test:

```bash
EATME_REAL_ALICE=1 cargo test -p eatme-alice --test import_export_workflow_real -- --nocapture
```

The test is a standard Rust integration test in
`crates/eatme-alice/tests/import_export_workflow_real.rs`. When
`EATME_REAL_ALICE` is unset or not `1`, the test returns early with a skip
message and passes. No `#[ignore]` attribute is used — the runtime check
matches the CI workflow pattern used by real Alice integration jobs.

Run all `eatme-alice` tests (the real-Alice tests skip automatically when the
environment variable is absent):

```bash
cargo test -p eatme-alice
```

## Environment gate

| Variable | Required value | Effect |
| --- | --- | --- |
| `EATME_REAL_ALICE` | `1` | Enables the import/export workflow integration test. Any other value or absence causes the test to skip. |
| `ALICE_HOME` | Path to Alice checkout | The Alice checkout directory. Defaults to `/opt/alice3` when not set (matching the existing `launch_smoke_real` test). |

The gate is a runtime `std::env::var` check, not a compile-time `cfg`
attribute. This means:

- `cargo test -p eatme-alice` always compiles the test.
- The test binary always includes `import_export_workflow_real`.
- The test body returns early when the gate is not satisfied.
- CI workflows that set `EATME_REAL_ALICE=1` on self-hosted runners with Alice
  desktop dependencies get the full integration validation.

## What the test proves

The import/export workflow integration test exercises the complete
save→reopen→export path with a real Alice installation:

1. **Launch smoke** — all 6 core manifest assertions pass (dependencies
   available, display responsive, process started, startup screenshot, no fatal
   logs, real Alice execution evidence).
2. **Save persistence** — the save hook produces a non-empty saved `.a3p` and
   save evidence JSON, both validated under `project-save/`.
3. **Reopen persistence** — the reopen hook opens the saved `.a3p` (not the
   bundled starter), produces reopened-state evidence, and state verification
   passes for the bounded selector.
4. **Export to NetBeans** — the export hook accepts the saved `.a3p`, produces
   export evidence JSON with `status: "exported"` and schema version
   `eatme.alice-project-export-result/v1`, and writes a NetBeans project
   structure.
5. **Build file verification** — the exported `build.xml` exists on disk in the
   export evidence directory as a non-empty file.

The test uses a single Xvfb virtual display on a dynamically reserved port
(lock-file scan over `:90`–`:129`; the actual port depends on which displays
are already locked) for both the reopen and export phases.

## Workflow phases

### Phase 1: Launch smoke

The test calls `run_launch_smoke()` with the `first-lessons-real-ui-actions`
scenario and asserts all 6 core assertions pass:

```rust
assert!(manifest.failure_category.is_none());
assert!(manifest.assertions.values().all(|a| a.passed));
```

### Phase 2: Save project

The test parses `ui-action-contract.json` from the launch-smoke run to extract
the saved `.a3p` path from the save-project probe. The save probe must have
`proves_save() == true`.

### Phase 3: Reopen project

The test invokes `tools/eatme-reopen-project` with the saved `.a3p` as
`--saved-project`. It asserts:

- `status` is `reopened`
- `state_verification` is `passed`
- `source_saved_project_artifact` resolves to the same canonical file the save
  probe produced

### Phase 4: Export project

The test invokes `tools/eatme-export-project` with the saved `.a3p` as
`--saved-project` and `--export-format netbeans`. It asserts:

- JSON output matches `eatme.alice-project-export-result/v1` schema
- `status` is `exported`
- `export_format` is `netbeans`
- `exported_build_file` is a simple relative path under the export evidence
  directory

### Phase 5: Verify build.xml

The test resolves the `exported_build_file` path under `project-export/` and
asserts:

- The file exists on disk
- The file is non-empty

### Phase 6: Cleanup

Drop-based guards clean up the Xvfb process and temporary directories when the
test completes or panics.

## Export hook API

The export hook is an Alice-side contract. Eatme invokes it and validates its
JSON output and artifacts.

### Command shape

```bash
tools/eatme-export-project \
  --saved-project runs/first-lessons-real-ui-actions/ie-<nanos>/project-save/saved-project.a3p \
  --export-format netbeans \
  --evidence-dir runs/first-lessons-real-ui-actions/ie-<nanos>/project-export \
  --json
```

### JSON output

The hook prints:

```json
{
  "schema_version": "eatme.alice-project-export-result/v1",
  "status": "exported",
  "export_format": "netbeans",
  "source_saved_project_artifact": "project-save/saved-project.a3p",
  "exported_build_file": "build.xml",
  "export_artifact": "project-export.json"
}
```

### Validation rules

| Field | Rule |
| --- | --- |
| `schema_version` | Must be `eatme.alice-project-export-result/v1`. |
| `status` | Must be `exported`. |
| `export_format` | Must be `netbeans`. |
| `source_saved_project_artifact` | Must be a simple relative path starting with `project-save/`, must resolve under the run's `project-save/` evidence directory to the same saved artifact from the save phase, and must not point to the bundled starter project. |
| `exported_build_file` | Must be a simple relative path under `project-export/` and must point to a non-empty file on disk. This is the Ant `build.xml` that NetBeans uses. |
| `export_artifact` | Must be a simple relative path under `project-export/` and must point to a non-empty file. |

Absolute paths, parent traversal, symlink escapes, empty files, malformed JSON,
wrong schema versions, and artifacts outside the expected run evidence
directories are not accepted as proof.

### Hook timeout

The export hook has a 60-second timeout (2× the save/reopen hook timeout of
30 seconds) because file generation for the NetBeans project structure may be
heavier than save or reopen operations. If the hook does not complete within the
timeout, the child process is killed and the probe reports `blocked`.

## Evidence directories

A complete import/export evidence bundle uses this layout under the run
directory:

```text
target/test-work/import-export-workflow-real/runs/
  first-lessons-real-ui-actions/ie-<nanos>/
  ├── manifest.json
  ├── alice.log
  ├── xvfb.log
  ├── screenshots/startup.png
  ├── procedure-edit/
  │   └── edited-project.a3p
  ├── project-save/
  │   ├── saved-project.a3p
  │   └── project-save.json
  ├── project-reopen/
  │   ├── reopened-project.a3p
  │   ├── project-reopen.json
  │   └── reopened-state.json
  ├── project-export/
  │   ├── build.xml
  │   └── project-export.json
  └── ui-action-contract.json
```

The `project-export/` directory is the export evidence lane. It is separate from
`project-save/` and `project-reopen/` to maintain clear evidence boundaries
between the save, reopen, and export phases.

## Configuration

### Integration test options

The import/export workflow test uses these options:

| Option | Value | Rationale |
| --- | --- | --- |
| `alice_home` | `ALICE_HOME` env var or `/opt/alice3` | Standard Alice checkout location (matches `launch_smoke_real`). |
| `scenario` | `first-lessons-real-ui-actions` | The first-lesson scenario with real UI action probes. |
| `run_id` | `ie-{nanos}` | Unique run id using nanosecond timestamp to prevent collisions. |
| `runs_dir` | `target/test-work/import-export-workflow-real/runs` | Isolated under `target/` to avoid polluting project root. |
| `display` | Dynamic (`:90`–`:129` range) | `reserve_display()` scans for an unlocked port via lock files. |
| `timeout_seconds` | `900` | 15-minute timeout for cold Maven builds and slow Java startup. |
| `export_timeout_seconds` | `60` | Export-specific timeout (2× save/reopen hook timeout). |
| `json` | `true` | Machine-readable output. |
| `no_memory` | `true` | No persistent memory side effects from test runs. |
| `offline_package` | `true` | Uses cached Maven dependencies, no network access. |

### Host requirements

The import/export workflow test requires the same host dependencies as the
[Deterministic Real-Alice Smoke Test](deterministic-real-alice-smoke-test.md)
plus the Alice export tooling:

| Dependency | Minimum | Purpose |
| --- | --- | --- |
| Java | 21 | Alice runtime |
| Maven | 3.9+ | Alice packaging |
| Xvfb | Any | Virtual X display |
| xdpyinfo | Any | Display readiness probe |
| wmctrl | Any | Window list capture |
| xwininfo | Any | Fallback window tree capture |
| xdotool | Any | Window activation |
| scrot or ImageMagick `import` | Any | Screenshot capture |
| Mesa/llvmpipe | Any | Software OpenGL rendering |

Install all dependencies on Ubuntu/Debian:

```bash
sudo apt-get install -y \
  openjdk-21-jdk maven \
  xvfb x11-utils wmctrl x11-xserver-utils xdotool \
  scrot imagemagick mesa-utils
```

## Rust API

### UiActionExportProjectProbe

Constructed by `probe_project_export_hook`. Fields:

| Field | Type | Purpose |
| --- | --- | --- |
| `id` | `String` | Always `alice-side-project-export-command-hook`. |
| `action_id` | `String` | Always `export-project`. |
| `status` | `String` | `passed`, `blocked`, or `failed`. |
| `detail` | `String` | Human-readable status explanation. |
| `export_format` | `String` | Always `netbeans` for this test. |
| `candidate_hook_path` | `String` | Resolved path to the export hook in the Alice checkout. |
| `command` | `Option<String>` | Full command line when the hook ran. |
| `exit_status` | `Option<i32>` | Exit code when the hook ran. |
| `stdout` | `String` | Hook stdout (expected JSON on success). |
| `stderr` | `String` | Hook stderr. |
| `source_saved_project_artifact` | `String` | Relative path starting with `project-save/` that must resolve to the same canonical artifact the save probe produced. |
| `exported_build_file` | `Option<ArtifactInfo>` | Validated Ant `build.xml` artifact info under `project-export/`. |
| `export_artifact` | `Option<ArtifactInfo>` | Validated export evidence JSON artifact info under `project-export/`. |
| `validation_errors` | `Vec<String>` | All validation failures, empty when `status` is `passed`. |
| `missing_affordance` | `Option<UiActionMissingAffordance>` | Present when `status` is `blocked` due to a missing hook or precondition. |

The `proves_export()` method returns `true` only when `status` is `passed`,
`source_saved_project_artifact` is non-empty, both artifact fields are `Some`,
and `validation_errors` is empty.

### ProjectExportHookResult

Typed deserialization struct for the `eatme.alice-project-export-result/v1`
JSON contract:

```rust
#[derive(Debug, Deserialize)]
pub(crate) struct ProjectExportHookResult {
    pub schema_version: String,
    pub status: String,
    pub export_format: String,
    pub source_saved_project_artifact: String,
    pub exported_build_file: String,
    pub export_artifact: String,
}
```

The struct uses `#[derive(Deserialize)]` only. No production code path
deserializes export results from external input; the derive exists solely for
the integration test.

## Examples

### Run the import/export workflow test on a self-hosted runner

```bash
export ALICE_HOME=/opt/alice3-modernization
EATME_REAL_ALICE=1 cargo test -p eatme-alice \
  --test import_export_workflow_real -- --nocapture
```

### Run all eatme-alice tests (import/export auto-skips)

```bash
cargo test -p eatme-alice
```

Output includes:

```text
test import_export_workflow_real::save_reopen_export_round_trip ... ok
```

When `EATME_REAL_ALICE` is not set, the test prints a skip message and passes
without exercising Alice.

### Inspect the export evidence after a real run

```bash
# Find the run directory
ls target/test-work/import-export-workflow-real/runs/first-lessons-real-ui-actions/

# Check export evidence
cat target/test-work/import-export-workflow-real/runs/first-lessons-real-ui-actions/ie-*/project-export/project-export.json \
  | jq '.'

# Verify build.xml exists
file target/test-work/import-export-workflow-real/runs/first-lessons-real-ui-actions/ie-*/project-export/build.xml
```

### Inspect the full workflow evidence chain

```bash
RUN_DIR=$(ls -d target/test-work/import-export-workflow-real/runs/first-lessons-real-ui-actions/ie-* | head -1)

# Phase 1: Launch assertions
jq '.assertions | to_entries[] | {key, passed: .value.passed}' "${RUN_DIR}/manifest.json"

# Phase 2-3: Save and reopen evidence
jq '.status' "${RUN_DIR}/project-save/project-save.json"
jq '.status, .state_verification' "${RUN_DIR}/project-reopen/project-reopen.json"

# Phase 4-5: Export evidence and build.xml
jq '.status, .export_format' "${RUN_DIR}/project-export/project-export.json"
wc -c "${RUN_DIR}/project-export/build.xml"
```

## Troubleshooting

### Test skips unexpectedly

Verify the environment variable is set to exactly `1`:

```bash
echo $EATME_REAL_ALICE   # should print: 1
```

The check is `std::env::var("EATME_REAL_ALICE") == Ok("1".into())`. Values
like `true`, `yes`, or empty string do not activate the test.

### Export hook is missing

If the Alice checkout does not have `tools/eatme-export-project`, the export
probe reports `blocked` with a `missing_affordance` detail. This is the
expected contract-first behavior: the test proves that the harness correctly
handles a missing export capability.

```text
Export probe status: blocked
Detail: Export hook tools/eatme-export-project not found in Alice checkout.
```

The save and reopen phases still execute and produce their own evidence. Only
the export phase is blocked.

### Display collision

The test reserves a virtual display in the `:90`–`:129` range using lock-file
scanning. If another eatme-alice real test (such as `launch_smoke_real`) is
running concurrently on the same machine, each test gets a different display.

If all ports are taken:

```bash
ls /tmp/.X*-lock   # check which displays are in use
```

### Export times out

The export hook has a 60-second timeout. If it exceeds this, the child process
is killed and the probe reports `blocked`. Check the Alice log and hook stderr:

```bash
RUN_DIR=$(ls -d target/test-work/import-export-workflow-real/runs/first-lessons-real-ui-actions/ie-* | head -1)
cat "${RUN_DIR}/alice.log" | tail -50
```

### Missing dependencies

Run the dependency check first:

```bash
cargo run -q -p eatme-cli -- deps check --json
```

### Unix socket path too long

In deep worktree paths, the X display socket path may exceed the 108-character
Unix socket limit. Use `TMPDIR=/tmp` to shorten the socket path:

```bash
TMPDIR=/tmp EATME_REAL_ALICE=1 cargo test -p eatme-alice \
  --test import_export_workflow_real
```

## Non-claims

This integration test does not claim:

- full Save completion beyond the bounded save-hook contract
- full reopen completion beyond the bounded artifact/state contract
- export correctness (that the NetBeans project compiles or runs)
- NetBeans IDE compatibility
- full project portability
- full UI automation
- first-lesson completion
- visible rendering correctness
- grading or creative assessment
- broad Alice compatibility
- deployed sharing or platform success

The test proves that the harness can invoke the save, reopen, and export hooks,
validate their JSON contracts, and confirm the expected artifacts exist on disk.
It does not prove that the exported project is semantically correct or usable in
NetBeans.

## Related documentation

- [Save/reopen Readiness](save-reopen-readiness.md) — Save/reopen evidence
  boundary, hook API, path validation, and readiness states.
- [Deterministic Real-Alice Smoke Test](deterministic-real-alice-smoke-test.md)
  — Launch smoke integration test documentation and manifest assertions.
- [Alice Integration](alice-integration.md) — CLI commands for discovery,
  packaging, and launch smoke.
- [Alice Lesson Smoke](alice-lesson-smoke.md) — Desktop scenario roster and
  evidence contracts.
- [Outside-in Alice Test Modules](outside-in-alice-test-modules.md) — Rust
  test module layout and authoring workflow.
- [Evidence Artifact Contract](evidence-artifact-contract.md) — Artifact
  schema and text contract.
- [Validation and Quality Gates](validation-quality-gates.md) — Repository
  quality gates.
