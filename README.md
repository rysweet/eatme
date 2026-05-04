# eatme

`eatme` is an outside-in QA harness for Alice. It keeps instructor and student
mission intent in editable YAML, validates those assets, generates Gadugi
adapter scenarios from the canonical eatme scenarios, and launches real Alice
desktop smoke runs when an environment explicitly opts in.

The repository is organized around a simple boundary:

| Area | Owner |
| --- | --- |
| Rust CLI and harness | Dependency checks, Alice discovery, packaging, launch smoke evidence, asset validation, Gadugi adapter generation |
| Canonical assets | Persona crews and eatme scenario YAML under `assets/` |
| Generated adapters | Gadugi-compatible scenario YAML under `assets/scenarios/gadugi/` |
| Documentation site | MkDocs pages under `docs/` |

The harness deliberately favors observable evidence over hidden implementation
assertions. A passing Alice smoke is based on dependency checks, packaging,
process startup, virtual display readiness, screenshot or window evidence,
captured logs, and a manifest with deterministic assertions.

## Who uses eatme

- **Instructors** use scenario and mission docs to understand classroom-ready
  Alice activities and the evidence each mission expects.
- **Students** follow mission prompts that emphasize prediction, observation,
  iteration, and reflection.
- **QA and agent runners** validate assets, refresh Gadugi adapters, and run
  manifest-level Alice smoke checks.
- **Maintainers** keep runtime behavior in Rust while keeping lesson intent in
  editable YAML.

## Repository layout

```text
assets/personas/
  alice-user-crew.yaml
assets/scenarios/eatme/
  Canonical editable eatme scenarios.
assets/scenarios/gadugi/
  Generated Gadugi adapter scenarios.
crates/
  Rust workspace crates for CLI, Alice harness, assets, and support code.
docs/
  MkDocs source pages and existing guides.
scripts/
  Local quality gate scripts.
```

## Installation

Install a current Rust toolchain, clone the repository, and build the CLI through
Cargo:

```bash
git clone https://github.com/rysweet/eatme.git
cd eatme
cargo build --workspace
```

Run the CLI from source:

```bash
cargo run -q -p eatme-cli -- --help
```

For real Alice launch smoke runs, set `ALICE_HOME` to an Alice checkout and make
sure the host has Java 21, Maven, Xvfb, `xdpyinfo`, `wmctrl`, a screenshot tool,
and software OpenGL/Mesa support:

```bash
export ALICE_HOME="${ALICE_HOME:-../alice3-modernization}"
cargo run -q -p eatme-cli -- deps check --json
```

## Core CLI usage

Validate all persona and scenario assets:

```bash
cargo run -q -p eatme-cli -- assets validate --json
```

Validate one asset:

```bash
cargo run -q -p eatme-cli -- assets validate \
  --path assets/scenarios/eatme/building-a-scene-first-world.yaml \
  --json
```

Verify committed Gadugi adapters are fresh:

```bash
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
```

Regenerate Gadugi adapters from canonical eatme scenarios:

```bash
cargo run -q -p eatme-cli -- assets generate-gadugi --json
```

Discover and package Alice:

```bash
cargo run -q -p eatme-cli -- alice discover \
  --alice-home "${ALICE_HOME}" \
  --json

cargo run -q -p eatme-cli -- alice package \
  --alice-home "${ALICE_HOME}" \
  --offline \
  --json
```

Run a real lesson-labeled launch smoke:

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

Non-baseline scenarios require `EATME_REAL_ALICE=1` so a real desktop launch is
never confused with a mocked or skipped run.

## Scenario authoring

Canonical scenario intent lives under `assets/scenarios/eatme/`. Authors edit
those files first, validate them, then regenerate or check the Gadugi adapters.
The scenario YAML is the human and agent contract for mission purpose, evidence,
acceptance criteria, artifacts, timeouts, and unsupported behavior.

Gadugi files under `assets/scenarios/gadugi/` are generated adapter artifacts.
Do not hand-edit them for mission intent; update the canonical eatme scenario
and run the adapter generator instead.

## Validation and quality gates

The standard local gates are:

```bash
cargo run -q -p eatme-cli -- assets validate --json
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
scripts/quality-gates.sh
```

The Rust quality gate script runs formatting, clippy, tests, module-size checks,
and coverage. Real Alice desktop execution is explicit and environment-gated.

## Documentation site

Install the docs toolchain and build the static site:

```bash
python3 -m venv .venv
. .venv/bin/activate
python -m pip install -r requirements-docs.txt
mkdocs build --strict
```

The MkDocs source is in `docs/`. The generated site is written to `site/` and is
published by the GitHub Pages workflow from `master`.

Start with the full docs site:

- `docs/index.md` - project overview and audience routing
- `docs/installation.md` - setup requirements
- `docs/cli-usage.md` - command reference and examples
- `docs/scenario-authoring.md` - scenario editing workflow
- `docs/gadugi-adapters.md` - generated adapter workflow
- `docs/validation-quality-gates.md` - validation and CI gates
- `docs/alice-integration.md` - Alice discovery, packaging, and launch smoke
- `docs/instructor-missions.md` - instructor mission model
- `docs/student-missions.md` - student mission model
- `docs/github-pages.md` - publishing workflow

## Environment notes

`NODE_OPTIONS=--max-old-space-size=32768` is a preserved agent/tooling
preference for Node-based wrappers. The Rust CLI and MkDocs site do not require
Node.js.
