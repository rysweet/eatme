# Student missions

Student missions describe the learner journey expected by eatme scenarios. They
focus on visible Alice behavior, prediction, observation, iteration, and
reflection.

## Student mission goals

Student missions help learners:

- understand the creative goal
- predict what an Alice action or code change will do
- run the world and observe visible behavior
- compare expected and actual results
- revise one meaningful choice
- explain the final behavior in their own words
- share evidence with an instructor or peer

## Student persona crew

The canonical student persona crew lives in:

```text
assets/personas/alice-user-crew.yaml
```

Student personas include:

| Persona | Learner focus |
| --- | --- |
| `curious-novice` | Safe cause-and-effect experiments |
| `creative-storyteller` | Narrative, camera, audio, and character choices |
| `playful-tinkerer` | Surprising variations and bugs as clues |
| `systems-puzzle-solver` | Rule-based games and simulations |
| `reflective-debugger` | Expected-versus-actual repair tests |
| `collaborative-peer-mentor` | Help through questions and evidence |
| `accessibility-advocate` | Communication across audience needs |
| `vr-player-tester` | Comfort, orientation, and fallback access |
| `media-audio-creator` | Audio, camera, timing, captions, and media cues |
| `model-texture-importer` | Responsible import and fallback behavior |
| `data-detective` | Variables, data types, arrays, and visible world state |
| `immersive-camera-director` | Camera and VR perspective with audience clarity and fallbacks |
| `game-narrative-designer` | Small playable stories/games with choices, state, and playtest evidence |

## Student scenario coverage

Student-facing scenarios cover Alice modernization from visible learner outcomes,
not internal Alice implementation details.

Current audit inventory:

- 46 canonical scenario assets under `assets/scenarios/eatme/`
- 47 Gadugi scenario assets under `assets/scenarios/gadugi/` (46 generated
  adapters and 1 hand-authored validation regression)
- 24 personas in `assets/personas/alice-user-crew.yaml`
- 45 canonical scenarios name at least one student persona; `real-alice-launch-smoke`
  is the baseline launch evidence scenario and names no student persona

Committed student-facing scenario assets currently include:

| Scenario id | Primary student personas | Student outcome |
| --- | --- | --- |
| `hour-of-code-studio-kickoff` | `curious-novice`, `creative-storyteller`, `reflective-debugger` | Build a tiny first scene or fallback studio role, record first animation evidence, and connect one visible change to a student choice. |
| `starter-project-open-save-export-preflight` | `creative-storyteller`, `accessibility-advocate` | Open the bundled starter project and collect bounded preflight evidence before any save, reopen, or export journey is trusted. |
| `first-lessons-real-ui-actions` | `curious-novice`, `creative-storyteller` | Record the first object, procedure, run, and save action contract plus learner packet expectations without claiming full UI automation. |
| `building-a-scene-first-world` | `curious-novice`, `creative-storyteller` | Build a small world, predict audience focus, run Alice, revise one visible scene choice. |
| `code-editor-first-run` | `curious-novice`, `reflective-debugger` | Edit code, predict the world behavior, run Alice, and explain the expected-versus-actual result. |
| `reusable-methods-and-parameters` | `systems-puzzle-solver`, `collaborative-peer-mentor` | Use reusable behavior with a parameter and describe why reuse changed the project. |
| `functions-as-questions-about-the-world` | `systems-puzzle-solver`, `reflective-debugger` | Treat functions as world-state questions and test answers against visible behavior. |
| `loops-and-conditionals-mini-challenge` | `playful-tinkerer`, `reflective-debugger` | Use repetition and choice logic, then debug one surprising visible result. |
| `events-collision-proximity-game` | `systems-puzzle-solver`, `game-narrative-designer` | Create trigger-driven feedback and test collision or proximity behavior. |
| `game-score-timer-win-lose-loop` | `systems-puzzle-solver`, `data-detective`, `game-narrative-designer` | Connect score, timer, win/lose state, and reflection to observable game behavior. |
| `variables-scorekeeper-timekeeper` | `data-detective`, `reflective-debugger` | Show how variables and data types change visible world state. |
| `arrays-collection-choreography` | `data-detective`, `playful-tinkerer` | Use arrays or lists to control item order, index behavior, and boundary tests. |
| `mythic-choice-event-tree` | `creative-storyteller`, `game-narrative-designer`, `collaborative-peer-mentor` | Build a playable story branch and revise it from peer playtest evidence. |
| `vr-camera-locomotion-journey` | `vr-player-tester`, `immersive-camera-director`, `accessibility-advocate` | Record VR availability, camera markers, comfort notes, and desktop fallback evidence. |
| `vr-camera-perspective-tour` | `immersive-camera-director`, `accessibility-advocate` | Design audience viewpoint and non-VR fallback communication. |
| `modified-class-portability` | `model-texture-importer`, `reflective-debugger` | Prove a shared modified class has before-export, destination-import, and after-import behavior evidence. |
| `student-reflection-artifact-review` | `reflective-debugger`, `collaborative-peer-mentor` | Pair a student learning artifact with reflection that names one Alice action, visible behavior, run result, and next revision. |

