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

## Good instructor outputs

A good instructor mission output is concrete enough to run in a classroom and
flexible enough for student choice. It should name the concept, the Alice
activity, the expected evidence, common misconceptions, and a fallback path when
setup or hardware is unavailable.

Avoid outputs that depend on exact UI coordinates, hidden implementation
details, or visual polish alone.

