# eatme documentation

`eatme` is the documentation, scenario, and launch-smoke harness for Alice
quality assurance. It describes classroom missions as editable scenario files,
checks those files before they are trusted, keeps generated runner files aligned,
and records repeatable evidence when real Alice is launched.

Eatme has three layers:

| Layer | Purpose |
| --- | --- |
| Mission files | Persona crews and scenario YAML owned by this repository |
| Harness commands | CLI commands for validation, Alice discovery, packaging, generated runner files, and launch smoke |
| Published docs | This MkDocs site, built locally and deployed through GitHub Pages |

## Audience routes

| If you are... | Start here |
| --- | --- |
| Installing eatme | [Installation](installation.md) |
| Running commands | [CLI Usage](cli-usage.md) |
| Writing scenarios | [Scenario Authoring](scenario-authoring.md) |
| Running generated scenarios | [Gadugi Adapters](gadugi-adapters.md) |
| Keeping generated files in sync | [Generated Asset Consistency](generated-asset-consistency.md) |
| Checking a change | [Validation and Quality Gates](validation-quality-gates.md) |
| Maintaining outside-in Alice Rust tests | [Outside-in Alice Test Modules](outside-in-alice-test-modules.md) |
| Running real Alice | [Alice Integration](alice-integration.md) |
| Auditing real Alice lesson scenario evidence | [Alice Lesson Smoke](alice-lesson-smoke.md) |
| Following the first-lesson readiness path | [Lesson Session Readiness](lesson-session-readiness.md) |
| Reading exactly what first-lesson evidence proves | [First-Lesson Evidence Readiness](first-lesson-evidence-readiness.md) |
| Reviewing starter project preflight evidence | [Starter Project Preflight Evidence](starter-project-preflight-evidence.md) |
| Recording exact-head PR readiness | [Default-workflow PR Readiness](default-workflow-pr-readiness.md) |
| Reviewing live studio workshop evidence | [Live Studio Workshop Evidence](live-studio-workshop-evidence.md) |
| Planning class activity | [Instructor Missions](instructor-missions.md) |
| Completing a learner journey | [Student Missions](student-missions.md) |
| Publishing docs | [GitHub Pages](github-pages.md) |

## Silver-thread lesson path

Use this path when you need the shortest end-to-end route from a scenario to
trustworthy first-lesson evidence:

1. Confirm how real Alice is launched in [Alice Integration](alice-integration.md).
2. Review the scenario roster and launch evidence in
   [Alice Lesson Smoke](alice-lesson-smoke.md).
3. Run or inspect the first-lesson readiness report in
   [Lesson Session Readiness](lesson-session-readiness.md).
4. Interpret the `Shown`, `Not yet shown`, and `Unproven` sections in
   [First-Lesson Evidence Readiness](first-lesson-evidence-readiness.md).
5. Use [Instructor Missions](instructor-missions.md) or
   [Student Missions](student-missions.md) for the classroom handoff.

The concrete classroom handoff scenarios are
[`instructor-student-launch-evidence-handoff`](https://github.com/rysweet/eatme/blob/main/assets/scenarios/eatme/instructor-student-launch-evidence-handoff.yaml),
which turns launch/action evidence into a student-facing handoff, and
[`instructor-student-outcomes-rubric`](https://github.com/rysweet/eatme/blob/main/assets/scenarios/eatme/instructor-student-outcomes-rubric.yaml),
which frames student-visible outcomes without automated grading. The student
readiness scenario is
[`first-lessons-real-ui-actions`](https://github.com/rysweet/eatme/blob/main/assets/scenarios/eatme/first-lessons-real-ui-actions.yaml).

This path proves only the evidence named by each report. It does not claim that
a lesson was completed, that a learner world was graded, or that all Alice UI
actions were automated unless a specific report shows that exact evidence.

## What eatme proves

Eatme proves that Alice-facing scenario files and generated runner files agree
before they are used by people or agents. For real Alice smoke scenarios, it also
proves that the desktop application can be packaged, launched, observed, and
reported through repeatable evidence.

A passing launch smoke records:

- dependency-check results
- Alice package command and exit status
- virtual display readiness
- Java launch command
- Alice process status
- startup screenshot or window evidence
- Alice log artifact
- fatal log scan
- run summary checks
- failure category, or `null` when the smoke passed

## What eatme does not pretend to prove

The real Alice lesson scenarios are launch-smoke scenarios. They prove smoke-ready
evidence for a scenario-labeled Alice run. They do not drive an entire lesson
through the Alice interface, score learner creativity, inspect private Alice
implementation details, or grade saved learner worlds.

Instructor and student mission docs describe the intended classroom path.
Runtime validation stays explicit about which parts are proven by the harness and
which parts still need human or agent review.

## Outside-in evidence for Alice lesson scenarios

This evidence connects instructor and student Alice lesson scenarios to a real
Alice launch path without overstating what the launch smoke proves.

| Scenario | Audience | What the reader can trust |
| --- | --- | --- |
| `real-alice-launch-smoke` | Harness and CI/manual preflight | Baseline Alice launch, run summary, log, window, screenshot, and repeatable assertion evidence. |
| [`first-lessons-real-ui-actions`](https://github.com/rysweet/eatme/blob/main/assets/scenarios/eatme/first-lessons-real-ui-actions.yaml) | Instructors, students, and reviewers | First-lesson readiness evidence for original and modernized Alice; the report summarizes shown evidence, optional next desktop action evidence, not-yet-shown states, and explicit unproven claims. |
| `instructor-lesson-materials-remix` | Instructors and instructor agents | Teacher plan, student handout, exit ticket, acceptance probes, and review/remix language derived from Alice resources. |
| [`instructor-student-launch-evidence-handoff`](https://github.com/rysweet/eatme/blob/main/assets/scenarios/eatme/instructor-student-launch-evidence-handoff.yaml) | Instructors and students | Handoff card, readiness note, and student action prompt that separate launch/action evidence from classroom observation. |
| [`instructor-student-outcomes-rubric`](https://github.com/rysweet/eatme/blob/main/assets/scenarios/eatme/instructor-student-outcomes-rubric.yaml) | Instructors and students | Student-visible outcomes rubric and feedback frame without claiming automated creative assessment or learner-world grading. |

Use the run summary from a real Alice run as setup evidence, then use the
mission artifact requirements to review learner or instructor work. A launch
run summary is not a creative assessment and does not grade a learner world.

## Main workflows

Validate assets:

```bash
cargo run -q -p eatme-cli -- assets validate --json
```

Check generated runner files:

```bash
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
```

Understand how generated file counts stay aligned with the asset inventory:
[Generated Asset Consistency](generated-asset-consistency.md).

Follow the instructor/student first-lesson readiness path:
[Lesson Session Readiness](lesson-session-readiness.md).

Review exactly what first-lesson evidence proves:
[First-Lesson Evidence Readiness](first-lesson-evidence-readiness.md).

Record exact-head pull request readiness:
[Default-workflow PR Readiness](default-workflow-pr-readiness.md).

Build the docs site:

```bash
python3 -m venv .venv
. .venv/bin/activate
python -m pip install -r requirements-docs.txt
mkdocs build --strict
```

Run a real lesson smoke:

```bash
EATME_REAL_ALICE=1 cargo run -q -p eatme-cli -- alice launch-smoke \
  --alice-home "${ALICE_HOME}" \
  --scenario building-a-scene-first-world \
  --run-id local-building-a-scene-first-world \
  --runs-dir runs \
  --json \
  --no-memory \
  --offline-package
```
