# Alice lesson smoke lanes

Eatme lesson smoke lanes are editable, scenario-labeled checks that run through
the real Alice launch smoke harness. A lesson lane does not introduce its own
launcher. It passes a scenario id to the same packaging, Xvfb, Java process,
screenshot, log, and manifest path used by the baseline launch smoke, then
records that lesson id in the run manifest.

The post-launch lesson lanes are:

```text
hour-of-code-studio-kickoff
building-a-scene-first-world
code-editor-first-run
reusable-methods-and-parameters
functions-as-questions-about-the-world
loops-and-conditionals-mini-challenge
events-collision-proximity-game
```

They are based on Alice.org lesson/tutorial resource families and prove that the
desktop harness can reach a smoke-ready Alice session for resource-grounded
lesson paths before
agentic instructor/student evaluation is trusted.

## What the lane verifies

Each lesson lane is a manifest-only, lesson-labeled launch smoke. It verifies
smoke readiness from deterministic harness evidence:

- Alice was launched through the existing `eatme-alice` launch smoke path.
- The manifest identifies `scenario_id` as the selected lesson lane, such as
  `hour-of-code-studio-kickoff`, `building-a-scene-first-world`,
  `code-editor-first-run`, or one of the expanded Alice.org-grounded lesson ids.
- The deterministic launch assertions pass: dependencies, X display, Alice
  process startup, startup screenshot, and fatal-log scan.
- Alice log and window-list files are captured as artifacts when available.
- A non-empty startup screenshot or captured window list proves visual startup
  evidence for lesson lanes. Screenshots are represented by the top-level
  `screenshot` manifest artifact when available.
- The run artifacts are stored under a scenario-specific run directory.

The lanes do not perform deep in-lesson UI automation. They intentionally stop
at launch-ready evidence so lesson smokes remain stable in normal developer and
CI environments. For the Hour of Code studio kickoff, learner-visible first-scene,
first-animation, evidence, and reflection expectations live in editable YAML as
agentic follow-on contracts; runtime smoke still stops at deterministic
launch-ready evidence.

## Scenario assets

Canonical lesson scenarios live under:

```text
assets/scenarios/eatme/
```

The canonical lesson lanes are defined by:

```text
assets/scenarios/eatme/hour-of-code-studio-kickoff.yaml
assets/scenarios/eatme/building-a-scene-first-world.yaml
assets/scenarios/eatme/code-editor-first-run.yaml
assets/scenarios/eatme/reusable-methods-and-parameters.yaml
assets/scenarios/eatme/functions-as-questions-about-the-world.yaml
assets/scenarios/eatme/loops-and-conditionals-mini-challenge.yaml
assets/scenarios/eatme/events-collision-proximity-game.yaml
```

These files are the editable design contracts for lesson smokes. Lesson copy,
resource links, smoke steps, expected evidence, timeouts, artifact paths, and
Gherkin-style acceptance criteria are edited in YAML rather than Rust tests.

Current runtime behavior is intentionally narrower: `alice launch-smoke` does
not load the YAML file. The `--scenario` value supplies the manifest
`scenario_id` and run-directory namespace; asset validation separately checks
that the YAML contract, including Hour of Code prompt/evidence fields, remains
well-formed.

Gadugi-compatible adapters live under:

```text
assets/scenarios/gadugi/
```

The gadugi adapters for these lanes are:

```text
assets/scenarios/gadugi/hour-of-code-studio-kickoff.yaml
assets/scenarios/gadugi/building-a-scene-first-world.yaml
assets/scenarios/gadugi/code-editor-first-run.yaml
assets/scenarios/gadugi/reusable-methods-and-parameters.yaml
assets/scenarios/gadugi/functions-as-questions-about-the-world.yaml
assets/scenarios/gadugi/loops-and-conditionals-mini-challenge.yaml
assets/scenarios/gadugi/events-collision-proximity-game.yaml
```

Gadugi lesson scenarios may invoke the eatme CLI and inspect manifest-level
evidence. They must not own or duplicate Alice runtime behavior such as Xvfb
management, Swing/Java launch details, screenshot capture, log capture, or
process lifecycle. The additional
`assets/scenarios/gadugi/validation-failure-exit-code.yaml` regression adapter
covers the asset-validation exit-code contract without launching Alice.

## Validate assets

Validate every committed persona and scenario asset:

```bash
cargo run -q -p eatme-cli -- assets validate --json
```

Validate only one lesson lane:

```bash
cargo run -q -p eatme-cli -- assets validate \
  --path assets/scenarios/eatme/building-a-scene-first-world.yaml \
  --json

cargo run -q -p eatme-cli -- assets validate \
  --path assets/scenarios/eatme/hour-of-code-studio-kickoff.yaml \
  --json

cargo run -q -p eatme-cli -- assets validate \
  --path assets/scenarios/eatme/code-editor-first-run.yaml \
  --json
```

Passing validation exits `0` and reports `"passed": true`. Validation failures
exit non-zero and report `"passed": false`; scenarios that intentionally
exercise malformed assets must expect the non-zero exit instead of command
success. Error messages include the asset path or scenario id and the field that
needs attention.

## Run the lesson smoke

Lesson-labeled Alice execution is explicit. Non-baseline scenarios refuse to run
unless `EATME_REAL_ALICE=1` is set.

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

For any other lesson lane, use the same command shape with one of the
scenario ids above and a matching descriptive run id.

`ALICE_HOME` must point at the Alice source checkout to package and launch. A
typical local value is:

```bash
export ALICE_HOME=/home/azureuser/src/alice3-modernization
```

The generic launch smoke still works without selecting the lesson lane:

```bash
EATME_REAL_ALICE=1 cargo run -q -p eatme-cli -- alice launch-smoke \
  --alice-home "${ALICE_HOME}" \
  --run-id local-real-alice-launch-smoke \
  --runs-dir runs \
  --timeout 900 \
  --json \
  --no-memory
```

When `--scenario` is omitted, the command uses the baseline
`real-alice-launch-smoke` scenario. The baseline scenario is the compatibility
path and does not enforce the `EATME_REAL_ALICE` gate in the CLI today, though
it still requires the same real desktop dependencies to pass.

## CLI reference

### `eatme alice launch-smoke`

Launches Alice through the real launch smoke harness and writes deterministic
evidence to a run manifest.

```bash
eatme alice launch-smoke \
  --alice-home <path> \
  --run-id <run-id> \
  [--scenario <scenario-id>] \
  [--runs-dir <path>] \
  [--timeout <seconds>] \
  [--json] \
  [--no-memory] \
  [--offline-package]
```

| Option | Description |
| --- | --- |
| `--alice-home <path>` | Alice checkout to package and launch. |
| `--run-id <run-id>` | Stable id for this run. Use a descriptive id for local or CI traces. |
| `--scenario <scenario-id>` | Scenario id to record in the manifest and run directory. Defaults to `real-alice-launch-smoke`; it does not load scenario YAML at runtime yet. |
| `--runs-dir <path>` | Root directory for run artifacts. Defaults to `runs`. |
| `--timeout <seconds>` | Maximum launch wait before the smoke fails. |
| `--json` | Accepted compatibility flag. Output is currently pretty JSON whether or not this flag is present. |
| `--no-memory` | Disable memory writes for the run. |
| `--offline-package` | Package Alice in offline mode before launch. |

### `eatme assets validate`

Validates editable assets before a smoke run.

```bash
eatme assets validate [--path <asset-path>] [--json]
```

| Option | Description |
| --- | --- |
| `--path <asset-path>` | Validate one asset file instead of the full committed asset set. |
| `--json` | Emit validation results as JSON. |

Exit code contract: successful validation exits `0`; schema or asset validation
failures exit non-zero while still printing the JSON validation report.

## Run artifacts

Lesson smoke artifacts are namespaced by scenario id:

```text
runs/building-a-scene-first-world/<run-id>/
|-- manifest.json
|-- alice.log
|-- xvfb.log
|-- window-list.txt
|-- home/
|-- prefs/
|-- tmp/
`-- screenshots/
    `-- startup.png
```

The code editor lane uses:

```text
runs/code-editor-first-run/<run-id>/
```

The baseline launch smoke uses:

```text
runs/real-alice-launch-smoke/<run-id>/
```

## Manifest reference

Every launch smoke manifest includes the launch evidence needed by eatme and
gadugi adapters. Important fields for lesson smoke consumers are:

| Field | Meaning |
| --- | --- |
| `schema_version` | Manifest schema version. |
| `scenario_id` | Scenario selected for the run, such as `hour-of-code-studio-kickoff`, `building-a-scene-first-world`, `code-editor-first-run`, `reusable-methods-and-parameters`, `functions-as-questions-about-the-world`, `loops-and-conditionals-mini-challenge`, or `events-collision-proximity-game`. |
| `run_id` | Caller-provided run id. |
| `alice_home` | Alice checkout used for packaging and launch. |
| `alice_git_commit` | Alice source commit when available. |
| `eatme_git_commit` | Eatme source commit when available. |
| `dependency_checks` | Host dependency check results. |
| `build_command` | Alice packaging command. |
| `build_exit_status` | Packaging result. |
| `launch_command` | Java launch command. |
| `display` | X display used by the run. |
| `xvfb_pid` | Xvfb process id. |
| `alice_pid` | Alice process id. |
| `timeout_seconds` | Launch timeout applied to the run. |
| `window_list.path` | Captured desktop window-list artifact path. |
| `window_list_error` | Window-list capture or metadata error, when present. |
| `screenshot.path` | Top-level startup screenshot artifact path. |
| `screenshot.size_bytes` | Startup screenshot size. |
| `screenshot.sha256` | Startup screenshot digest. |
| `screenshot_error` | Screenshot capture or metadata error, when present. |
| `log.path` | Alice log path. |
| `log.size_bytes` | Alice log size. |
| `log.sha256` | Alice log digest. |
| `log_error` | Alice log read or metadata error, when present. |
| `fatal_log_scan` | Fatal DISPLAY/OpenGL/Java pattern scan result. |
| `assertions` | Deterministic launch assertions. |
| `failure_category` | Failure classification, or `null` for a passing smoke. |

Consumers should treat `assertions` and `failure_category` as the source of
truth for smoke status. Gadugi adapters should not inspect desktop internals
outside the manifest and captured artifacts.

`startup_screenshot` is an assertion key under `assertions`, not a top-level
artifact field. The top-level screenshot artifact is named `screenshot`. Startup
visual evidence must be either a non-empty screenshot or a captured
Alice-specific window identity. The harness records screenshot and log read
errors in the manifest instead of treating missing artifacts as success.

## Scenario YAML reference

Eatme scenario assets use `eatme.scenario/v1`.

```yaml
schema_version: eatme.scenario/v1
id: building-a-scene-first-world
title: Building a Scene First World
kind: alice_lesson_smoke
owner: eatme
resource_basis:
  - name: Alice.org Building a Scene lesson family
    url: https://www.alice.org/resources/
purpose: >-
  Prove that the lesson-specific smoke lane launches through the same real
  Alice desktop harness as the baseline launch smoke.
launcher:
  command: alice launch-smoke
  scenario: building-a-scene-first-world
real_alice:
  gated_by: EATME_REAL_ALICE=1
capabilities:
  required:
    - rust-cli
    - java-21
    - maven
    - xvfb
  optional:
    - glxinfo
adapter:
  targets:
    - eatme-cli
    - gadugi-cli
smoke_ready:
  evidence:
    - manifest_assertions
    - captured_logs
    - screenshot_or_window_evidence
    - scenario_id
acceptance_criteria:
  - given: Alice launch smoke dependencies are available
    when: the building-a-scene-first-world scenario is launched through eatme
    then: the manifest identifies scenario_id building-a-scene-first-world
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
  If host graphics, Java, Maven, or the EATME_REAL_ALICE=1 gate are missing,
  fail loudly rather than substituting a mocked Alice runtime.
```

Validated fields:

| Field | Requirement |
| --- | --- |
| `schema_version` | Must be `eatme.scenario/v1`. |
| `id` | Stable scenario id matching the file and launcher scenario. |
| `title` | Human-readable lesson title. |
| `kind` | Scenario category, such as `alice_lesson_smoke`. |
| `owner` | Canonical owner, normally `eatme`. |
| `purpose` | Plain-language reason the lane exists. |
| `launcher.command` | Eatme CLI command family, normally `alice launch-smoke`. |
| `launcher.scenario` | Scenario id passed to `--scenario`. |
| `real_alice.gated_by` | Real Alice gate, `EATME_REAL_ALICE=1`. |
| `smoke_ready.evidence` | Evidence that defines smoke-ready state. |
| `acceptance_criteria` | Editable Given/When/Then checks where useful. |
| `steps` | Human- and agent-readable smoke steps. |
| `timeouts` | Scenario and launch timeout values. |
| `artifacts` | Expected manifest, screenshot, and log locations. |
| `unsupported_policy` | Behavior when prerequisites are unavailable. |

`kind`, `owner`, `real_alice.gated_by`, `smoke_ready.evidence`, and
`acceptance_criteria` are enforced for every `alice_lesson_smoke` asset. A
scenario must define a launcher or steps, route runtime through
`alice launch-smoke`, define `artifacts.manifest`,
`artifacts.screenshot`, and `artifacts.log`, and include at least one timeout.

Design-convention fields such as `resource_basis`, `capabilities`,
`persona_assets`, `studio_cycle`, and `adapter.targets` may appear in assets,
but the current validator does not deserialize or enforce them. Treat them as
editable documentation for humans and agents, not as runtime inputs.

