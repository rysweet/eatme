# alice.eatme — Agentic Alice user crew

Editable outside-in design assets for an instructor/student Alice crew. These assets are intended for gadugi-agentic-test custom agents: personas define motives and observable behaviors; scenarios define user-facing evidence, not brittle implementation details.

## Asset files

- `assets/personas/alice-user-crew.yaml` — canonical editable YAML: asset shapes, constituency coverage, instructor/student personality prompt cards, persona list, core scenarios grounded in Alice resources, and 12 creative scenarios.
- `assets/scenarios/eatme/*.yaml` — canonical editable eatme scenarios for the real-Alice launch smoke baseline, Alice.org-grounded lesson scenarios, desktop journey preflights, current outside-in lesson coverage, setup/migration/import/VR-player scenarios, and instructor agentic flows.
- `assets/scenarios/gadugi/*.yaml` — gadugi-compatible adapters generated from canonical eatme scenarios, plus hand-authored CLI regression contracts such as validation exit codes.
- `docs/alice-lesson-smoke.md` — usage, CLI, schema, configuration, and tutorial documentation for lesson smoke scenarios.

## How to use with agentic tests

1. Pick a persona pair: one instructor persona and one or more student personas.
2. Pick a scenario by `id` and pass `agentic_test_prompt` to the custom agent.
3. Judge output with `acceptance_probes` and `observable_behaviors`.
4. Reject outputs that violate `avoid`, especially exact UI selectors, hidden implementation assertions, one-path-only lessons, or visual-polish-only grading.

For deterministic desktop smoke coverage, use the editable scenario assets under
`assets/scenarios/eatme/` and the `alice launch-smoke --scenario <id>` command.
The lesson scenarios are documented in
[`docs/alice-lesson-smoke.md`](alice-lesson-smoke.md).

For instructor modernization pressure, use the editable
`kind: instructor_agentic_flow` assets under `assets/scenarios/eatme/` with
their paired `assets/scenarios/gadugi/` adapters. These flows intentionally
stay at the natural-language prompt, acceptance probe, and rubric boundary so a
non-coder can maintain lesson intent without touching Rust.

For the first real Alice lesson scenario, use
`assets/scenarios/eatme/first-lessons-real-ui-actions.yaml` as the student
launch/action-contract source of truth. It records scenario-labeled
manifest/log/window/screenshot evidence and `ui-action-contract.json`
expectations; it does not drive an entire lesson through the Alice interface,
score learner creativity, or grade saved learner worlds.

For instructor remix work, use
`assets/scenarios/eatme/instructor-lesson-materials-remix.yaml` as the
lesson-material/remix evidence contract. It keeps teacher plan, student handout,
exit ticket, and review/remix probes discoverable without claiming automated
creative grading or learner-world assessment.

For workshop facilitation coverage, use
`assets/scenarios/eatme/workshop-facilitator-live-studio.yaml` and its generated
Gadugi adapter. The scenario connects workshop facilitator personas to a
reviewable instructor agentic flow without claiming desktop automation or
automated grading.

For teacher-community sharing coverage, use
`assets/scenarios/eatme/teacher-community-sharing-loop.yaml` and its generated
Gadugi adapter. The scenario connects the teacher-community curator persona to a
teacher-facing share card, classroom handoff note, and remix feedback prompt
with local LookingGlass community platform evidence and without claiming an
external cloud community deployment.

## QA-team outside-in test shape

```yaml
scenario_under_test: code-editor-first-run
agent_role: instructor-agent
student_personas:
  - curious-novice
  - reflective-debugger
prompt: "Use the scenario's agentic_test_prompt."
expected_evidence:
  - Student predicts visible behavior before running.
  - Student observes the world and edits one thing.
  - Final response assesses learning via artifact behavior and explanation.
reject_if:
  - Output depends on exact UI coordinates or private implementation details.
  - Output grades only visual polish.
```

## Editable personality assets