Committed outside-in Alice QA expansion scenarios add these student missions as
canonical scenario assets with generated Gadugi adapters:

| Scenario id | Primary student personas | Student outcome |
| --- | --- | --- |
| `setup-support-lab-readiness` | `collaborative-peer-mentor`, `curious-novice` | Confirm launch readiness or fallback participation without treating environment blockers as learner mistakes. |
| `alice-2-migration-bridge` | `curious-novice`, `creative-storyteller` | Produce current Alice 3 evidence while preserving the learning intent of an older Alice 2 activity. |
| `vr-player-comfort-playtest` | `vr-player-tester`, `accessibility-advocate` | Playtest orientation, locomotion comfort, discoverability, and fallback access without assuming headset availability. |
| `model-texture-import-checkpoint` | `model-texture-importer`, `reflective-debugger`, `creative-storyteller` | Check imported model or texture source, license, scale, orientation, visible texture behavior, and fallback asset choice. |
| `media-audio-cue-storyboard` | `media-audio-creator`, `creative-storyteller`, `accessibility-advocate` | Storyboard one sound, timing, camera, or caption cue with prediction, run evidence, accessibility fallback, and revision reflection. |
| `student-artifact-package-share-evidence` | `reflective-debugger`, `collaborative-peer-mentor` | Package one Alice artifact or screenshot with student change, visible run result, attribution or classroom context, and a next revision for instructor or peer review. |
| `classroom-gallery-walk-and-rubric` | `collaborative-peer-mentor`, `reflective-debugger`, `accessibility-advocate` | Use peer observation, concept language, respectful questions, creator response, and one revision checkpoint during a gallery walk. |
| `teacher-community-sharing-loop` | `collaborative-peer-mentor`, `accessibility-advocate` | Carry student evidence and accessibility notes into a teacher-facing activity handoff without ranking classmates or teachers. |
| `lost-robot-debug-museum` | `reflective-debugger`, `collaborative-peer-mentor` | Plan a debugging investigation: record expected-vs-actual tour behavior, write a hypothesis before editing, make one minimal change, rerun, and pose a peer question before concluding. |

Instructor-led scenario assets also produce student-facing prompts, handouts, or
handoffs. They are instructor missions first, but their student persona mappings
matter when checking learner-facing coverage:

| Scenario id | Student-facing piece |
| --- | --- |
| `instructor-exercise-builder` | Student exercise brief with one concept focus, scaffolded entry, choice, stretch path, and visible evidence. |
| `instructor-lesson-materials-remix` | Student handout and exit ticket derived from Alice.org resources. |
| `instructor-alice-concept-map` | Student-facing concept language and misconception checks tied to visible Alice actions. |
| `instructor-student-outcomes-rubric` | Rubric language for concept evidence, creativity, process, reflection, and accessibility. |
| `instructor-classroom-setup-readiness` | Student readiness note and fallback plan that avoids treating environment blockers as learner mistakes. |
| `instructor-student-launch-evidence-handoff` | Student action prompt asking for one Alice action, visible run result, and one next revision. |
| `workshop-facilitator-live-studio` | Student prompt cards, help signals, peer feedback, revision, reflection, and share-out evidence. |
| `curriculum-sequence-remix-pack` | Student evidence plan linked to prerequisites, pacing, fallback notes, and swap points. |

