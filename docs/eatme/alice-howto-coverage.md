# Alice.org HowTo coverage

Eatme covers Alice.org HowTo content with scenario files that describe real
student or instructor actions, expected visible results, and platform evidence.
The source files live in `assets/scenarios/eatme/`; generated Gadugi adapters
live in `assets/scenarios/gadugi/` and are refreshed from those source files.

## Contents

- [Coverage rule](#coverage-rule)
- [Validation targets](#validation-targets)
- [Configuration](#configuration)
- [Run a HowTo scenario](#run-a-howto-scenario)
- [Coverage inventory](#coverage-inventory)
- [Adding or updating coverage](#adding-or-updating-coverage)
- [Bug workflow](#bug-workflow)

## Coverage rule

A HowTo item is covered only when its scenario walks through meaningful Alice
user steps and checks the result. Opening the app by itself is not enough.

Each covered scenario records:

1. The Alice.org HowTo or local RabbitHole lesson source that the scenario maps
   to.
2. The user role: student, instructor, or both.
3. The action path: create or open a project, add or change scene content, edit
   code or lesson material, run the world when the lesson requires it, and save
   or review evidence.
4. The expected result: visible scene state, procedure/function state, event
   behavior, saved artifact, reflection artifact, or instructor handoff.
5. The platform result for RabbitHole and LookingGlass.

The RabbitHole-vs-LookingGlass closure source is
`assets/parity/rabbithole-lookingglass-journey-matrix.yaml`. This page is the
human inventory; the matrix is the executable closure contract.

## Validation targets

| Target | Path | Used for |
| --- | --- | --- |
| RabbitHole | `$ALICE_HOME` | Desktop Alice behavior, save/reopen, UI action hooks, instructor-facing Alice workflows |
| LookingGlass | `$LOOKINGGLASS_HOME` | Supported web tasks and generated evidence artifacts |
| eatme | current repository | Scenario source, validation, generated Gadugi assets, and evidence comparison |

LookingGlass is validated only for flows it supports. Desktop-only items stay
marked as "not supported in LookingGlass" instead of pretending web parity
exists.

## Configuration

Use these environment variables for local validation:

```bash
export ALICE_HOME=/path/to/alice
export LOOKINGGLASS_HOME=/absolute/path/to/LookingGlass
export ALICE_WEB_URL=http://127.0.0.1:3099
export NODE_OPTIONS=--max-old-space-size=32768
```

Build and start LookingGlass when a scenario has web coverage:

```bash
cd "$LOOKINGGLASS_HOME"
npm install
npm run build:server
node dist-server/cli.js serve --port 3099 --evidence-dir ./evidence
```

The health endpoint identifies the LookingGlass server with the web-prototype
runtime token:

```bash
curl http://127.0.0.1:3099/api/health
```

```json
{
  "status": "running",
  "launched": false,
  "runtime": "lookingglass"
}
```

Validate scenario files and generated assets from the eatme repository:

```bash
cargo run -q -p eatme-cli -- assets validate --json
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
```

## Run a HowTo scenario

Run a RabbitHole scenario with real user steps:

```bash
EATME_REAL_ALICE=1 cargo run -q -p eatme-cli -- alice run-howto \
  --alice-home "$ALICE_HOME" \
  --scenario building-a-scene-first-world \
  --run-id local-building-scene \
  --runs-dir runs \
  --timeout 1800 \
  --json
```

Run the same scenario against LookingGlass when the inventory marks it as
supported:

```bash
EATME_WEB_PLATFORM=1 ALICE_WEB_URL="${ALICE_WEB_URL:-http://localhost:3099}" \
  cargo test -p eatme-alice --test web_platform_curriculum_e2e -- --test-threads=1
EATME_WEB_PLATFORM=1 ALICE_WEB_URL="${ALICE_WEB_URL:-http://localhost:3099}" \
  cargo test -p eatme-alice --test web_platform_setup_readiness_e2e -- --test-threads=1
```

The run passes only when the scenario assertions match the expected Alice user
result. A platform setup problem is reported as blocked. A product behavior
problem is reported as a bug and linked to a GitHub issue.

## Coverage inventory

This table lists HowTo and curriculum-journey coverage. It excludes Alice
startup checks because opening the app by itself checks readiness, not
Alice.org HowTo coverage.

| HowTo area | Scenario | User journey covered | RabbitHole | LookingGlass |
| --- | --- | --- | --- | --- |
| Setup and first use | `setup-preflight-ready-to-create` | Instructor checks install readiness, confirms required tools, and prepares a classroom-ready create-project path. | Partial: editable scenario asset validates; missing instructor agentic acceptance output evidence | Covered: web setup/readiness API evidence is covered; desktop setup checks are not claimed |
| Setup and first use | `setup-support-lab-readiness` | Support helper reproduces setup guidance, diagnoses missing desktop prerequisites, and records repair guidance. | Covered | Covered: web setup/readiness API evidence is covered; desktop setup checks are not claimed |
| Setup and first use | `instructor-classroom-setup-readiness` | Instructor prepares lab machines, confirms Alice can create a starter world, and saves setup evidence. | Partial: editable scenario asset validates; missing instructor agentic checklist/fallback/student-note output evidence | Covered: web setup/readiness API evidence is covered; desktop setup checks are not claimed |
| Setup and first use | `instructor-student-launch-evidence-handoff` | Instructor hands students a verified launch package with expected next actions and evidence review notes. | Partial: editable scenario asset validates; missing instructor agentic student handoff output evidence | Covered: web setup/readiness API evidence is covered; desktop setup checks are not claimed |
| First scene | `building-a-scene-first-world` | Student creates a first scene, adds a visible object, adjusts it, runs the world, and saves the project. | Covered | Covered: web launch, add-object, transform/adjust, run, and save evidence are covered |
| First scene | `alice-objects-first-world` | Student creates or opens an objects-first world, places and changes an object, edits movement, runs, saves, reopens, and verifies persistence. | Covered | Covered: object placement, transform/adjust, procedure edit, run, save, reopen, persistence, and export evidence are covered |
| First scene | `alice-objects-first-full-path` | Automation performs the full objects-first path with object placement, transform, procedure edit, run, save, reopen, and persistence assertions. | Covered | Covered: object placement, transform, procedure edit, run, save, reopen, persistence, and export evidence are covered |
| First lessons | `first-lessons-real-ui-actions` | Student completes first-lesson UI actions and records object, code-edit, run, and save proof. | Covered | Covered: browser UI action evidence covers object placement, code edit, run, visible evidence capture, save, and reopen; desktop Alice UI actions are not claimed |
| Code editor | `code-editor-first-run` | Student opens code editing, changes a first procedure, predicts behavior, runs the world, and checks the result. | Covered | Covered |
| Procedures | `reusable-methods-and-parameters` | Student creates reusable behavior, adds parameters, calls the method, and checks that the object behavior changes. | Covered | Covered |
| Procedures | `alien-linguist-parameter-dialogue` | Student builds a parameterized dialogue, changes arguments, and verifies different visible speech results. | Covered | Covered |
| Procedures | `creature-choreography-loop-lab` | Student composes repeated creature movement with procedures and verifies the choreography after running. | Covered | Covered |
| Functions | `functions-as-questions-about-the-world` | Student adds a function-style question, uses the answer in behavior, and checks the observed branch or value. | Covered | Covered |
| Variables | `variables-scorekeeper-timekeeper` | Student creates score and timer variables, updates them during a run, and checks the displayed or recorded state. | Covered | Covered |
| Data types | `data-types-alice-catalog` | Student uses Alice data categories in a project and verifies that typed values drive behavior correctly. | Covered | Covered |
| Math expressions | `arithmetic-expressions-math-playground` | Student builds arithmetic expressions, runs the world, and checks movement, timing, or score results. | Covered | Covered |
| Comparisons | `relational-expressions-comparison-lab` | Student builds comparison expressions and verifies different branches for true and false cases. | Covered | Covered |
| Control flow | `loops-and-conditionals-mini-challenge` | Student combines repetition and a conditional rule, runs the world, and checks both repeated and branched behavior. | Covered | Covered |
| Control flow | `weather-wizard-conditional-theater` | Student creates a weather-driven scene branch and verifies the selected animation path. | Covered | Covered |
| Control flow | `ecosystem-balance-loop-simulation` | Student models repeated ecosystem changes, runs the simulation, and checks state changes over time. | Covered | Covered |
| Events | `events-collision-proximity-game` | Student registers interaction, collision, and proximity events, fires them, and verifies the correct handler runs. | Covered | Covered |
| Events | `mars-rover-proximity-mission` | Student builds a proximity mission, moves actors into range, and checks the triggered result. | Covered | Covered |
| Collections | `arrays-collection-choreography` | Student creates a collection-driven choreography, iterates through objects, and checks that each object participates. | Covered | Covered |
| Concurrency | `time-travel-recipe-sequencing` | Student compares ordered and simultaneous actions and verifies timing-sensitive behavior. | Covered | Covered |
| Comments | `using-comments-code-clarity` | Student adds meaningful comments to a procedure and verifies that the project behavior remains intact. | Covered | Covered |
| Debugging | `lost-robot-debug-museum` | Student follows a broken-world debugging path, fixes the behavior, reruns, and records the correction. | Covered | Covered |
| Games | `game-score-timer-win-lose-loop` | Student builds a score, timer, and win/lose loop, runs the game, and verifies state transitions. | Covered | Covered |
| Narrative | `mythic-choice-event-tree` | Student builds a branching story with event choices and checks that each choice reaches the expected scene result. | Covered | Covered |
| Design process | `design-process-story-or-game` | Student plans, builds, playtests, revises, and records a story or game artifact. | Covered | Covered: bounded LookingGlass REST evidence records plan, build, playtest, revise, and review phases |
| Hour of Code | `hour-of-code-studio-kickoff` | Student follows an Hour of Code starter path, creates a simple world, runs it, and saves evidence. | Covered | Covered |
| Hour of Code | `workshop-facilitator-live-studio` | Facilitator runs a live studio workshop, checks participant handoff material, and records review prompts. | Covered | Covered: local LookingGlass live-studio runtime records synchronized facilitator session, participant orchestration, and handoff evidence |
| Camera | `vr-camera-perspective-tour` | Student changes camera perspective, runs the world, and checks the expected viewpoint result. | Covered | Covered |
| VR | `vr-camera-locomotion-journey` | Student records VR camera comfort planning and checks bounded movement-comfort evidence. | Covered | Partial: bounded browser camera comfort API evidence is covered; missing headset/native VR locomotion evidence |
| VR | `vr-player-comfort-playtest` | Student uses VR/player comfort guidance to collect observed notes before making playtest or revision claims. | Covered | Partial: bounded browser WebXR/player comfort boundary evidence and submitted player-comfort session/revision-loop evidence are covered; missing independently observed real headset session artifact from live hardware |
| Audio and media | `media-audio-cue-storyboard` | Student authors an audio cue storyboard, runs native Web Audio playback, and verifies cue timing evidence. | Covered | Covered: native Web Audio playback and audio authoring evidence covered by LookingGlass main contract test |
| Audio and media | `audio-camera-and-export-sharecase` | Student combines camera, native audio playback, export, and native Web Share capability evidence for a sharecase package. | Covered | Covered: camera/export path, native Web Audio playback, and native Web Share capability evidence covered by LookingGlass main contract tests |
| Import/export | `starter-project-open-save-export-preflight` | Student opens a starter project, saves it, exports it, and verifies the exported artifact. | Covered: desktop launch/log/window/screenshot/readiness evidence is covered; save/export artifact verification is not claimed | Covered: web open, save, reopen, and export project evidence is covered; desktop starter-gallery selection is not claimed |
| Import/export | `model-texture-import-checkpoint` | Student imports a model or texture, applies it, saves the project, and verifies the resource remains available. | Covered | Covered: LookingGlass main contains imported model, texture, safe resource, export, and reopen persistence contract tests |
| Alice 2 migration | `alice-2-migration-bridge` | Student opens migrated Alice 2 content, checks compatibility guidance, and records the converted result. | Covered | Covered: scoped empty Alice 2 World conversion is covered; broader Alice 2 projects remain guidance-only |
| Classes | `modified-class-portability` | Student saves a modified class, imports it into another project, and checks that behavior travels with it. | Covered | Partial: browser UI class behavior export/import/save/reopen evidence is covered via LookingGlass #266; missing desktop Alice UI class portability evidence |
| Accessibility | `accessibility-rescue-camera-captions` | Student uses camera/caption guidance and verifies the project remains understandable and navigable. | Covered | Covered for browser accessibility caption evidence |
| Accessibility | `ide-accessibility-parity` | Reviewer checks labels, keyboard access, contrast, and zoom behavior across the editor. | Covered | Covered |
| Performance | `ide-performance-parity` | Reviewer opens a large project, performs editing and run actions, and checks that interaction remains usable. | Covered | Covered |
| Instructor planning | `instructor-alice-concept-map` | Instructor maps Alice concepts to lesson steps and checks the handoff against student evidence. | Covered | Covered |
| Instructor planning | `instructor-exercise-builder` | Instructor builds an exercise, adds expected student evidence, and verifies the rubric fits the task. | Covered | Covered |
| Instructor planning | `instructor-lesson-materials-remix` | Instructor remixes lesson materials, produces a student handout, and checks that the activity still maps to Alice actions. | Covered | Covered |
| Instructor planning | `curriculum-sequence-remix-pack` | Instructor reorganizes a curriculum sequence and checks prerequisite and assessment coverage. | Covered | Covered |
| Instructor review | `instructor-student-outcomes-rubric` | Instructor reviews student outcomes against a rubric and records concept, process, creativity, and reflection evidence. | Covered | Covered |
| Student review | `student-reflection-artifact-review` | Student reviews a saved artifact, explains expected versus actual behavior, and records revision notes. | Covered | Covered |
| Sharing | `student-artifact-package-share-evidence` | Student packages project, screenshot, notes, and share evidence for review. | Covered | Covered |
| Sharing | `classroom-gallery-walk-and-rubric` | Class reviews projects in a gallery walk and uses a rubric to record feedback. | Covered | Covered for web gallery review, rubric criteria, and live studio support evidence; rubric recording remains unsupported |
| Sharing | `teacher-community-sharing-loop` | Teacher packages a reusable classroom resource and checks community-sharing metadata. | Covered | Covered: package/share metadata and local LookingGlass community platform evidence are recorded without claiming external cloud deployment |
| Data storytelling | `neighborhood-data-story` | Student turns local data into an Alice story and verifies that values drive scene behavior. | Covered | Covered |

## Adding or updating coverage

1. Edit the source scenario in `assets/scenarios/eatme/`.
2. Include `resource_basis` entries for the Alice.org or local RabbitHole source.
3. Describe the user steps in plain language. Each step must include an expected
   visible result or evidence artifact.
4. Mark platform support explicitly:
   `RabbitHole covered`, `LookingGlass covered`, `LookingGlass partial`, or
   `not supported in LookingGlass`. Partial rows must name the bounded evidence,
   the missing evidence, and any upstream PR or default-branch dependency.
5. Validate the source scenario:

   ```bash
   cargo run -q -p eatme-cli -- assets validate \
     --path assets/scenarios/eatme/building-a-scene-first-world.yaml \
     --json
   ```

6. Refresh generated Gadugi assets:

   ```bash
   cargo run -q -p eatme-cli -- assets generate-gadugi --json
   ```

7. Run the RabbitHole path and, when applicable, the LookingGlass path before the
   scenario is considered covered.

## Bug workflow

When validation finds a product bug, file a GitHub issue before starting the fix.
The issue includes:

- Scenario id
- Platform: RabbitHole, LookingGlass, or both
- Reproduction command
- Expected Alice user result
- Actual result
- Evidence paths with secrets and local-only data removed

The fix is tracked in a separate default-workflow PR linked to that issue. The
scenario remains covered only when the linked fix restores the expected user
journey on the affected platform.
