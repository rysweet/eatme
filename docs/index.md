# eatme documentation

`eatme` is the documentation, scenario, and launch-smoke toolkit for Alice
quality checks. It describes classroom missions as editable scenario files,
checks those files before they are trusted, keeps generated runner files aligned,
and records repeatable evidence when real Alice is launched.

Eatme has three layers:

| Layer | Purpose |
| --- | --- |
| Mission files | Persona crews and scenario YAML owned by this repository |
| CLI commands | Commands for validation, Alice discovery, packaging, generated runner files, and launch smoke |
| Published docs | This MkDocs site, built locally and deployed through GitHub Pages |

## Audience routes

| If you are... | Start here |
| --- | --- |
| Installing eatme | [Installation](installation.md) |
| Running commands | [CLI Usage](cli-usage.md) |
| Writing scenarios | [Scenario Authoring](scenario-authoring.md) |
| Running generated scenarios | [Gadugi Adapters](gadugi-adapters.md) |
| Following scenario links through generated runners | [Scenario-link Generated Runners](scenario-link-generated-runners.md) |
| Keeping generated files in sync | [Generated Asset Consistency](generated-asset-consistency.md) |
| Checking a change | [Validation and Quality Gates](validation-quality-gates.md) |
| Maintaining Alice Rust tests | [Alice Test Modules](outside-in-alice-test-modules.md) |
| Running real Alice | [Alice Integration](alice-integration.md) |
| Choosing real Alice lesson scenarios | [Alice Lesson Smoke](alice-lesson-smoke.md) |
| Following the first-lesson readiness path | [Lesson Session Readiness](lesson-session-readiness.md) |
| Reviewing the first-lesson evidence boundary contract | [First-Lesson Evidence Readiness](first-lesson-evidence-readiness.md) |
| Auditing readiness artifact shape and wording | [Evidence Artifact Contract](evidence-artifact-contract.md) |
| Reviewing starter project preflight evidence | [Starter Project Preflight Evidence](starter-project-preflight-evidence.md) |
| Checking pull request readiness | [Pull Request Readiness](default-workflow-pr-readiness.md) |
| Recovering PR #199 merge-readiness evidence | [PR #199 Recovery Workflow](pr-199-recovery-workflow.md) |
| Reviewing live studio workshop evidence | [Live Studio Workshop Evidence](live-studio-workshop-evidence.md) |
| Planning class activity | [Instructor Missions](instructor-missions.md) |
| Completing a learner journey | [Student Missions](student-missions.md) |
| Publishing docs | [GitHub Pages](github-pages.md) |

## First-lesson readiness path

Use this path when a reader needs the shortest route from the docs home page to
trusted first-lesson evidence:

1. Start with the learner and instructor route in
   [Lesson Session Readiness](lesson-session-readiness.md).
2. Review editable scenario expectations in
   [Scenario Authoring](scenario-authoring.md).
3. Follow how canonical scenario links become generated checks in
   [Scenario-link Generated Runners](scenario-link-generated-runners.md).
4. Choose the scenario and launch evidence in
   [Alice Lesson Smoke](alice-lesson-smoke.md).
5. Interpret the validation evidence in
   [First-Lesson Evidence Readiness](first-lesson-evidence-readiness.md).
6. Use [Instructor Missions](instructor-missions.md) or
   [Student Missions](student-missions.md) for the classroom handoff.

The concrete classroom handoff scenarios are
[`instructor-student-launch-evidence-handoff`](https://github.com/rysweet/eatme/blob/main/assets/scenarios/eatme/instructor-student-launch-evidence-handoff.yaml),
which turns launch/action evidence into a student-facing handoff, and
[`instructor-student-outcomes-rubric`](https://github.com/rysweet/eatme/blob/main/assets/scenarios/eatme/instructor-student-outcomes-rubric.yaml),
which frames student-visible outcomes without automated grading. The student
readiness scenario is
[`first-lessons-real-ui-actions`](https://github.com/rysweet/eatme/blob/main/assets/scenarios/eatme/first-lessons-real-ui-actions.yaml).

This path confirms only the evidence named by each report. It does not claim
that a lesson was completed, that a learner world was graded, or that all Alice
UI actions were automated unless a specific report shows that exact evidence.

## What eatme verifies

When `assets validate` and `assets generate-gadugi --check` pass, eatme verifies
that Alice-facing scenario files and generated runner files agree before they are
used by reviewers or runners. For real Alice smoke scenarios, a passing launch
smoke records that the desktop application was packaged, launched, observed, and
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

## What eatme does not pretend to verify

The real Alice lesson scenarios are launch-smoke scenarios. They record
smoke-ready evidence for a scenario-labeled Alice run. They do not drive an
entire lesson through the Alice interface, score learner creativity, inspect
private Alice details, or grade saved learner worlds.

Instructor and student mission docs describe the intended classroom path.
Runtime validation stays explicit about which parts are checked by the CLI and
which parts still need human review.

## Evidence for Alice lesson scenarios

This evidence connects instructor and student Alice lesson scenarios to a real
Alice launch path without overstating what the launch smoke verifies.

| Scenario | Audience | What the reader can trust |
| --- | --- | --- |
| `real-alice-launch-smoke` | Harness and CI/manual preflight | Baseline Alice launch, run summary, log, window, screenshot, and repeatable assertion evidence. |
| [`first-lessons-real-ui-actions`](https://github.com/rysweet/eatme/blob/main/assets/scenarios/eatme/first-lessons-real-ui-actions.yaml) | Instructors, students, and reviewers | First-lesson readiness evidence for original and modernized Alice; the report summarizes shown evidence, optional next desktop action evidence, not-yet-shown states, and explicit unproven claims. |
| `instructor-lesson-materials-remix` | Instructors and reviewers | Teacher plan, student handout, exit ticket, acceptance probes, and review/remix language derived from Alice resources. |
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

Follow scenario links through generated runners:
[Scenario-link Generated Runners](scenario-link-generated-runners.md).

Understand how generated file counts stay aligned with the asset inventory:
[Generated Asset Consistency](generated-asset-consistency.md).

Follow the instructor/student first-lesson readiness path:
[Lesson Session Readiness](lesson-session-readiness.md).

Review exactly what first-lesson evidence shows:
[First-Lesson Evidence Readiness](first-lesson-evidence-readiness.md).

Audit the readiness artifact shape and wording contract:
[Evidence Artifact Contract](evidence-artifact-contract.md).

Check pull request readiness:
[Pull Request Readiness](default-workflow-pr-readiness.md).

Recover PR #199 merge-readiness evidence after the manual-fallback violation:
[PR #199 Recovery Workflow](pr-199-recovery-workflow.md).

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
