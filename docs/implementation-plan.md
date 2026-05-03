# Eatme implementation plan

## Purpose

`eatme` is the private agentic QA harness for Alice. It builds editable instructor/student personas and outside-in scenarios from Alice.org resources, then exercises the real `alice3-modernization` fork through observable desktop sessions.

The plan is intentionally staged. The first milestone is not a classroom simulator and not an agentic lesson. The first milestone is a deterministic real-Alice launch smoke that proves the harness can run the actual desktop application and capture trustworthy evidence.

## Hard constraints

- Keep assets editable by non-coders.
- Keep Rust modules under 500 lines.
- Target at least 70% line coverage for the Rust workspace.
- Keep Alice itself Java/Maven; Rust orchestrates, validates, observes, records, and reports.
- Use deterministic process/X/log/screenshot evidence as the first test oracle.
- Demote agentic judgment until deterministic evidence is stable.
- Keep memory and generated artifacts namespaced under `alice.eatme`.
- Lesson-labeled real Alice validation stays gated behind `EATME_REAL_ALICE=1`.

## Milestone 0: deterministic real-Alice launch smoke

### Scope

Milestone 0 is deliberately small:

1. Detect host dependencies.
2. Package real Alice.
3. Start long-lived Xvfb.
4. Launch Alice via direct Java.
5. Isolate user state and temp/cache state.
6. Capture process status, window/display information, logs, screenshot, and manifest.
7. Emit deterministic pass/fail.

No gadugi dependency. No personas. No lesson evaluation. No parallel GUI execution.

### Rust crates for Milestone 0

```text
crates/
├── eatme-cli/
├── eatme-core/
├── eatme-alice/
└── eatme-test-support/
```

Additional crates (`eatme-assets`, `eatme-gadugi`, `eatme-memory`, `eatme-report`) come after launch smoke passes.

### CLI commands

```bash
eatme deps check --json
eatme alice discover --alice-home /home/azureuser/src/alice3-modernization --json
eatme alice package --alice-home /home/azureuser/src/alice3-modernization --offline --json
eatme alice launch-smoke \
  --alice-home /home/azureuser/src/alice3-modernization \
  --run-id local-real-alice-launch-smoke \
  --runs-dir runs \
  --timeout 120 \
  --json \
  --no-memory
```

### Required host dependencies

- `Xvfb`
- `xdpyinfo`
- `xdotool`
- `wmctrl`
- `import` or `scrot`
- `glxinfo`
- Mesa software rendering/GLX libraries
- Java 21
- Maven

Dependency checks must fail loudly with actionable messages.

### Xvfb contract

- Use long-lived Xvfb, not one-shot `xvfb-run`.
- Enable GLX: `+extension GLX +render -noreset`.
- Validate display with `xdpyinfo`.
- Default to serial execution.
- Later parallel execution must allocate unique display/workspace/user home/prefs root per run.

### Direct Java launch contract

Direct Java launch must include:

- JavaFX `--module-path`
- `--add-modules javafx.graphics,javafx.media`
- `alice-ide/target/alice-ide-9.1.0-SNAPSHOT.jar`
- `alice-ide/target/lib/*`
- `org.alice.stageide.EntryPoint`
- starter project path, initially `core/resources/target/distribution/application/starter-projects/africa.a3p`
- `-Dorg.alice.ide.rootDirectory=./core/resources/target/distribution`
- isolated `-Duser.home`
- isolated `-Djava.util.prefs.userRoot`
- isolated `-Djava.io.tmpdir`
- `-Djogamp.gluegen.UseTempJarCache=false`
- `LIBGL_ALWAYS_SOFTWARE=1`

### Manifest contract

Each run writes:

```text
runs/real-alice-launch-smoke/<run-id>/
├── manifest.json
├── alice.log
├── xvfb.log
├── window-list.txt
├── home/
├── prefs/
├── tmp/
└── screenshots/
    └── startup.png
```

Required `manifest.json` fields:

- `schema_version`
- `scenario_id`
- `run_id`
- `alice_home`
- `alice_git_commit`
- `eatme_git_commit`
- `java_version`
- `maven_version`
- `dependency_checks`
- `build_command`
- `build_exit_status`
- `launch_command`
- `display`
- `xvfb_pid`
- `alice_pid`
- `timeout_seconds`
- `screenshot.path`
- `screenshot.size_bytes`
- `screenshot.sha256`
- `log.path`
- `log.size_bytes`
- `log.sha256`
- `fatal_log_scan`
- `assertions`
- `failure_category`

### Milestone 0 assertions

Pass/fail must come from deterministic evidence:

- dependency checks passed
- Alice package command succeeded
- X display responsive
- Alice process started
- screenshot exists and is non-empty
- fatal DISPLAY/OpenGL/Java exception patterns are absent

The Alice log and `window-list.txt` are diagnostic artifacts, not independent
pass/fail assertions in the current harness. Agentic annotations may be attached
later, but they do not decide pass/fail in Milestone 0.

## Milestone 1: canonical assets and first lesson smoke

