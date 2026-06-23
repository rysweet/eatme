# eatme — End-to-End Test Suite for Alice 3

`eatme` tests Alice 3 the way students and instructors actually use it: building
scenes, writing procedures, running animations, handling events, and working
through the full [Alice.org](https://www.alice.org) curriculum. It runs real
end-to-end workflows against both the Java desktop application and LookingGlass,
the TypeScript web port.

## What it tests

The test suite covers **57 canonical EatMe scenario definitions** spanning every concept taught
on Alice.org plus setup and readiness checks:

| Curriculum Area | Example Scenarios |
|---|---|
| **Getting started** | Building a first scene, code editor basics, first lessons |
| **Procedures & parameters** | Reusable methods, parameter passing, alien linguist dialogue |
| **Functions** | Functions as questions about the world |
| **Variables** | Scorekeeper, timekeeper |
| **Control flow** | Loops, conditionals, creature choreography, ecosystem simulation |
| **Events & interaction** | Mouse clicks, key presses, collision, proximity detection |
| **Concurrent execution** | doInOrder, doTogether, time-travel recipe sequencing |
| **Arrays** | Collection choreography |
| **Camera & audio** | VR camera perspectives, locomotion, audio cues |
| **OOP & inheritance** | Modified class portability, custom classes |
| **Games & narrative** | Score/timer/win-lose loops, mythic choice event trees |
| **Comments & code clarity** | Using comments effectively |
| **Project management** | Open, save, export, import models/textures |
| **Debugging** | Lost robot debug museum |
| **Accessibility** | Screen reader support, keyboard navigation, high contrast |
| **Instructor tools** | Classroom setup, exercise building, rubrics, grading |
| **Student workflow** | Artifact review, reflection, portfolio sharing |

Each scenario defines the **steps a student or instructor would take**, the
**expected outcomes**, and the **acceptance criteria** — all in plain YAML.

## How it works

**Offline tests** (~1,400 tests) validate scenario structure, curriculum
coverage, A3P project file parsing, AST round-trips, grading pipelines, and
quality scoring — no running Alice instance needed.

**Desktop tests** (opt-in via `EATME_REAL_ALICE=1`) launch the real Java Alice
application under a virtual display, walk through lesson workflows, and capture
evidence that each step completed correctly.

**Web platform tests** (opt-in via `EATME_WEB_PLATFORM=1`) run the same
curriculum scenarios against LookingGlass's REST API, covering 29
curriculum workflows including scene building, procedures, events, loops,
functions, variables, arrays, camera, audio, vehicles, joints, and more.

## Quick start

```bash
git clone https://github.com/rysweet/eatme.git
cd eatme
cargo build --workspace
```

Run all offline tests:

```bash
cargo test --workspace
```

Validate scenario files:

```bash
cargo run -q -p eatme-cli -- assets validate --json
```

## Running against real Alice

To run tests against the actual Alice desktop application, you need Java 21,
Maven, and a virtual display (Xvfb on Linux). Point `ALICE_HOME` at your Alice
checkout:

```bash
export ALICE_HOME="/path/to/alice3"

# Check that all dependencies are available
cargo run -q -p eatme-cli -- deps check --json

# Run a curriculum scenario end-to-end
EATME_REAL_ALICE=1 cargo run -q -p eatme-cli -- alice run-howto \
  --alice-home "${ALICE_HOME}" \
  --scenario building-a-scene-first-world \
  --run-id local-run \
  --runs-dir runs \
  --timeout 1800 \
  --json
```

## Running against LookingGlass

To run curriculum tests against LookingGlass:

```bash
# Start LookingGlass (in the alice-web-prototype repo)
cd /path/to/alice-web-prototype
npm run build:server
node dist-server/cli.js serve --port 3099 --evidence-dir ./evidence

# Run web platform tests
EATME_WEB_PLATFORM=1 cargo test --workspace
```

Set `ALICE_WEB_URL` to override the default `http://localhost:3099`.

## Repository layout

```text
assets/scenarios/eatme/     57 canonical scenario definitions (YAML)
assets/scenarios/gadugi/    Generated adapter scenarios (do not hand-edit)
crates/eatme-core/          Core types: AST, collaboration, commands
crates/eatme-alice/         Alice integration: discovery, launch, web adapter
crates/eatme-assets/        Scenario validation and curriculum coverage
crates/eatme-cli/           Command-line interface
crates/eatme-test-support/  Shared test utilities
docs/                       Documentation site (MkDocs)
scripts/                    Quality gate scripts
```

## Writing new scenarios

Scenarios live in `assets/scenarios/eatme/` as YAML files. Each one describes:

- **What the student/instructor does** (step-by-step actions)
- **What should happen** (expected outcomes at each step)
- **How to verify it worked** (acceptance criteria)

Edit the YAML, validate it, then regenerate the adapter files:

```bash
cargo run -q -p eatme-cli -- assets validate --json
cargo run -q -p eatme-cli -- assets generate-gadugi --json
```

## Quality gates

```bash
cargo fmt --check              # Code formatting
cargo clippy --workspace       # Lint checks
cargo test --workspace         # All tests pass
scripts/quality-gates.sh       # Full gate (formatting, lint, tests, coverage)
```

## Documentation site

```bash
python3 -m venv .venv && . .venv/bin/activate
pip install -r requirements-docs.txt
mkdocs build --strict
```