`assets/personas/alice-user-crew.yaml` defines top-level `personality_assets`
for instructor prompt cards, student reflection cards, and pairing patterns.
They are intentionally editable YAML prompts: agents can tune teaching voice,
learner reflection shape, and pairing strategy without changing Rust or the
launch harness.

## Persona roster

### Instructor crew

| Persona | Purpose |
| --- | --- |
| `concept-cartographer` | Sequences Alice experiences into teachable programming concepts. |
| `exercise-forger` | Creates meaningful exercises with scaffolds, choice, and stretch paths. |
| `studio-facilitator` | Runs lessons as studio cycles: build, run, critique, revise. |
| `debug-coach` | Turns run/debug into prediction, observation, hypothesis, revision. |
| `assessment-curator` | Assesses concept evidence, creativity, process, and reflection. |
| `classroom-orchestrator` | Handles setup, sharing, fallback plans, and classroom logistics. |
| `curriculum-pathway-designer` | Sequences Alice resources into editable units and concept progressions. |
| `setup-support-specialist` | Coordinates install, launch, graphics, storage, and fallback readiness. |
| `workshop-facilitator` | Runs short workshop/studio blocks with checkpoints, helpers, and share-outs. |
| `alice-2-migration-mentor` | Bridges Alice 2 lesson intent into current Alice 3 workflows. |
| `teacher-community-curator` | Packages shareable/remixable teacher-community assets with context and attribution. |

### Student crew

| Persona | Purpose |
| --- | --- |
| `curious-novice` | Learns through safe cause-and-effect experiments. |
| `creative-storyteller` | Uses programming to express narrative, camera, audio, and character choices. |
| `playful-tinkerer` | Learns by trying surprising variations and treating bugs as clues. |
| `systems-puzzle-solver` | Builds rule-based games/simulations with visible logic. |
| `reflective-debugger` | Practices expected-vs-actual reasoning and small repair tests. |
| `collaborative-peer-mentor` | Helps peers through questions and evidence, not takeover. |
| `accessibility-advocate` | Tests whether worlds communicate across audience needs and constraints. |
| `vr-player-tester` | Playtests worlds for comfort, orientation, discoverability, and fallback access. |
| `media-audio-creator` | Uses audio, camera, timing, captions, and media cues for audience meaning. |
| `model-texture-importer` | Imports or falls back from external models/textures responsibly. |
| `data-detective` | Connects variables, data types, arrays, and visible world state. |
| `immersive-camera-director` | Designs camera/VR perspective with audience clarity and fallbacks. |
| `game-narrative-designer` | Builds small playable stories/games with choices, state, and playtest evidence. |

## Student outside-in flow prompt cards

The persona asset includes top-level `student_outside_in_flow_assets` cards
editable by non-coders:

- `curiosity-loop-card` — prediction, observation, surprise, next experiment.
- `data-state-card` — variables, data types, arrays, and visible state evidence.
- `interactive-playtest-card` — trigger, state/condition, feedback, and peer revision.
- `camera-vr-fallback-card` — camera/VR perspective with non-VR classroom fallback.
- `setup-migration-readiness-card` — setup blockers and Alice 2 migration bridges with learner-safe fallback evidence.
- `import-fallback-checkpoint-card` — responsible model/texture provenance, visual checks, and fallback asset evidence.
- `artifact-reflection-review-card` — student-owned Alice action, visible artifact behavior, run result, and next revision evidence.

## Constituency coverage

`constituency_coverage` in `assets/personas/alice-user-crew.yaml` is validated so
non-coder editors can add or revise personas/scenarios without touching Rust. The
references are validated against the persona crew's own scenario inventory; they
are coverage markers and do not automatically mean a matching file already exists
under `assets/scenarios/eatme/`. The current constituency table now has
matching standalone eatme scenario assets and generated Gadugi adapters for each
listed scenario.

It requires persona and scenario references for:

| Constituency | Persona | Scenario |
| --- | --- | --- |
| Curriculum designers | `curriculum-pathway-designer` | `curriculum-sequence-remix-pack` |
| IT/setup support | `setup-support-specialist` | `setup-support-lab-readiness` |
| Workshop facilitators | `workshop-facilitator` | `workshop-facilitator-live-studio` |
| VR/player users | `vr-player-tester` | `vr-player-comfort-playtest` |
| Media/audio creators | `media-audio-creator` | `media-audio-cue-storyboard` |
| Model/texture import users | `model-texture-importer` | `model-texture-import-checkpoint` |
| Alice 2 migration users | `alice-2-migration-mentor` | `alice-2-migration-bridge` |
| Teacher-community sharing | `teacher-community-curator` | `teacher-community-sharing-loop` |

## Persona-crew scenario coverage

The following ids are scenario references inside
`assets/personas/alice-user-crew.yaml`. Some are also committed standalone
scenario assets under `assets/scenarios/eatme/`; others are design-forward
coverage markers that should become standalone assets as the outside-in Alice QA
expansion is built. The committed standalone assets named in the constituency
coverage table each have generated Gadugi adapters.

| Coverage area | Scenario IDs |
| --- | --- |
| Setup | `setup-preflight-ready-to-create`, `starter-project-open-save-export-preflight` |
| Lessons | `hour-of-code-studio-kickoff`, `building-a-scene-first-world`, `code-editor-first-run`, `reusable-methods-and-parameters`, `functions-as-questions-about-the-world`, `loops-and-conditionals-mini-challenge`, `events-collision-proximity-game`, `game-score-timer-win-lose-loop`, `vr-camera-locomotion-journey`, `design-process-story-or-game`, `variables-scorekeeper-timekeeper`, `arrays-collection-choreography`, `vr-camera-perspective-tour` |
| World creation | `hour-of-code-studio-kickoff`, `building-a-scene-first-world`, `design-process-story-or-game`, `vr-camera-locomotion-journey`, `audio-camera-and-export-sharecase`, `arrays-collection-choreography`, `vr-camera-perspective-tour` |
| Run/debug | `hour-of-code-studio-kickoff`, `code-editor-first-run`, `loops-and-conditionals-mini-challenge`, `vr-camera-locomotion-journey`, `lost-robot-debug-museum`, `arrays-collection-choreography` |
| Export/share | `audio-camera-and-export-sharecase`, `modified-class-portability`, `classroom-gallery-walk-and-rubric`, `creature-choreography-loop-lab` |
| Classroom use | `setup-preflight-ready-to-create`, `hour-of-code-studio-kickoff`, `classroom-gallery-walk-and-rubric`, `mars-rover-proximity-mission`, `game-score-timer-win-lose-loop`, `vr-camera-perspective-tour` |
| Curriculum design | `curriculum-sequence-remix-pack` |
| IT/setup support | `setup-support-lab-readiness` |
| Workshops | `workshop-facilitator-live-studio` |
| VR/player experience | `vr-player-comfort-playtest` |
| Media/audio creation | `media-audio-cue-storyboard` |
| Model/texture import | `model-texture-import-checkpoint` |
| Alice 2 migration | `alice-2-migration-bridge` |
| Teacher-community sharing | `teacher-community-sharing-loop` |

## Desktop smoke and outside-in scenario assets

Editable eatme + gadugi scenario assets define the deterministic desktop
boundary for Alice.org-grounded lessons, student creative scenarios, and explicit QA
contracts.

