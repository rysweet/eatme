# Alice lesson and desktop preflight scenarios

Eatme desktop scenarios are editable, scenario-labeled contracts for student
outside-in Alice flows. Every scenario routes through the existing real Alice launch
smoke harness. A scenario does not introduce its own launcher; it passes a scenario
id to the same packaging, Xvfb, Java process, screenshot, log, and manifest path
used by the baseline launch smoke, then records that id in the run manifest.

## Committed desktop scenario roster

| Scenario | Role | Runtime contract |
| --- | --- | --- |
| `building-a-scene-first-world` | Alice.org lesson smoke | Launch Alice and record scenario-labeled manifest evidence. |
| `code-editor-first-run` | Alice.org lesson smoke | Launch Alice and record scenario-labeled manifest evidence. |
| `reusable-methods-and-parameters` | Alice.org lesson smoke | Gate agentic method/parameter review on real launch evidence. |
| `functions-as-questions-about-the-world` | Alice.org lesson smoke | Gate function/state review on real launch evidence. |
| `loops-and-conditionals-mini-challenge` | Alice.org lesson smoke | Gate loop/conditional review on real launch evidence. |
| `events-collision-proximity-game` | Alice.org lesson smoke | Gate event/proximity review on real launch evidence. |
| `game-score-timer-win-lose-loop` | Student game/state smoke | Gate score, timer, win/lose, and reflection review on real launch evidence. |
| `hour-of-code-studio-kickoff` | Alice.org studio smoke | Keep first-scene and reflection expectations in YAML while runtime stops at launch evidence. |
| `starter-project-open-save-export-preflight` | Desktop preflight | Launch Alice with the bundled starter project before any save/reopen/export claim is trusted. |
| `vr-camera-locomotion-journey` | VR/camera preflight | Record VR availability and require camera/comfort fallback evidence when real VR is unavailable. |
| `variables-scorekeeper-timekeeper` | Student data/state smoke | Gate variables, data types, scorekeeper, and timer review on real launch evidence. |
| `arrays-collection-choreography` | Student data/state smoke | Gate array/list/index review on real launch evidence. |
| `mythic-choice-event-tree` | Student interactive narrative smoke | Gate choice, event, branch, and peer-playtest review on real launch evidence. |
| `vr-camera-perspective-tour` | Student camera/VR smoke | Gate audience viewpoint and non-VR fallback review on real launch evidence. |
| `first-lessons-real-ui-actions` | Real UI action contract | Launch Alice, detect the Alice window, write `ui-action-contract.json`, and fail explicitly until deterministic UI actions are automated. |
| `modified-class-portability` | Class portability contract | Validate the export/import evidence contract and route the scenario through launch-smoke before agents judge class portability. |

These scenarios are based on Alice.org lesson/tutorial resource families, Alice
desktop QA journeys, and editable student creative scenarios. They prove that the
desktop harness can reach a smoke-ready Alice session for resource-grounded paths
before agentic instructor/student evaluation is trusted. The UI action and class
portability scenarios intentionally add evidence contracts around the launch smoke
instead of pretending the launch smoke already performs those user actions.

The outside-in Alice QA expansion commits these additional desktop scenarios:

| Scenario | Role | Runtime contract |
| --- | --- | --- |
| `setup-support-lab-readiness` | IT/setup support smoke | Gate install, Java, graphics, storage, account, and fallback readiness on real launch evidence. |
| `alice-2-migration-bridge` | Alice 2 migration smoke | Gate Alice 2 to Alice 3 lesson mapping on real launch evidence and visible student outcomes. |
| `vr-player-comfort-playtest` | VR/player comfort smoke | Gate comfort, orientation, discoverability, and desktop fallback claims on recorded availability evidence. |
| `model-texture-import-checkpoint` | Model/texture import smoke | Gate source, license, scale, orientation, texture visibility, and fallback claims on explicit evidence. |

## What the desktop scenarios verify

Most scenarios are manifest-only, scenario-labeled launch smokes. They verify smoke
readiness from deterministic harness evidence:

- Alice was launched through the existing `eatme-alice` launch smoke path.
- The manifest identifies `scenario_id` as the selected desktop scenario, such as
  `hour-of-code-studio-kickoff`, `building-a-scene-first-world`,
  `code-editor-first-run`, or one of the expanded Alice.org-grounded lesson ids,
  including the score/timer game scenario and starter-project preflight.
- The deterministic launch assertions pass: dependencies, X display, Alice
  process startup, startup screenshot, and fatal-log scan.
