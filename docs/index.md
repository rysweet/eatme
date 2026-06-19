# eatme — End-to-end testing for Alice 3

`eatme` tests Alice 3 the way students and instructors actually use it. It
checks scene building, code editing, running animations, working with events,
and moving through the Alice curriculum on both the Java desktop app and the
TypeScript web port.

## What it tests

The suite covers the Alice.org learning path.

| Curriculum area | Example work |
| --- | --- |
| Getting started | First scene, first run, first lesson setup |
| Procedures and parameters | Reusable behavior and parameter passing |
| Functions and variables | Asking questions about the world and tracking state |
| Control flow | Loops, conditionals, choreography, and simulations |
| Events and interaction | Key presses, mouse actions, collision, and proximity |
| Collections | Arrays and grouped object behavior |
| Camera and audio | Camera movement, viewpoint, sound, and media cues |
| Games and stories | Score, timer, win/lose, branching narrative |
| Project management | Open, save, export, reopen, and evidence review |
| Instructor tools | Lesson prep, rubrics, and classroom handoff |
| Student workflow | Build, run, reflect, save, reopen, and share |

## How eatme works

Eatme has three testing layers:

- **Offline tests** validate scenario files, grading logic, and project parsing
  without launching Alice.
- **Desktop tests** launch the real Java Alice app when `EATME_REAL_ALICE=1` is
  set.
- **Web platform tests** hit the TypeScript web port when
  `EATME_WEB_PLATFORM=1` is set.

`alice-objects-first-world` is a desktop workflow scenario. It proves that a
project is created or opened, a visible object is added, the object is changed, a
movement procedure is edited, the world runs, the project is saved, the saved
project is reopened, and the reopened state still contains the expected object
and behavior evidence.

Launch-only Alice scenarios prove startup evidence for a scenario-labeled Alice
run. They do not score learner creativity, inspect private Alice implementation
details, or grade saved learner worlds. The objects-first workflow has its own
full-workflow evidence contract and still does not replace instructor judgment.

## Quick start

```bash
git clone https://github.com/rysweet/eatme.git
cd eatme
cargo build --workspace
cargo test --workspace
cargo run -q -p eatme-cli -- assets validate --json
```

## Start here

- [Installation](./installation.md)
- [CLI usage](./cli-usage.md)
- [Alice integration](./alice-integration.md)
- [Alice Objects-First Full Path](./alice-objects-first-full-path.md)
- [Alice Objects-First World](./alice-objects-first-world.md)
- [Web platform testing](./web-platform-testing.md)

## Documentation map

### Getting Started

- [Installation](./installation.md)
- [CLI usage](./cli-usage.md)

### Curriculum Scenarios

- [Scenario authoring](./scenario-authoring.md)
- [Alice Objects-First Full Path](./alice-objects-first-full-path.md)
- [Alice Objects-First World](./alice-objects-first-world.md)
- [Student missions](./student-missions.md)
- [Instructor missions](./instructor-missions.md)
- [Alice lesson smoke](./alice-lesson-smoke.md)
- [Student lesson E2E tests](./student-lesson-e2e-tests.md)
- [Code editor first run E2E test](./code-editor-first-run-e2e.md)
- [First-Lesson Grading Report](./first-lesson-grading-report.md)
- [Loops and conditionals grading](./loops-and-conditionals-grading.md)
- [Events and collision grading](./events-and-collision-grading.md)
- [Real-Alice lesson grading integration tests](./real-alice-lesson-grading-tests.md)
- [Creative assessment boundary](./creative-assessment-boundary.md)

### Testing

- [Validation and quality gates](./validation-quality-gates.md)
- [Web platform testing](./web-platform-testing.md)
- [Deterministic real-Alice smoke test](./deterministic-real-alice-smoke-test.md)
- [Real-Alice grading integration tests](./real-alice-grading-integration-tests.md)
- [Alice content coverage tests](./alice-content-coverage-tests.md)
- [Outside-in Alice test modules](./outside-in-alice-test-modules.md)
- [Post-focus screenshot evidence](./post-focus-screenshot-evidence.md)
- [Run window polling](./run-window-polling.md)
- [Edit procedure proof verification](./edit-procedure-proof-verification.md)

### Architecture

- [Grading module architecture](./grading-module-architecture.md)
- [Evidence artifact contract](./evidence-artifact-contract.md)
- [Alice Objects-First Full Path Reference](./alice-objects-first-full-path-reference.md)
- [Alice Objects-First World Reference](./alice-objects-first-world-reference.md)
- [Lesson readiness module boundary](./lesson-readiness-module-boundary.md)
- [First-lesson vertical slice](./first-lesson-vertical-slice.md)
- [First-Lesson Evidence Readiness](./first-lesson-evidence-readiness.md)
- [Lesson session readiness](./lesson-session-readiness.md)
- [Starter project preflight evidence](./starter-project-preflight-evidence.md)
- [Save/reopen readiness](./save-reopen-readiness.md)
- [Import/export workflow](./import-export-workflow.md)
- [Sharing platform readiness](./sharing-platform-readiness.md)
- [Generated asset consistency](./generated-asset-consistency.md)
- [Gadugi adapters](./gadugi-adapters.md)
- [Persona assets](./persona-assets.md)
- [Live studio workshop evidence contract](./live-studio-workshop-evidence.md)

### Integrations and publishing

- [Alice integration](./alice-integration.md)
- [GitHub Pages](./github-pages.md)

### Workflows and readiness

- [Default-workflow PR readiness](./default-workflow-pr-readiness.md)
- [PR #199 Recovery Workflow](./pr-199-recovery-workflow.md)
- [PR #160 Gap-Reporting Readiness](./pr-160-gap-reporting-readiness.md)

### Existing project note

- [Implementation plan](./implementation-plan.md)