| Scenario IDs | User-facing outcome |
| --- | --- |
| `building-a-scene-first-world`, `code-editor-first-run`, `reusable-methods-and-parameters`, `functions-as-questions-about-the-world`, `loops-and-conditionals-mini-challenge`, `events-collision-proximity-game`, `hour-of-code-studio-kickoff` | Alice.org lesson scenarios that prove the real desktop harness can start Alice before agents judge lesson intent. |
| `starter-project-open-save-export-preflight` | Starter-project preflight that opens the bundled project before save, reopen, or export coverage is claimed. |
| `game-score-timer-win-lose-loop`, `variables-scorekeeper-timekeeper`, `arrays-collection-choreography` | Student data/state scenarios for visible variables, score/time rules, arrays, item order, and boundary tests. |
| `mythic-choice-event-tree` | Student interactive narrative scenario for player triggers, state or condition checks, feedback, and alternate path playtests. |
| `vr-camera-locomotion-journey`, `vr-camera-perspective-tour` | Camera and VR-perspective scenarios that record VR availability and require non-VR fallback evidence when classroom hardware is unavailable. |
| `first-lessons-real-ui-actions` | Real UI action contract that detects the Alice window. Without an Alice-side placement hook it fails with `ui_action_automation_unimplemented`; with object placement proof it moves to `ui_action_remaining_steps_unimplemented` and names the missing procedure-edit contract before running or saving can be claimed. |
| `modified-class-portability` | Class portability contract requiring before-export, destination-import, and after-import behavior evidence before a shared modified class is trusted. |

Standalone outside-in Alice QA scenario assets and generated Gadugi adapters
include:

| Scenario ID | User-facing outcome |
| --- | --- |
| `classroom-gallery-walk-and-rubric` | Instructor/student gallery-walk scenario for student-visible rubric evidence, peer feedback, creator response, revision checkpoints, and human review boundaries. |
| `vr-player-comfort-playtest` | VR/player comfort scenario for orientation, locomotion comfort, discoverability, peer feedback, and desktop fallback evidence. |
| `media-audio-cue-storyboard` | Media/audio cue scenario for sound, timing, camera, captions, prediction, run evidence, accessibility fallback, and revision reflection. |
| `model-texture-import-checkpoint` | Model/texture import scenario for source, license, scale, orientation, texture visibility, accessibility, and fallback evidence. |
| `setup-support-lab-readiness` | IT/setup-support scenario for install, Java, graphics, storage, accounts, and fallback readiness. |
| `alice-2-migration-bridge` | Migration scenario that maps Alice 2 lesson intent into Alice 3 workflows with visible student evidence. |
| `workshop-facilitator-live-studio` | Workshop facilitation scenario for checkpoint evidence, helper roles, recovery moves, student-owned action notes, and a final share-out. |
| `student-artifact-package-share-evidence` | Student artifact sharing scenario for artifact references, student explanation, classroom context or attribution, next revision, and human review boundaries. |
| `teacher-community-sharing-loop` | Teacher-community sharing scenario for share cards, classroom handoff notes, attribution, student evidence, accessibility notes, and remix feedback prompts. |
| `curriculum-sequence-remix-pack` | Curriculum design scenario for sequencing committed Alice assets with prerequisites, pacing, swap points, fallback notes, and visible student evidence. |

Launch-smoke standalone scenarios route runtime through
`EATME_REAL_ALICE=1 cargo run -q -p eatme-cli -- alice launch-smoke --scenario <id>`
and keep gadugi at the manifest-evidence boundary. The media/audio storyboard
scenario stays at the editable prompt, probe, and student-evidence boundary.
The YAML contracts describe the user outcome agents must inspect; launch smoke
evidence does not substitute for unimplemented user-interface, VR, or
export/import automation. `real-alice-launch-smoke` remains the baseline
manifest/log/window/screenshot proof only; it does not drive an entire lesson
through the Alice interface, score learner creativity, or grade saved learner
worlds.

## Instructor agentic flow assets

Editable eatme + gadugi agentic-flow assets cover the instructor goals that
Alice modernization work pressures first:

| Scenario ID | Instructor goal | Grounding |
| --- | --- | --- |
| `instructor-exercise-builder` | Create exercises with concept focus, scaffolds, student choice, and visible evidence. | Alice 3 lessons list; Programming in Alice. |
| `instructor-lesson-materials-remix` | Prepare teacher plan, student handout, and exit ticket from Alice.org resources. | Building A Scene; Alice 3 resource categories. |
| `instructor-alice-concept-map` | Map Alice actions to transferable CS vocabulary and misconception checks. | Programming in Alice; Alice 3 lessons list. |
| `instructor-student-outcomes-rubric` | Check outcomes with concept, creativity, process, and reflection rubric evidence. | Alice 3 resource categories; Building A Scene. |
| `instructor-classroom-setup-readiness` | Prepare setup checklist, student-facing note, and fallback plan. | Alice 3 setup/download; Alice resources overview. |
| `design-process-story-or-game` | Guide students through a structured design process before coding: story-vs-game framing, scene-sketch card, and design-to-code bridge card. | Programming in Alice; Alice 3 lessons list. |
| `setup-preflight-ready-to-create` | Run a device readiness check before the first creation lesson: setup readiness checklist, student self-check card, and fallback path guide for no-install options. | Alice 3 download page; Alice resources overview. |

The persona crew also defines these additional outside-in coverage areas:

| Scenario ID | Implementation role | Instructor goal | Grounding |
| --- | --- | --- | --- |
| `setup-support-lab-readiness` | Existing standalone scenario | Prepare a lab readiness runbook with explicit dependency, graphics, storage, and fallback evidence. | Alice 3 setup/download; Alice resources overview. |
| `alice-2-migration-bridge` | Existing standalone scenario | Convert Alice 2 lesson intent into Alice 3 classroom steps and visible evidence. | Alice resources overview; Alice 3 resource categories. |
| `vr-player-comfort-playtest` | Existing standalone scenario | Facilitate a short VR/player comfort playtest with helper roles and a non-VR path. | Design Process Virtual Reality; Moving The Camera. |
| `model-texture-import-checkpoint` | Existing standalone scenario | Review external model/texture use through source, license, scale, orientation, texture, and fallback checks. | Alice 3 resource categories; Building A Scene. |
| `workshop-facilitator-live-studio` | Existing standalone scenario | Facilitate a short live studio workshop with checkpoint evidence, helper roles, recovery moves, and a final share. | Alice 3 resource categories; Alice 3 lessons list. |
| `teacher-community-sharing-loop` | Existing standalone scenario | Package a teacher-facing share card, classroom handoff note, and remix feedback prompt with attribution and student evidence expectations. | Alice resources overview; Alice 3 resource categories. |

Each committed asset exposes `resource_basis`, `agentic_test_prompt`,
`acceptance_criteria`, `acceptance_probes`, `rubric`, `avoid`, and expected
agentic outputs as YAML. The paired gadugi adapters run asset validation and an
`agentic_test` step instead of owning Alice desktop runtime details.

## Creative scenario roster

1. `weather-wizard-conditional-theater`
2. `lost-robot-debug-museum`
3. `alien-linguist-parameter-dialogue`
4. `ecosystem-balance-loop-simulation`
5. `time-travel-recipe-sequencing`
6. `mars-rover-proximity-mission`
7. `creature-choreography-loop-lab`
8. `neighborhood-data-story`
9. `accessibility-rescue-camera-captions`
10. `game-score-timer-win-lose-loop`
11. `mythic-choice-event-tree`

## Source map

Grounded in official Alice resource themes:

- Alice resources overview: <https://www.alice.org/resources/>
- Alice 3 resource categories: <https://www.alice.org/resources/alice-3/>
- Alice 3 lessons: <https://www.alice.org/resources/alice-3-lessons>
- Building A Scene: <https://www.alice.org/resources/lessons/building-a-scene/>
- Programming in Alice: <https://www.alice.org/resources/lessons/programming-in-alice/>
- Alice 3 setup/download: <https://www.alice.org/get-alice/alice-3/>
- Design Process Virtual Reality lesson: <https://www.alice.org/resources/lessons/design-process-virtual-reality/>
- Moving The Camera how-to: <https://www.alice.org/resources/how-tos/moving-the-camera/>
- Using Camera Markers how-to: <https://www.alice.org/resources/how-tos/using-camera-markers/>
- Using Camera Views how-to: <https://www.alice.org/resources/how-tos/using-camera-views/>