- The starter-project preflight expects the launch command to include bundled
  `africa.a3p`, giving the next open/save/export pass a real opened-project
  baseline.
- Alice log and window-list files are captured as artifacts when available.
- A non-empty startup screenshot or captured window list proves visual startup
  evidence for desktop scenarios. Screenshots are represented by the top-level
  `screenshot` manifest artifact when available.
- The run artifacts are stored under a scenario-specific run directory.

Most scenarios intentionally stop at launch-ready evidence so lesson smokes remain
stable in normal developer and CI environments. For the Hour of Code studio
kickoff, learner-visible first-scene, first-animation, evidence, and reflection
expectations live in editable YAML as agentic follow-on contracts; runtime smoke
still stops at deterministic launch-ready evidence.

The baseline `real-alice-launch-smoke` scenario proves only the scenario-labeled
launch path and captured manifest/log/window/screenshot evidence. It is not full
UI automation, not creative assessment, and not learner-world grading.

The `first-lessons-real-ui-actions` scenario is different: it is an executable
harness contract for the first real UI actions. It launches Alice, verifies an
Alice Stage IDE window from window-manager evidence, writes
`ui-action-contract.json`, and fails loudly with
`ui_action_automation_unimplemented` until a deterministic
`deterministic-alice-object-gallery-placement-affordance` can place a named
object without coordinate guessing, and follow-on automation can edit a
procedure/code block, run the world, and save a project.
This is launch/action-contract evidence only. It is not full UI automation, not
creative assessment, and not learner-world grading.

The `modified-class-portability` scenario is also not a plain lesson smoke. Its YAML
defines the export package, import report, and after-import behavior evidence
required before anyone claims a modified class travels between Alice projects.
The shared launch-smoke path records the scenario manifest; class export/import
proof remains an explicit evidence contract for follow-on automation or agentic
review.

The `vr-camera-locomotion-journey` scenario adds an explicit VR preflight contract:
real headset or Alice Player VR execution is optional, but availability must be
recorded. If real VR is unavailable, evidence must state
`real_vr_available=false` and include the desktop launch manifest plus
camera-marker/viewpoint and locomotion-comfort artifacts. This keeps VR claims
outside-in and evidence-based instead of silently skipping unavailable hardware.

The expanded instructor/student outside-in scenarios use the same rule. Real Alice
execution remains manual or locally gated with `EATME_REAL_ALICE=1`. A passing
manifest proves the selected scenario reached a smoke-ready desktop session; it does
not replace the scenario-specific readiness checklist, migration map, comfort
notes, import review, fallback artifact, or student reflection required by the
YAML contract.

Instructor lesson-material evidence is handled separately by
`instructor-lesson-materials-remix`. That agentic-flow asset verifies
scenario-labeled prompts, acceptance probes, teacher plan, student handout, exit
ticket, and instructor review/remix language without claiming automated creative
grading or learner-world assessment.

## Outside-in evidence guide for Alice lesson scenarios

Use outside-in evidence for instructor and student Alice lesson scenarios when a
reviewer needs to connect a classroom scenario to real Alice startup artifacts
and explicit instructor/student deliverables.

| Need | Scenario | What to collect |
| --- | --- | --- |
| Prove the harness can launch real Alice for a named scenario | `real-alice-launch-smoke` or any `alice_lesson_smoke` id | `manifest.json`, `alice.log`, `window-list.txt` when available, startup screenshot, and passing launch assertions. |
| Prove the student first-lesson scenario has an executable action contract | `first-lessons-real-ui-actions` | Launch manifest, Alice window evidence, screenshot/log artifacts, and `ui-action-contract.json` with object placement, procedure edit, run-world, and save-project expectations. |
| Prove instructor lesson materials are represented as reviewable assets | `instructor-lesson-materials-remix` | Teacher plan, student handout, exit ticket, instructor review prompts, remix notes, and acceptance probes. |

The three evidence levels are intentionally separate:

1. Launch evidence proves Alice started for the selected scenario id.
2. Action-contract evidence records the first UI actions that future automation
   must perform deterministically.
3. Mission evidence is the human or agent-reviewed classroom output, such as a
   learner reflection or instructor handout.

Do not collapse those levels into one pass/fail claim. Passing launch smoke does
not mean Alice has been driven through a lesson, does not assess creative
quality, and does not grade a saved world.

### Student first-lesson recipe

Run the action-contract scenario when the student scenario requires evidence for the
first real Alice action path:

```bash
export NODE_OPTIONS=--max-old-space-size=32768
export ALICE_HOME="${ALICE_HOME:-../alice3-modernization}"
export SCENARIO_ID=first-lessons-real-ui-actions
export RUN_ID=student-${SCENARIO_ID}

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

Expected evidence location:

```text
runs/first-lessons-real-ui-actions/student-first-lessons-real-ui-actions/
|-- manifest.json
|-- alice.log
|-- window-list.txt
|-- ui-action-contract.json
`-- screenshots/
    `-- startup.png
```

The explicit `ui_action_automation_unimplemented` failure is honest evidence
that the action contract exists but deterministic UI automation is not yet
claiming a full lesson pass. For object placement, inspect both
`action_precondition_probes[].missing_affordance` and
`candidate_affordance_probes[]` in `ui-action-contract.json`. The candidate
probe validates whether the Alice checkout exposes `tools/eatme-place-object`
and only passes object placement when that Alice-side command returns a
non-empty placement artifact plus a scene/project diff for the named gallery
object. Treat every other outcome as a boundary signal, not as completed UI
coverage.

### Instructor remix recipe

Use `instructor-lesson-materials-remix` when the evidence is a classroom packet
rather than desktop automation. Validate the canonical asset and adapter before
using the prompts:

```bash
cargo run -q -p eatme-cli -- assets validate \
  --path assets/scenarios/eatme/instructor-lesson-materials-remix.yaml \
  --json

cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
```

The instructor packet should contain:

| Output | Minimum evidence |
| --- | --- |
| Teacher plan | Alice resource grounding, concept goal, setup/fallback notes, facilitation steps, and timing. |
| Student handout | Learner mission, prediction/run/revise/reflection prompts, and artifact submission shape. |
| Exit ticket | Short checks for concept understanding, evidence of revision, and remaining questions. |
| Review/remix notes | What was changed, what stayed aligned with the Alice resource, and what needs instructor judgment. |

This scenario can reference real Alice launch evidence as setup context, but it
does not perform creative assessment or learner-world grading automatically.

## Scenario assets

Canonical desktop scenarios live under:

```text
assets/scenarios/eatme/
```

The canonical desktop scenarios are defined by:

```text
assets/scenarios/eatme/building-a-scene-first-world.yaml
assets/scenarios/eatme/code-editor-first-run.yaml
assets/scenarios/eatme/reusable-methods-and-parameters.yaml
assets/scenarios/eatme/functions-as-questions-about-the-world.yaml
assets/scenarios/eatme/loops-and-conditionals-mini-challenge.yaml
assets/scenarios/eatme/events-collision-proximity-game.yaml
assets/scenarios/eatme/first-lessons-real-ui-actions.yaml
assets/scenarios/eatme/game-score-timer-win-lose-loop.yaml
assets/scenarios/eatme/modified-class-portability.yaml
assets/scenarios/eatme/hour-of-code-studio-kickoff.yaml
assets/scenarios/eatme/starter-project-open-save-export-preflight.yaml
assets/scenarios/eatme/vr-camera-locomotion-journey.yaml
assets/scenarios/eatme/variables-scorekeeper-timekeeper.yaml
assets/scenarios/eatme/arrays-collection-choreography.yaml
assets/scenarios/eatme/mythic-choice-event-tree.yaml
assets/scenarios/eatme/vr-camera-perspective-tour.yaml
```

These files are the editable design contracts for desktop scenarios. Lesson copy,
resource links, smoke steps, expected evidence, timeouts, artifact paths, and
Gherkin-style acceptance criteria are edited in YAML rather than Rust tests.

Runtime behavior is intentionally narrower than the YAML contracts:
`alice launch-smoke` does not load the YAML file. The `--scenario` value
supplies the manifest `scenario_id` and run-directory namespace; asset
validation separately checks that the YAML contract remains well-formed.

The expanded canonical scenario files include:

```text
assets/scenarios/eatme/setup-support-lab-readiness.yaml
assets/scenarios/eatme/alice-2-migration-bridge.yaml
assets/scenarios/eatme/vr-player-comfort-playtest.yaml
assets/scenarios/eatme/model-texture-import-checkpoint.yaml
```

Gadugi-compatible adapters live under:

```text
assets/scenarios/gadugi/
```

The gadugi adapters for the committed scenarios are:

```text
assets/scenarios/gadugi/building-a-scene-first-world.yaml
assets/scenarios/gadugi/code-editor-first-run.yaml
assets/scenarios/gadugi/reusable-methods-and-parameters.yaml
assets/scenarios/gadugi/functions-as-questions-about-the-world.yaml
assets/scenarios/gadugi/loops-and-conditionals-mini-challenge.yaml
assets/scenarios/gadugi/events-collision-proximity-game.yaml
assets/scenarios/gadugi/first-lessons-real-ui-actions.yaml
assets/scenarios/gadugi/game-score-timer-win-lose-loop.yaml
assets/scenarios/gadugi/modified-class-portability.yaml
assets/scenarios/gadugi/hour-of-code-studio-kickoff.yaml
assets/scenarios/gadugi/starter-project-open-save-export-preflight.yaml
assets/scenarios/gadugi/vr-camera-locomotion-journey.yaml
assets/scenarios/gadugi/variables-scorekeeper-timekeeper.yaml
assets/scenarios/gadugi/arrays-collection-choreography.yaml
assets/scenarios/gadugi/mythic-choice-event-tree.yaml
assets/scenarios/gadugi/vr-camera-perspective-tour.yaml
```

Gadugi lesson scenarios may invoke the eatme CLI and inspect manifest-level
evidence. They must not own or duplicate Alice runtime behavior such as Xvfb
management, Swing/Java launch details, screenshot capture, log capture, or
process lifecycle. The additional
`assets/scenarios/gadugi/validation-failure-exit-code.yaml` regression adapter
covers the asset-validation exit-code contract without launching Alice.

Generated adapters for the expanded canonical files are committed at:

```text
assets/scenarios/gadugi/setup-support-lab-readiness.yaml
assets/scenarios/gadugi/alice-2-migration-bridge.yaml
assets/scenarios/gadugi/vr-player-comfort-playtest.yaml
assets/scenarios/gadugi/model-texture-import-checkpoint.yaml
```

## Validate assets

Validate every committed persona and scenario asset:

```bash
cargo run -q -p eatme-cli -- assets validate --json
```

Validate only one scenario:

```bash
cargo run -q -p eatme-cli -- assets validate \
  --path assets/scenarios/eatme/building-a-scene-first-world.yaml \
  --json

