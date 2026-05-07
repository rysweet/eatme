# Instructor missions

Instructor missions describe how an educator or instructor-facing agent prepares,
facilitates, and assesses Alice learning activities. They are written as
scenario assets so mission intent can change without rewriting Rust.

## Instructor mission goals

Instructor missions help educators:

- choose an Alice lesson scenario
- map Alice actions to transferable programming concepts
- prepare setup and fallback plans
- create student-facing instructions
- define evidence of learning
- assess concept, creativity, process, and reflection
- package shareable classroom artifacts

## Instructor persona crew

The canonical persona crew lives in:

```text
assets/personas/alice-user-crew.yaml
```

Instructor personas include:

| Persona | Mission focus |
| --- | --- |
| `concept-cartographer` | Sequence Alice experiences into teachable CS concepts |
| `exercise-forger` | Create meaningful exercises with scaffolds and choice |
| `studio-facilitator` | Run build, run, critique, revise cycles |
| `debug-coach` | Turn debugging into prediction and observation |
| `assessment-curator` | Assess evidence, creativity, process, and reflection |
| `classroom-orchestrator` | Manage setup, sharing, and fallback plans |
| `curriculum-pathway-designer` | Sequence Alice resources into units |
| `setup-support-specialist` | Coordinate installation and readiness |
| `workshop-facilitator` | Run short studio blocks |
| `alice-2-migration-mentor` | Bridge Alice 2 lesson intent into Alice 3 workflows |
| `teacher-community-curator` | Package remixable teacher-community assets |

## Instructor flow assets

Current instructor mission assets include:

| Scenario id | Outcome |
| --- | --- |
| `instructor-exercise-builder` | Exercise with concept focus, scaffolds, choice, and visible evidence |
| `instructor-lesson-materials-remix` | Teacher plan, student handout, and exit ticket from Alice resources |
| `instructor-alice-concept-map` | Alice action to CS vocabulary map with misconception checks |
| `instructor-student-outcomes-rubric` | Rubric for concept, creativity, process, and reflection |
| `instructor-classroom-setup-readiness` | Setup checklist, student note, and fallback plan |
| `student-reflection-artifact-review` | Review note and revision prompt for a student learning artifact plus reflection |
| `classroom-gallery-walk-and-rubric` | Gallery-walk rubric, peer feedback card, creator response prompt, and revision checkpoint |
| `media-audio-cue-storyboard` | Media cue storyboard, student prediction prompt, accessibility fallback note, and revision reflection prompt |
| `student-artifact-package-share-evidence` | Artifact share packet checklist, student evidence handoff prompt, and instructor review boundary note |
| `instructor-student-launch-evidence-handoff` | Real Alice evidence handoff card, instructor readiness note, and student action prompt |
| `workshop-facilitator-live-studio` | Live-studio facilitation plan, timing plan, student prompt cards, help signals, and share-out artifacts |
| `teacher-community-sharing-loop` | Teacher-community share card, classroom handoff note, and remix feedback prompt |
| `curriculum-sequence-remix-pack` | Curriculum sequence map, lesson sequence remix pack, and student evidence plan |

`instructor-lesson-materials-remix` is the instructor lesson-material/remix
evidence contract. It verifies that an Alice lesson packet is represented by
scenario-labeled assets, instructor-facing prompts, acceptance probes, and
teacher-plan/student-handout/exit-ticket outputs. The instructor flow does not
grade learner worlds or assess creativity automatically; those remain instructor
judgment and classroom review tasks.

### Lesson-material/remix packet

The instructor remix packet is complete when it contains these reviewable
artifacts:

| Artifact | Required contents |
| --- | --- |
| Teacher plan | Alice resource grounding, concept focus, timing, setup/fallback notes, facilitation moves, and evidence checkpoints. |
| Student handout | Plain-language mission, prediction prompt, build/run/revise steps, reflection prompt, and submission shape. |
| Exit ticket | Short checks for concept vocabulary, observed behavior, revision evidence, and remaining questions. |
| Remix notes | What changed from the source resource, why the change is classroom-safe, and what must still be judged by the instructor. |

The packet may point to a real Alice launch manifest as setup evidence. It must
not present that manifest as proof of learner understanding, creative quality, or
world correctness.

The outside-in Alice QA expansion commits these instructor/student scenarios as
canonical eatme scenarios with generated Gadugi adapters:

| Scenario id | Outcome |
| --- | --- |
| `setup-support-lab-readiness` | Lab install, graphics, storage, access, and fallback readiness evidence |
| `alice-2-migration-bridge` | Alice 2 lesson intent mapped to Alice 3 workflows and visible student evidence |
| `vr-player-comfort-playtest` | Instructor-ready VR/player comfort playtest with non-VR classroom fallback |
| `model-texture-import-checkpoint` | Import checkpoint for source, license, scale, orientation, texture visibility, and fallback assets |
| `classroom-gallery-walk-and-rubric` | Gallery-walk evidence packet for peer critique, student response, and one revision checkpoint |

## Instructor scenario coverage

Instructor-facing scenarios pressure Alice modernization through classroom
decisions a teacher or support lead can verify. Committed instructor-facing
scenario assets currently cover:

| Scenario id | Instructor persona | Classroom outcome |
| --- | --- | --- |
| `instructor-exercise-builder` | `exercise-forger` | Exercise plan with one concept focus, scaffolded entry, learner choice, stretch path, and visible evidence. |
| `instructor-lesson-materials-remix` | `curriculum-pathway-designer`, `teacher-community-curator` | Teacher plan, student handout, and exit ticket derived from Alice.org resources. |
| `instructor-alice-concept-map` | `concept-cartographer` | Alice actions mapped to transferable CS vocabulary and misconception checks. |
| `instructor-student-outcomes-rubric` | `assessment-curator` | Rubric that scores concept evidence, creativity, process, reflection, and accessibility. |
| `instructor-classroom-setup-readiness` | `classroom-orchestrator` | Classroom setup checklist, student-facing readiness note, and fallback plan. |
| `student-reflection-artifact-review` | `assessment-curator`, `studio-facilitator` | Instructor review of artifact behavior and learner explanation with one student-owned revision. |
| `classroom-gallery-walk-and-rubric` | `assessment-curator`, `studio-facilitator`, `classroom-orchestrator` | Gallery-walk rubric and feedback prompts for visible artifact behavior, concept language, peer questions, creator response, and revision evidence. |
| `media-audio-cue-storyboard` | `exercise-forger`, `studio-facilitator`, `assessment-curator` | Media cue storyboard that links sound, timing, camera, captions, prediction, run evidence, fallback notes, and revision reflection. |
| `student-artifact-package-share-evidence` | `teacher-community-curator`, `assessment-curator`, `classroom-orchestrator` | Student artifact share packet that keeps artifact references, student explanation, classroom context, next revision, and human review boundaries visible. |
| `instructor-student-launch-evidence-handoff` | `classroom-orchestrator`, `debug-coach` | Classroom handoff that separates real Alice launch evidence from learner behavior and asks students for one visible action, run result, and next revision. |
| `workshop-facilitator-live-studio` | `workshop-facilitator`, `studio-facilitator` | Live-studio workshop plan with setup, timing, observation, help signals, peer feedback, revision, reflection, and share-out evidence. |
| `teacher-community-sharing-loop` | `teacher-community-curator`, `classroom-orchestrator`, `assessment-curator` | Teacher-facing share card and handoff note with attribution, classroom constraints, student evidence, accessibility notes, and remix feedback prompts. |
| `curriculum-sequence-remix-pack` | `curriculum-pathway-designer`, `assessment-curator` | Curriculum sequence map that links committed Alice scenario assets to prerequisites, pacing, swap points, fallback notes, and visible student evidence. |

Committed outside-in Alice expansion scenarios add these instructor decisions:

| Scenario id | Instructor persona | Classroom outcome |
| --- | --- | --- |
| `setup-support-lab-readiness` | `setup-support-specialist` | Lab readiness runbook separating install, Java, graphics, storage, account, and fallback responsibilities. |
| `alice-2-migration-bridge` | `alice-2-migration-mentor` | Migration bridge that preserves Alice 2 lesson intent while requiring Alice 3 evidence. |
| `vr-player-comfort-playtest` | `workshop-facilitator`, `classroom-orchestrator` | Short studio playtest with comfort checkpoints, helpers, share-out, and desktop fallback. |
| `model-texture-import-checkpoint` | `studio-facilitator`, `assessment-curator` | Import review that accepts responsible fallbacks instead of requiring one third-party model. |
| `classroom-gallery-walk-and-rubric` | `assessment-curator`, `studio-facilitator`, `classroom-orchestrator` | Gallery-walk review that turns peer observation into student-owned revision without ranking visual spectacle. |
| `teacher-community-sharing-loop` | `teacher-community-curator`, `classroom-orchestrator`, `assessment-curator` | Teacher-community handoff that shares editable activity context without ranking teachers or claiming a deployed platform. |
| `lost-robot-debug-museum` | `debug-coach`, `exercise-forger` | Debugging mystery brief, student debug journal with hypothesis-before-edit discipline, and peer question checkpoint that turns visible wrong behavior into a learning conversation. |

## Mission design contract

An instructor mission should include:

- a plain-language goal
- relevant Alice resource grounding
- target instructor persona
- expected student personas or learner modes
- setup requirements
- facilitation steps
- acceptance probes
- rubric dimensions
- outputs the instructor can hand to students
- avoid-list for brittle or over-specified responses

## Classroom readiness workflow

1. Select the mission scenario.
2. Validate assets:

   ```bash
   cargo run -q -p eatme-cli -- assets validate --json
   ```

3. Check whether generated adapters are fresh:

   ```bash
   cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
   ```

4. If the mission depends on a real Alice scenario, run the relevant launch smoke:

   ```bash
   export NODE_OPTIONS=--max-old-space-size=32768

   EATME_REAL_ALICE=1 cargo run -q -p eatme-cli -- alice launch-smoke \
     --alice-home "${ALICE_HOME}" \
     --scenario building-a-scene-first-world \
     --run-id instructor-preflight-building-a-scene \
     --json \
     --no-memory \
     --offline-package
   ```

5. Use the mission's acceptance probes and rubric to evaluate the instructor
   output.

For expanded outside-in coverage, choose the launch-smoke scenario that matches
the instructor decision being prepared:

| Instructor decision | Scenario id |
| --- | --- |
| Can this lab start Alice with the required desktop dependencies and fallbacks? | `setup-support-lab-readiness` |
| Can this Alice 2 lesson be taught as an Alice 3 workflow without losing learning intent? | `alice-2-migration-bridge` |
| Can students test VR comfort and still complete the lesson without a headset? | `vr-player-comfort-playtest` |
| Can imported models/textures be reviewed responsibly with a fallback asset? | `model-texture-import-checkpoint` |

Real Alice evidence stays explicit and gated. CI can validate assets and Gadugi
generation without launching Alice; local or lab preflight runs opt in with
`EATME_REAL_ALICE=1`.

## Good instructor outputs

A good instructor mission output is concrete enough to run in a classroom and
flexible enough for student choice. It should name the concept, the Alice
activity, the expected evidence, common misconceptions, and a fallback path when
setup or hardware is unavailable.

Avoid outputs that depend on exact UI coordinates, hidden implementation
details, or visual polish alone.