## Mission rhythm

A student mission follows this rhythm:

1. **Prompt** - Read the creative or technical mission.
2. **Predict** - State what should happen before running the world.
3. **Build** - Make a small Alice scene, code, camera, audio, or behavior change.
4. **Run** - Observe the world.
5. **Compare** - Name expected versus actual behavior.
6. **Revise** - Change one meaningful thing.
7. **Reflect** - Explain what changed and why it matters.
8. **Share** - Provide the artifact, screenshot, description, or reflection
   requested by the mission.

For `student-artifact-package-share-evidence`, "share" means a review packet for
an instructor or peer. The expected output is not a public URL, hosted gallery
entry, deployment log, or proof that a sharing platform works. See
[Sharing Readiness Boundary](sharing-readiness-boundary.md).

## Evidence expectations

Good student evidence includes:

- a named Alice world or scenario
- one visible behavior that can be observed by someone else
- a prediction made before running
- an observation made after running
- one revision based on evidence
- a short explanation of cause and effect
- any required screenshot, exported file, or reflection text

For VR or hardware-dependent missions, evidence should state whether real
hardware was available. If it was not, the student should use the documented
desktop fallback instead of pretending the VR path was tested.

For real Alice lesson scenarios, a student or reviewer may attach the gated launch
manifest as setup evidence:

```bash
export NODE_OPTIONS=--max-old-space-size=32768
export SCENARIO_ID=building-a-scene-first-world
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

The manifest proves Alice reached a smoke-ready desktop session for that
scenario. It does not replace the learner evidence: prediction, visible world
behavior, revision, and reflection still have to be supplied by the mission.

The `first-lessons-real-ui-actions` scenario is the student Alice lesson
scenario evidence contract for the first real Alice actions. It records a
scenario-labeled real Alice launch path, manifest/log/window/screenshot evidence,
Alice window
detection, and `ui-action-contract.json` expectations for place object, edit
procedure, run world, and save project. This is launch/action-contract evidence
only. It is not full UI automation, not creative assessment, and not
learner-world grading.

### First lesson evidence packet

A complete student packet for the first real Alice lesson scenario includes both
machine evidence and learner evidence:

| Evidence | Source | Required meaning |
| --- | --- | --- |
| Launch manifest | `runs/first-lessons-real-ui-actions/<run-id>/manifest.json` | Alice launched for the scenario id and reported deterministic assertions. |
| Alice log/window/screenshot artifacts | Run artifact directory | The desktop session produced observable startup evidence. |
| Action contract | `ui-action-contract.json` | The first object/code/run/save actions are declared for deterministic automation. |
| Learner prediction | Student response | The learner stated expected visible behavior before running. |
| Learner observation and revision | Student response or artifact | The learner compared actual behavior and changed one meaningful thing. |
| Reflection | Student response | The learner explained cause and effect in their own words. |

The first three rows support setup and harness claims. The last three rows are
the mission evidence. Do not accept a launch manifest alone as proof that the
student completed or understood the lesson.

## Example student mission

```text
Mission: Building a Scene First World

Create a small Alice scene with at least two objects. Before running, predict
what the audience will notice first. Run the world, observe the result, revise
one placement, camera, or timing choice, then explain how the revision changed
the audience experience.
```

Expected response shape:

```text
Prediction:
I expected the viewer to notice the penguin first because it starts closest to
the camera.

Observation:
The tree blocked part of the penguin, so the viewer noticed the tree instead.

Revision:
I moved the penguin forward and rotated the camera slightly.

Reflection:
The scene now communicates the intended character focus because the first visible
movement and camera angle point toward the penguin.
```

## What missions avoid

Student missions should not require:

- exact UI coordinates
- hidden implementation details
- memorized click paths without concept evidence
- visual polish without behavior or explanation
- a single correct creative answer

The mission succeeds when the learner can show evidence of thinking and visible
Alice behavior, not when every student produces the same world.