cargo run -q -p eatme-cli -- assets validate \
  --path assets/scenarios/eatme/code-editor-first-run.yaml \
  --json

cargo run -q -p eatme-cli -- assets validate \
  --path assets/scenarios/eatme/starter-project-open-save-export-preflight.yaml \
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

For any other committed lesson scenario, use the same command shape with one of the
committed scenario ids above and a matching descriptive run id.

`ALICE_HOME` must point at the Alice source checkout to package and launch. A
typical local value is:

```bash
export ALICE_HOME="${ALICE_HOME:-../alice3-modernization}"
```

The generic launch smoke still works without selecting the lesson scenario:

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
path; it does not enforce the `EATME_REAL_ALICE` gate, though it still requires
the same real desktop dependencies to pass.

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
  [--starter-project <path>] \
  [--timeout <seconds>] \
  [--json] \
  [--no-memory] \
  [--offline-package]
```

### Real UI action contract

Use the action contract scenario when the intended evidence is a declared
first-action contract toward user-visible Alice behavior rather than
manifest-only startup:

```bash
EATME_REAL_ALICE=1 cargo run -q -p eatme-cli -- alice launch-smoke \
  --alice-home "${ALICE_HOME}" \
  --scenario first-lessons-real-ui-actions \
  --run-id local-first-lessons-real-ui-actions \
  --runs-dir runs \
  --timeout 900 \
  --json \
  --no-memory \
  --offline-package