Milestone 1 layers the first lesson-specific smoke lane on top of the real
Alice launch smoke harness:

- `eatme-assets` validates editable persona and scenario YAML.
- Canonical scenarios live under `assets/scenarios/eatme/`.
- Gadugi adapters live under `assets/scenarios/gadugi/`.
- Lesson smoke scenarios route through `eatme alice launch-smoke --scenario <id>`.
- Non-baseline lesson smoke scenarios remain gated by `EATME_REAL_ALICE=1`.
- Scenario YAML is validated separately; `launch-smoke` currently records the
  scenario id and run namespace but does not load YAML fields as runtime inputs.

The first lesson smoke is Alice.org resource-specific:

- `building-a-scene-first-world`
- resource basis: Building a Scene + Scene Editor Overview
- current evidence: manifest-only launch readiness under a lesson-specific
  scenario id
- future lesson-automation evidence:
  - at least two objects
  - one object positioned/oriented/scaled
  - camera view/marker language
  - saved project/world
  - learner explanation/reflection

Usage, CLI, manifest, scenario schema, configuration, and examples are
documented in [`alice-lesson-smoke.md`](alice-lesson-smoke.md).

## Milestone 2: gadugi boundary

Gadugi should orchestrate `eatme` as a CLI/system harness. It should not own Swing/Xvfb/Desktop behavior yet.

Boundary:

- `eatme` owns Alice packaging, Xvfb/display allocation, window manager, Java process lifecycle, screenshots, logs, manifests, rubrics, persona assets, and memory namespace.
- `gadugi` owns running `eatme` CLI commands, collecting stdout/stderr/result JSON, and evaluating manifest-level pass/fail evidence.
- `eatme-gadugi` owns compiling/adapting `eatme` scenarios into gadugi-compatible CLI/MIXED scenarios.

Add:

```bash
eatme gadugi compile --scenario real-alice-launch-smoke --out assets/scenarios/gadugi/real-alice-launch-smoke.yaml
```

Canonical scenarios live in `assets/scenarios/eatme/`. Generated/adapted gadugi scenarios live in `assets/scenarios/gadugi/`.

## Milestone 3: personas and resource-grounded scenarios

Start with:

- Instructor: `concept-cartographer`
- Student: `curious-novice`

Then add Alice.org core path scenarios:

1. `building-a-scene-first-world`
2. `code-editor-first-run`
3. `control-structures-visible-change`
4. `introduction-to-events-first-binding`
5. `design-process-thin-slice`

Defer export/player and collision/proximity game scenarios until after the core path.

Scenario design conventions should include:

- `schema_version`
- `resource_basis`
- `capabilities.required`
- `capabilities.optional`
- `adapter.targets`
- `steps[].id`
- `timeouts`
- `artifacts`
- `unsupported_policy`

The current validator enforces the subset needed for launch-smoke routing and
the `building-a-scene-first-world` contract. Fields such as `resource_basis`,
`capabilities.*`, and `adapter.targets` are human/agent documentation until the
asset schema grows stricter enforcement.

## Milestone 4: memory and reporting

Memory starts local and simple:

```text
.eatme/memory/events.jsonl
```

Store:

- scenario outcomes
- Alice launch failures
- missing dependencies
- persona/scenario coverage
- successful recovery actions

Only after JSONL memory is stable should we add an optional adapter to the local Rust `amplihack-memory` crate.

## Milestone 5: controlled parallelism

Default execution is serial.

No parallel GUI runs until display allocation, workspace isolation, port/display locking, and cleanup are tested.

Parallel GUI runs require unique:

- `DISPLAY`
- workspace
- user home
- prefs root
- temp/cache directory
- cleanup guard

## Governance boundaries

- Agents may read lessons, scenarios, and artifacts.
- Agents may not modify Alice source.
- Agents may not modify `eatme` source during test execution.
- Supporting tool repos such as `amplihack-rs`, `gadugi-agentic-test`, `amplihack-recipe-runner`, and `amplihack-memory-lib` are in scope for bug fixes or feature work when needed.
- Any supporting-tool repo change must follow the default workflow, and subagents doing that work must follow the default workflow too.
- Commands must be visible through CLI stdout/stderr and manifest fields.
  Dedicated `commands.jsonl` command logging is future work, not part of the
  current launch-smoke artifact tree.
- Memory writes stay under `.eatme/memory` or `alice.eatme`.
- No silent repo mutation.

## Validation commands

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo llvm-cov --workspace --all-features --fail-under-lines 70
find crates -name '*.rs' -not -path '*/target/*' -exec wc -l {} + \
  | awk '$1 > 500 { print; bad=1 } END { exit bad }'
```

Real Alice validation:

```bash
EATME_REAL_ALICE=1 cargo run -q -p eatme-cli -- alice launch-smoke \
  --alice-home /home/azureuser/src/alice3-modernization \
  --scenario building-a-scene-first-world \
  --run-id local-building-a-scene-first-world \
  --runs-dir runs \
  --timeout 120 \
  --json \
  --no-memory \
  --offline-package
```
