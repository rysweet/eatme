# Alice integration

Eatme integrates with a real Alice checkout through explicit CLI commands. The
integration discovers the checkout, checks host dependencies, packages Alice,
launches Alice through a virtual display, and writes deterministic evidence to a
run manifest.

## Configure `ALICE_HOME`

```bash
export ALICE_HOME="${ALICE_HOME:-../alice3-modernization}"
```

Every Alice command accepts `--alice-home`. The same value can be supplied
through the `ALICE_HOME` environment variable.

## Dependency check

```bash
cargo run -q -p eatme-cli -- deps check --json
```

The check covers the desktop and build tools needed by real launch smoke runs:

- Java 21
- Maven
- Xvfb
- `xdpyinfo`
- `wmctrl`
- `xwininfo`
- `xdotool`
- screenshot tooling
- GLX/Mesa software rendering support

`glxinfo` is reported when present, but it is diagnostic-only.

## Discover Alice

```bash
cargo run -q -p eatme-cli -- alice discover \
  --alice-home "${ALICE_HOME}" \
  --json
```

Discovery verifies that the configured Alice checkout has the expected shape
before packaging or launch commands depend on it.

## Package Alice

```bash
cargo run -q -p eatme-cli -- alice package \
  --alice-home "${ALICE_HOME}" \
  --offline \
  --json
```

Packaging delegates to the Alice Maven build. Use `--offline` when the Maven
cache already contains the required dependencies.

## Launch smoke

```bash
EATME_REAL_ALICE=1 cargo run -q -p eatme-cli -- alice launch-smoke \
  --alice-home "${ALICE_HOME}" \
  --scenario building-a-scene-first-world \
  --run-id local-building-a-scene-first-world \
  --runs-dir runs \
  --timeout 900 \
  --json \
  --no-memory \
  --offline-package
```

The launch smoke records evidence under:

```text
runs/<scenario-id>/<run-id>/
```

Typical artifacts include:

```text
manifest.json
alice.log
xvfb.log
window-list.txt
home/
prefs/
tmp/
screenshots/startup.png
```

## Manifest contract

The manifest is the integration contract for automation. Important fields are:

| Field | Meaning |
| --- | --- |
| `schema_version` | Manifest schema version |
| `scenario_id` | Scenario selected for the run |
| `run_id` | Caller-provided run id |
| `alice_home` | Alice checkout used for the run |
| `alice_git_commit` | Alice commit when available |
| `eatme_git_commit` | Eatme commit when available |
| `dependency_checks` | Host dependency results |
| `build_command` | Alice packaging command |
| `build_exit_status` | Packaging result |
| `launch_command` | Java launch command |
| `display` | X display used by the run |
| `xvfb_pid` | Xvfb process id |
| `alice_pid` | Alice process id |
| `timeout_seconds` | Launch timeout |
| `window_list.path` | Captured window-list artifact when available |
| `screenshot.path` | Startup screenshot artifact when available |
| `log.path` | Alice log path |
| `fatal_log_scan` | Fatal DISPLAY/OpenGL/Java scan result |
| `assertions` | Deterministic pass/fail assertions |
| `failure_category` | Failure classification, or `null` on pass |

Consumers should treat `assertions` and `failure_category` as the source of
truth. Gadugi adapters and external scripts should not infer pass/fail by
replaying Alice internals.

## Current scope

The real Alice integration proves launch readiness. It does not yet drive a full
lesson UI path, edit procedures, run the world, save projects, grade a student
world, or automate creative assessment. Readiness reports can consume Save
Project and Select Project proof-artifact declarations from RabbitHole evidence,
but those declarations report artifact availability only. Both categories remain
visible as `missing` when declarations are absent. Emitted proof-artifact paths
are evidence-root-relative summaries, artifact contents are never read or
emitted, and blocker details are normalized before reporting. The
`first-lessons-real-ui-actions` scenario now probes one deterministic
object-placement candidate:
`tools/eatme-place-object` inside the Alice checkout. That Alice-side command
must accept the opened project, named object identifier, and evidence directory,
then return JSON with non-empty `placement_artifact` and
`scene_or_project_diff` files before eatme marks object placement as proven.
Absent or invalid hook evidence remains an explicit blocked result.

The [import/export workflow](import-export-workflow.md) extends the save/reopen
contract with a deterministic export phase. After save and reopen succeed, the
`tools/eatme-export-project` hook exports the saved `.a3p` to NetBeans project
format and the test verifies the Ant `build.xml` exists on disk. The export hook
contract uses `eatme.alice-project-export-result/v1` JSON schema. A missing
export hook produces a bounded `blocked` result without failing the save/reopen
evidence.