```

Until real UI automation is wired, this command is expected to exit non-zero
after writing a manifest and `ui-action-contract.json`. Treat that explicit
failure as the contract, not as passing coverage. This scenario is launch/action-
contract evidence only; it is not full UI automation, not creative assessment,
and not learner-world grading.

| Option | Description |
| --- | --- |
| `--alice-home <path>` | Alice checkout to package and launch. |
| `--run-id <run-id>` | Stable id for this run. Use a descriptive id for local or CI traces. |
| `--scenario <scenario-id>` | Scenario id to record in the manifest and run directory. Defaults to `real-alice-launch-smoke`; it does not load scenario YAML at runtime yet. |
| `--runs-dir <path>` | Root directory for run artifacts. Defaults to `runs`. |
| `--starter-project <path>` | Starter project to open. Relative paths resolve from `--alice-home`; defaults to Alice's `africa.a3p`. |
| `--timeout <seconds>` | Maximum launch wait before the smoke fails. |
| `--json` | Accepted compatibility flag. Output is pretty JSON whether or not this flag is present. |
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

The code editor scenario uses:

```text
runs/code-editor-first-run/<run-id>/
```

The baseline launch smoke uses:

```text
runs/real-alice-launch-smoke/<run-id>/
```

If a run reuses the same `--run-id`, the previous evidence directory is archived
next to the new run as `<run-id>.previous-...` instead of being deleted.

## Manifest reference

Every launch smoke manifest includes the launch evidence needed by eatme and
gadugi adapters. Important fields for lesson smoke consumers are:

| Field | Meaning |
| --- | --- |
| `schema_version` | Manifest schema version. |
| `scenario_id` | Scenario selected for the run, such as `building-a-scene-first-world`, `code-editor-first-run`, `reusable-methods-and-parameters`, `functions-as-questions-about-the-world`, `loops-and-conditionals-mini-challenge`, `events-collision-proximity-game`, `first-lessons-real-ui-actions`, `game-score-timer-win-lose-loop`, `modified-class-portability`, `hour-of-code-studio-kickoff`, `starter-project-open-save-export-preflight`, `vr-camera-locomotion-journey`, `variables-scorekeeper-timekeeper`, `arrays-collection-choreography`, `mythic-choice-event-tree`, `vr-camera-perspective-tour`, `setup-support-lab-readiness`, `alice-2-migration-bridge`, `vr-player-comfort-playtest`, or `model-texture-import-checkpoint`. |
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
| `screenshot.path` | Top-level startup screenshot artifact path. |
| `screenshot.size_bytes` | Startup screenshot size. |
| `screenshot.sha256` | Startup screenshot digest. |
| `window_list.path` | Captured window-list artifact path from `wmctrl -lx`, or `xwininfo -root -tree` when no window-manager client list is available. |
| `window_list_error` | Window-list capture or artifact error when unavailable. |
| `screenshot_error` | Screenshot capture or artifact error when unavailable. |
| `log.path` | Alice log path. |
| `log.size_bytes` | Alice log size. |
| `log.sha256` | Alice log digest. |
| `log_error` | Log read or artifact error when unavailable. |
| `fatal_log_scan` | Fatal DISPLAY/OpenGL/Java pattern scan result. |
| `assertions` | Deterministic launch assertions. |
| `failure_category` | Failure classification, or `null` for a passing smoke. |

Consumers should treat `assertions` and `failure_category` as the source of
truth for smoke status. Gadugi adapters should not inspect desktop internals
outside the manifest and captured artifacts.

`startup_screenshot` is an assertion key under `assertions`, not a top-level
artifact field. The top-level screenshot artifact is named `screenshot`. The
`real_alice_execution_evidence` assertion is the adapter contract that proves a
real Alice process stayed alive on a responsive virtual display while visual
evidence and a non-empty launch log were captured.

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
  Prove that the lesson-specific smoke scenario launches through the same real
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
      - manifest assertions include real_alice_execution_evidence passed=true
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
| `purpose` | Plain-language reason the scenario exists. |
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
`artifacts.screenshot`, and `artifacts.log`, include at least one timeout, and
make the launch-smoke step inspect
`manifest.assertions.real_alice_execution_evidence`.

Design-convention fields such as `resource_basis`, `capabilities`, and
`adapter.targets` may appear in assets, but the validator does not deserialize
or enforce them. Treat them as documentation for humans and agents, not as
runtime inputs.

## Configuration

| Variable | Required | Description |
| --- | --- | --- |
| `EATME_REAL_ALICE=1` | Yes for lesson-labeled real launch | Enables non-baseline lesson smoke scenarios. Without it, those scenarios fail fast. |
| `ALICE_HOME` | Yes for real launch | Alice checkout used by `--alice-home`. |
| `RUN_ID` | Optional | Convenience value used by scenario YAML and gadugi adapters. |
| `NODE_OPTIONS=--max-old-space-size=32768` | Optional | Preserved environment preference for Node-based wrappers or agent tooling; the Rust CLI does not require it. |

Host dependencies for real launch are the same as the baseline smoke: Java 21,
Maven, Xvfb, `xdpyinfo`, `wmctrl`, `xwininfo`, `xdotool`, a screenshot tool, and
OpenGL/Mesa support for software rendering.
`glxinfo` is useful for diagnostics when present, but it is not required by the
launch-smoke preflight.

## Tutorial: local lesson smoke

1. Validate the editable assets:

   ```bash
   cargo run -q -p eatme-cli -- assets validate --json
   ```

2. Check desktop prerequisites:

   ```bash
   cargo run -q -p eatme-cli -- deps check --json
   ```

3. Run a lesson scenario:

   ```bash
   export ALICE_HOME="${ALICE_HOME:-../alice3-modernization}"
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
the lesson assertion has accepted captured window evidence. The
`real_alice_execution_evidence` assertion must pass; when preflight is blocked,
the command writes a manifest and diagnostic `alice.log` with a non-null
`failure_category` instead of silently skipping execution.

