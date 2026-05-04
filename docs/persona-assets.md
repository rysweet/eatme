# alice.eatme — Agentic Alice user crew

Editable outside-in design assets for an instructor/student Alice crew. These assets are intended for gadugi-agentic-test custom agents: personas define motives and observable behaviors; scenarios define user-facing evidence, not brittle implementation details.

## Asset files

- `assets/personas/alice-user-crew.yaml` — canonical editable YAML: asset shapes, instructor/student personality prompt cards, persona list, core scenarios grounded in Alice resources, and 10 creative new scenarios.
- `assets/scenarios/eatme/*.yaml` — canonical editable eatme scenarios for the real-Alice launch smoke baseline plus Alice.org-grounded lesson lanes.
- `assets/scenarios/gadugi/*.yaml` — gadugi-compatible adapters that invoke eatme CLI behavior and check manifest-level evidence only.
- `docs/alice-lesson-smoke.md` — usage, CLI, schema, configuration, and tutorial documentation for lesson smoke lanes.

## How to use with agentic tests

1. Pick a persona pair: one instructor persona and one or more student personas.
2. Pick a scenario by `id` and pass `agentic_test_prompt` to the custom agent.
3. Judge output with `acceptance_probes` and `observable_behaviors`.
4. Reject outputs that violate `avoid`, especially exact UI selectors, hidden implementation assertions, or one-path-only lessons.

For deterministic desktop smoke coverage, use the editable scenario assets under
`assets/scenarios/eatme/` and the `alice launch-smoke --scenario <id>` command.
The lesson lanes are documented in
[`docs/alice-lesson-smoke.md`](alice-lesson-smoke.md).

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

`assets/personas/alice-user-crew.yaml` now includes top-level
`personality_assets` for instructor prompt cards, student reflection cards, and
pairing patterns. They are intentionally editable YAML prompts: agents can tune
teaching voice, learner reflection shape, and pairing strategy without changing
Rust or the launch harness.

## Initial persona roster

### Instructor crew

| Persona | Purpose |
| --- | --- |
| `concept-cartographer` | Sequences Alice experiences into teachable programming concepts. |
| `exercise-forger` | Creates meaningful exercises with scaffolds, choice, and stretch paths. |
| `studio-facilitator` | Runs lessons as studio cycles: build, run, critique, revise. |
| `debug-coach` | Turns run/debug into prediction, observation, hypothesis, revision. |
| `assessment-curator` | Assesses concept evidence, creativity, process, and reflection. |
| `classroom-orchestrator` | Handles setup, sharing, fallback plans, and classroom logistics. |

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

## Core scenario coverage

| Coverage area | Scenario IDs |
| --- | --- |
| Setup | `setup-preflight-ready-to-create` |
| Lessons | `hour-of-code-studio-kickoff`, `building-a-scene-first-world`, `code-editor-first-run`, `reusable-methods-and-parameters`, `functions-as-questions-about-the-world`, `loops-and-conditionals-mini-challenge`, `events-collision-proximity-game`, `vr-camera-locomotion-journey`, `design-process-story-or-game`, `variables-scorekeeper-timekeeper` |
| World creation | `hour-of-code-studio-kickoff`, `building-a-scene-first-world`, `design-process-story-or-game`, `vr-camera-locomotion-journey`, `audio-camera-and-export-sharecase` |
| Run/debug | `hour-of-code-studio-kickoff`, `code-editor-first-run`, `loops-and-conditionals-mini-challenge`, `vr-camera-locomotion-journey`, `lost-robot-debug-museum` |
| Export/share | `audio-camera-and-export-sharecase`, `classroom-gallery-walk-and-rubric`, `creature-choreography-loop-lab` |
| Classroom use | `setup-preflight-ready-to-create`, `hour-of-code-studio-kickoff`, `classroom-gallery-walk-and-rubric`, `mars-rover-proximity-mission` |

## Hour of Code studio kickoff

Added editable eatme + gadugi scenario assets for
`hour-of-code-studio-kickoff`, grounded in Alice.org Hour of Code, Building a
Scene, and Programming in Alice resources. The lane captures instructor setup
and fallback moves, novice first-scene/first-animation prompts, and
evidence/reflection expectations in YAML so facilitators can tune the studio
kickoff without changing Rust.

## New Alice.org-grounded smoke scenario assets

Added editable eatme + gadugi scenario assets for:

1. `reusable-methods-and-parameters`
2. `functions-as-questions-about-the-world`
3. `loops-and-conditionals-mini-challenge`
4. `events-collision-proximity-game`
5. `vr-camera-locomotion-journey`

Each routes runtime through `EATME_REAL_ALICE=1 cargo run -q -p eatme-cli -- alice launch-smoke --scenario <id>` and keeps gadugi at the manifest-evidence boundary.
The VR camera lane also records whether real VR execution is available; when it
is not, expected evidence is the desktop launch manifest plus camera-marker,
viewpoint, and locomotion-comfort artifacts rather than a silent VR skip.

## 10 creative new scenarios

1. `weather-wizard-conditional-theater`
2. `lost-robot-debug-museum`
3. `alien-linguist-parameter-dialogue`
4. `ecosystem-balance-loop-simulation`
5. `time-travel-recipe-sequencing`
6. `mars-rover-proximity-mission`
7. `creature-choreography-loop-lab`
8. `neighborhood-data-story`
9. `accessibility-rescue-camera-captions`
10. `mythic-choice-event-tree`

## Source map

Grounded in official Alice resource themes:

- Alice resources overview: <https://www.alice.org/resources/>
- Alice 3 resource categories: <https://www.alice.org/resources/alice-3/>
- Alice 3 lessons: <https://www.alice.org/resources/alice-3-lessons>
- Building A Scene: <https://www.alice.org/resources/lessons/building-a-scene/>
- Programming in Alice: <https://www.alice.org/resources/lessons/programming-in-alice/>
- Design Process Virtual Reality: <https://www.alice.org/resources/lessons/design-process-virtual-reality/>
- Moving The Camera how-to: <https://www.alice.org/resources/how-tos/moving-the-camera/>
- Using Camera Markers how-to: <https://www.alice.org/resources/how-tos/using-camera-markers/>
- Using Camera Views how-to: <https://www.alice.org/resources/how-tos/using-camera-views/>
- Alice 3 setup/download: <https://www.alice.org/get-alice/alice-3/>
