# Instructor missions

Instructor missions describe how an educator or instructor-facing agent prepares,
facilitates, and assesses Alice learning activities. They are written as
scenario assets so mission intent can change without rewriting Rust.

## Instructor mission goals

Instructor missions help educators:

- choose an Alice lesson lane
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

The outside-in Alice QA expansion commits these instructor/student lanes as
canonical eatme scenarios with generated Gadugi adapters:

| Scenario id | Outcome |
| --- | --- |
| `setup-support-lab-readiness` | Lab install, graphics, storage, access, and fallback readiness evidence |
| `alice-2-migration-bridge` | Alice 2 lesson intent mapped to Alice 3 workflows and visible student evidence |
| `vr-player-comfort-playtest` | Instructor-ready VR/player comfort playtest with non-VR classroom fallback |
| `model-texture-import-checkpoint` | Import checkpoint for source, license, scale, orientation, texture visibility, and fallback assets |

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

Committed outside-in Alice expansion lanes add these instructor decisions:

| Scenario id | Instructor persona | Classroom outcome |
| --- | --- | --- |
| `setup-support-lab-readiness` | `setup-support-specialist` | Lab readiness runbook separating install, Java, graphics, storage, account, and fallback responsibilities. |
| `alice-2-migration-bridge` | `alice-2-migration-mentor` | Migration bridge that preserves Alice 2 lesson intent while requiring Alice 3 evidence. |
| `vr-player-comfort-playtest` | `workshop-facilitator`, `classroom-orchestrator` | Short studio playtest with comfort checkpoints, helpers, share-out, and desktop fallback. |
| `model-texture-import-checkpoint` | `studio-facilitator`, `assessment-curator` | Import review that accepts responsible fallbacks instead of requiring one third-party model. |

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

4. If the mission depends on a real Alice lane, run the relevant launch smoke:

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