## Tutorial: gadugi adapter boundary

Use the gadugi assets when a gadugi runner needs to exercise a scenario:

```text
assets/scenarios/gadugi/building-a-scene-first-world.yaml
assets/scenarios/gadugi/code-editor-first-run.yaml
assets/scenarios/gadugi/reusable-methods-and-parameters.yaml
assets/scenarios/gadugi/functions-as-questions-about-the-world.yaml
assets/scenarios/gadugi/loops-and-conditionals-mini-challenge.yaml
assets/scenarios/gadugi/events-collision-proximity-game.yaml
assets/scenarios/gadugi/first-lessons-real-ui-actions.yaml
assets/scenarios/gadugi/game-score-timer-win-lose-loop.yaml
assets/scenarios/gadugi/modified-class-portability.yaml
assets/scenarios/gadugi/hour-of-code-studio-kickoff.yaml
assets/scenarios/gadugi/starter-project-open-save-export-preflight.yaml
assets/scenarios/gadugi/vr-camera-locomotion-journey.yaml
assets/scenarios/gadugi/variables-scorekeeper-timekeeper.yaml
assets/scenarios/gadugi/arrays-collection-choreography.yaml
assets/scenarios/gadugi/mythic-choice-event-tree.yaml
assets/scenarios/gadugi/vr-camera-perspective-tour.yaml
```

The adapter performs three kinds of work:

1. Run `eatme assets validate --json` and expect exit `0` only when the output
   reports `"passed": true`.
2. Run `eatme deps check --json`.
3. Run `eatme alice launch-smoke --scenario <lesson-id>`.

Standard launch-smoke adapters assert command success and manifest-level output
such as the selected `"scenario_id"`, `"failure_category": null`, startup
screenshot or window evidence, `real_alice_execution_evidence`, and passing
assertions. They do not reimplement or configure Alice launch internals.

The `first-lessons-real-ui-actions` adapter is the deliberate exception. It
preserves the current action-contract boundary by expecting the launch step to
exit `1` with `"failure_category": "ui_action_automation_unimplemented"` after
real Alice launch evidence and `ui-action-contract.json` have been written. Do
not reinterpret that non-zero result as completed UI automation; it remains a
declared first-action contract until deterministic object/place/edit/run/save
automation exists.

Use `assets/scenarios/gadugi/validation-failure-exit-code.yaml` as the negative
counterpart: it creates a malformed scenario asset and expects
`eatme assets validate --path ...` to exit `1` with `"passed": false`.

The committed gadugi adapters are generated from the canonical eatme scenario
assets and portable: the agent config uses `cwd: .` and shell commands begin
with `cd "${EATME_REPO:-.}"`, so a runner may set `EATME_REPO` without baking
in a checkout-specific path. Run these scenarios from the checkout under test so
asset validation counts the assets in that checkout, including gadugi-only
regression adapters. Regenerate or verify them with:

```bash
cargo run -q -p eatme-cli -- assets generate-gadugi --root .
cargo run -q -p eatme-cli -- assets generate-gadugi --root . --check
```

## Testing expectations

Always-on tests cover asset/schema validation and fake/gated harness behavior
without requiring Alice to launch. Real Alice validation is the explicitly gated
CLI smoke command:

```bash
EATME_REAL_ALICE=1 cargo run -q -p eatme-cli -- alice launch-smoke \
  --alice-home "${ALICE_HOME}" \
  --scenario starter-project-open-save-export-preflight \
  --run-id local-starter-project-open-save-export-preflight \
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

A desktop scenario is ready to trust when committed scenario assets validate,
malformed scenario fixtures fail with actionable messages, the fake harness
proves the scenario id is routed through the existing launch smoke path, and the
gated real Alice command produces distinct scenario artifacts when the host
supports desktop launch. UI action, portability, and VR claims also require
their declared evidence contracts; a scenario-labeled launch manifest alone is
not enough to claim those user outcomes.