## Configuration

| Variable | Required | Description |
| --- | --- | --- |
| `EATME_REAL_ALICE=1` | Yes for lesson-labeled real launch | Enables non-baseline lesson smoke scenarios. Without it, those scenarios fail fast. |
| `ALICE_HOME` | Yes for real launch | Alice checkout used by `--alice-home`. |
| `RUN_ID` | Optional | Convenience value used by scenario YAML and gadugi adapters. |
| `NODE_OPTIONS=--max-old-space-size=32768` | Optional | Preserved environment preference for Node-based wrappers or agent tooling; the Rust CLI does not require it. |

Host dependencies for real launch are the same as the baseline smoke: Java 21,
Maven, Xvfb, `xdpyinfo`, `wmctrl`, a screenshot tool, and OpenGL/Mesa support
for software rendering.

## Tutorial: local lesson smoke

1. Validate the editable assets:

   ```bash
   cargo run -q -p eatme-cli -- assets validate --json
   ```

2. Check desktop prerequisites:

   ```bash
   cargo run -q -p eatme-cli -- deps check --json
   ```

3. Run a lesson lane:

   ```bash
   export ALICE_HOME=/home/azureuser/src/alice3-modernization
   export SCENARIO_ID=building-a-scene-first-world
   export RUN_ID=local-${SCENARIO_ID}

   EATME_REAL_ALICE=1 cargo run -q -p eatme-cli -- alice launch-smoke \
     --alice-home "${ALICE_HOME}" \
     --scenario "${SCENARIO_ID}" \
     --run-id "${RUN_ID}" \
     --runs-dir runs \
     --timeout 900 \
     --json \
     --no-memory \
     --offline-package
   ```

4. Inspect the manifest:

   ```bash
   jq '.scenario_id, .failure_category, .assertions' \
     "runs/${SCENARIO_ID}/${RUN_ID}/manifest.json"
   ```

5. Inspect captured artifacts:

   ```bash
   ls -lh "runs/${SCENARIO_ID}/${RUN_ID}/alice.log"
   ls -lh "runs/${SCENARIO_ID}/${RUN_ID}/screenshots/startup.png"
   ```

The run is smoke-ready when `failure_category` is `null`, all deterministic
assertions pass, and the manifest points at a non-empty startup screenshot or
the lesson assertion has accepted captured window evidence. The Alice log is
captured for diagnosis when artifact metadata is available, but log
non-emptiness is not currently a separate assertion.

## Tutorial: gadugi adapter boundary

Use the gadugi assets when a gadugi runner needs to exercise a lane:

```text
assets/scenarios/gadugi/hour-of-code-studio-kickoff.yaml
assets/scenarios/gadugi/building-a-scene-first-world.yaml
assets/scenarios/gadugi/code-editor-first-run.yaml
```

The adapter performs three kinds of work:

1. Run `eatme assets validate --json` and expect exit `0` only when the output
   reports `"passed": true`.
2. Run `eatme deps check --json`.
3. Run `eatme alice launch-smoke --scenario <lesson-id>`.

The adapter asserts command success and manifest-level output such as
the selected `"scenario_id"`, `"failure_category": null`, startup screenshot or
window evidence, and passing assertions. It does not reimplement or configure
Alice launch internals.

Use `assets/scenarios/gadugi/validation-failure-exit-code.yaml` as the negative
counterpart: it creates a malformed scenario asset and expects
`eatme assets validate --path ...` to exit `1` with `"passed": false`.

The committed gadugi adapters avoid repository-specific absolute paths. Run
these scenarios from the checkout under test so asset validation counts the
assets in that checkout, including gadugi-only regression adapters.

## Testing expectations

Always-on tests cover asset/schema validation and fake/gated harness behavior
without requiring Alice to launch. Real Alice validation is the explicitly gated
CLI smoke command:

```bash
EATME_REAL_ALICE=1 cargo run -q -p eatme-cli -- alice launch-smoke \
  --alice-home /home/azureuser/src/alice3-modernization \
  --scenario hour-of-code-studio-kickoff \
  --run-id local-hour-of-code-studio-kickoff \
  --runs-dir runs \
  --timeout 900 \
  --json \
  --no-memory \
  --offline-package
```

Normal workspace validation does not require real Alice:

```bash
cargo test --all-targets --all-features
```

The lesson lane is complete when committed scenario assets validate, malformed
scenario fixtures fail with actionable messages, the fake harness proves the
scenario id is routed through the existing launch smoke path, and the gated real
Alice command produces distinct scenario-namespaced artifacts when the host
supports desktop launch.
