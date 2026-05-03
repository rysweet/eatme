# Eatme implementation plan

## Purpose

`eatme` is the private agentic QA harness for Alice. It builds editable instructor/student personas and outside-in scenarios from Alice.org resources, then exercises the real `alice3-modernization` fork through observable desktop sessions.

The first priority is not a full classroom simulator. The first priority is a repeatable vertical slice:

1. Build/package real Alice from `/home/azureuser/src/alice3-modernization`.
2. Start Alice under Xvfb with isolated user state.
3. Capture deterministic evidence: process status, logs, screenshot, run manifest.
4. Evaluate one editable lesson/scenario with a student persona and deterministic pass/fail probes.
5. Store results for later review and memory.

## Design constraints

- Keep assets editable by non-coders.
- Keep Rust modules under 500 lines.
- Target at least 70% line coverage for the Rust workspace.
- Keep Alice itself Java/Maven; Rust orchestrates, validates, observes, records, and reports.
- Prefer agentic observations and high-level intents over brittle UI selectors.
- Keep deterministic evidence around every agentic judgment.
- Namespace generated memory/todos/artifacts under `alice.eatme`.

## Repository structure

```text
eatme/
├── Cargo.toml
├── assets/
│   ├── alice/
│   ├── lessons/
│   ├── personas/
│   ├── prompts/
│   ├── rubrics/
│   └── scenarios/
├── crates/
│   ├── eatme-cli/
│   ├── eatme-core/
│   ├── eatme-assets/
│   ├── eatme-alice/
│   ├── eatme-gadugi/
│   ├── eatme-memory/
│   ├── eatme-report/
│   └── eatme-test-support/
├── docs/
├── runs/
└── tests/
```

## Phase 1: foundation

### Rust workspace

Create a minimal Rust workspace with:

- `eatme-cli`: CLI command routing.
- `eatme-core`: typed config, errors, run manifests, `CommandRunner` trait.
- `eatme-assets`: YAML asset loading and validation.
- `eatme-alice`: Alice build/run command construction and Xvfb session orchestration.
- `eatme-gadugi`: adapter for gadugi-agentic-test invocation.
- `eatme-memory`: `MemoryStore` trait with `NoopMemoryStore` and JSONL fallback.
- `eatme-report`: markdown/json report summaries.
- `eatme-test-support`: fake command runner and fixtures.

### Editable assets

Add:

- `assets/personas/alice-user-crew.yaml` (already present).
- `assets/alice/forks.yaml` for Alice source locations and build profiles.
- `assets/lessons/real-alice-launch-smoke.yaml` for the first vertical slice.
- `assets/scenarios/gadugi/real-alice-launch-smoke.yaml` for gadugi-compatible execution.
- `assets/rubrics/real-alice-launch-smoke.yaml` for deterministic evidence criteria.

### CLI commands

Initial commands:

```bash
eatme assets validate
eatme alice discover --alice-home /home/azureuser/src/alice3-modernization
eatme alice package --alice-home /home/azureuser/src/alice3-modernization --offline
eatme alice launch-smoke --alice-home /home/azureuser/src/alice3-modernization
eatme report summarize --run runs/<id>
```

## Phase 2: real Alice vertical slice

### Host dependencies

Install or detect:

- `xvfb`
- `xauth`
- `x11-utils`
- `x11-apps`
- `imagemagick` or `scrot`
- `xdotool`
- `wmctrl`
- Mesa software rendering libraries

The harness should fail loudly with actionable missing-dependency messages.

### Alice launch model

Use direct Java after packaging, not `mvn exec:java`, because direct launch avoids Maven dependency noise during UI tests.

Required observations:

- Alice process alive.
- X display responds.
- Screenshot captured.
- Alice log captured.
- No fatal `Unable to open DISPLAY`, `SEVERE`, or uncaught Java exception dominates the run.

### Evidence output

Each run writes:

```text
runs/<scenario>/<timestamp>/
├── manifest.json
├── alice.log
├── screenshots/
│   └── startup.png
├── commands.jsonl
└── report.md
```

## Phase 3: instructor/student agentic scenarios

Start with one instructor and one student:

- Instructor: `concept-cartographer`
- Student: `curious-novice`

First scenario:

- Resource basis: Alice.org "Building a Scene" and setup/download docs.
- Goal: teacher assigns a minimal first-world setup; student launches Alice, sees a starter project or editor, and records what is visible.
- Deterministic probes: screenshot exists, process alive, no fatal log, run manifest complete.

Then add:

- `programming-in-alice-first-run`
- `control-structures-visible-change`
- `events-binding-smoke`
- `export-player-smoke`

## Phase 4: memory and review

Memory starts as local JSONL:

```text
.eatme/memory/events.jsonl
```

Store:

- scenario outcomes
- recurring Alice launch failures
- missing dependencies
- persona/scenario coverage
- successful recovery actions

Add optional integration with the local Rust `amplihack-memory` crate after the JSONL flow is stable.

## Phase 5: scale-out

Only after the vertical slice passes locally and in CI-like conditions:

- Add more personas.
- Add the 11 resource-grounded core scenarios.
- Add the 10 creative teaching/learning scenarios.
- Add parallel run isolation by display/workspace allocation.
- Propose a gadugi `DESKTOP`/`SWING` agent upstream if the eatme-specific harness proves useful.

## Second-pass review targets

The plan needs focused review from:

- harness reviewer: Xvfb/Alice launch feasibility and missing host risks
- gadugi reviewer: scenario adapter boundaries and upstream feature candidates
- curriculum reviewer: whether scenarios reflect Alice.org resources accurately
- crusty reviewer: whether scope is still too broad or sequencing is wrong

## Initial validation commands

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
EATME_REAL_ALICE=1 ALICE_HOME=/home/azureuser/src/alice3-modernization cargo test --test real_alice
```
