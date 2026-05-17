# eatme documentation

`eatme` is the documentation, asset, and launch-smoke harness for agentic Alice
quality assurance. It describes classroom missions as editable assets, validates
those assets before they are trusted, generates Gadugi adapter scenarios from the
canonical eatme scenarios, and records deterministic evidence when real Alice is
launched.

Eatme has three layers:

| Layer | Purpose |
| --- | --- |
| Canonical mission assets | Persona crews and eatme scenario YAML owned by this repository |
| Harness commands | Rust CLI commands for validation, Gadugi generation, Alice discovery, packaging, and launch smoke |
| Published docs | This MkDocs site, built locally and deployed through GitHub Pages |

## Audience routes

| If you are... | Start here |
| --- | --- |
| Installing eatme | [Installation](installation.md) |
| Running commands | [CLI Usage](cli-usage.md) |
| Writing scenarios | [Scenario Authoring](scenario-authoring.md) |
| Using Gadugi | [Gadugi Adapters](gadugi-adapters.md) |
| Keeping generated assets in sync | [Generated Asset Consistency](generated-asset-consistency.md) |
| Checking a change | [Validation and Quality Gates](validation-quality-gates.md) |
| Maintaining outside-in Alice Rust tests | [Outside-in Alice Test Modules](outside-in-alice-test-modules.md) |
| Running real Alice | [Alice Integration](alice-integration.md) |
| Running the deterministic real-Alice integration test | [Deterministic Real-Alice Smoke Test](deterministic-real-alice-smoke-test.md) |
| Reviewing post-focus screenshot evidence | [Post-focus Screenshot Evidence](post-focus-screenshot-evidence.md) |
| Checking RabbitHole evidence needed before first-lesson readiness | [Lesson Session Readiness](lesson-session-readiness.md) |
| Checking first-lesson readiness preflight | [First-Lesson Grading Report](first-lesson-grading-report.md) |
| Understanding machine vs. human assessment boundaries | [Creative Assessment Boundary](creative-assessment-boundary.md) |
| Reviewing the first-lesson evidence boundary contract | [First-Lesson Evidence Readiness](first-lesson-evidence-readiness.md) |
| Auditing readiness artifact shape and wording | [Evidence Artifact Contract](evidence-artifact-contract.md) |
| Maintaining first-lesson readiness module boundaries | [Lesson Readiness Module Boundary](lesson-readiness-module-boundary.md) |
| Reviewing starter project preflight evidence | [Starter Project Preflight Evidence](starter-project-preflight-evidence.md) |
| Reviewing save/reopen readiness evidence | [Save/reopen Readiness](save-reopen-readiness.md) |
| Recording exact-head PR readiness | [Default-workflow PR Readiness](default-workflow-pr-readiness.md) |
| Recovering PR #199 merge-readiness evidence | [PR #199 Recovery Workflow](pr-199-recovery-workflow.md) |
| Reviewing PR #160 gap-reporting recovery readiness | [PR #160 Gap-Reporting Readiness](pr-160-gap-reporting-readiness.md) |
| Reviewing live studio workshop evidence | [Live Studio Workshop Evidence Contract](live-studio-workshop-evidence.md) |
| Checking sharing and deployment feature readiness | [Sharing Platform Readiness](sharing-platform-readiness.md) |
| Planning class activity | [Instructor Missions](instructor-missions.md) |
| Completing a learner journey | [Student Missions](student-missions.md) |
| Publishing docs | [GitHub Pages](github-pages.md) |
| Reviewing code editor first run E2E test evidence | [Code Editor First Run E2E Test](code-editor-first-run-e2e.md) |
| Auditing real Alice lesson scenario evidence | [Alice Lesson Smoke](alice-lesson-smoke.md) |
| Reviewing events and collision grading | [Events and Collision Grading](events-and-collision-grading.md) |
| Understanding grading module layout and shared helpers | [Grading Module Architecture](grading-module-architecture.md) |
| Running the real-Alice events-and-collision integration test | [Events and Collision Grading — Real-Alice Integration Test](events-and-collision-grading.md#real-alice-integration-test) |
| Running real-Alice lesson grading tests (L5–L8) | [Real-Alice Lesson Grading Tests](real-alice-lesson-grading-tests.md) |

## What eatme proves

Eatme proves that Alice-facing assets and adapter scenarios are coherent before
they are used by people or agents. For real Alice smoke scenarios, it also proves
that the desktop application can be packaged, launched, observed, and reported
through deterministic artifacts.

A passing launch smoke records:

- dependency-check results
- Alice package command and exit status
- virtual display readiness
- Java launch command
- Alice process status
- startup screenshot or window evidence
- Alice log artifact
- fatal log scan
- manifest assertions
- failure category, or `null` when the smoke passed

## What eatme does not pretend to prove

The real Alice lesson scenarios are launch-smoke scenarios. They prove smoke-ready
evidence for a scenario-labeled Alice run. They do not drive an entire lesson
through the Alice interface, score learner creativity, inspect private Alice
implementation details, or grade saved learner worlds.

Instructor and student mission docs describe the intended classroom and agentic
contract. Runtime validation stays explicit about which parts are deterministic
and which parts are evidence expectations for human or agent review.

## Outside-in evidence for Alice lesson scenarios

This evidence connects instructor and student Alice lesson scenarios to a real
Alice launch path without overstating what the launch smoke proves.

| Scenario | Audience | Evidence contract |
| --- | --- | --- |
| `real-alice-launch-smoke` | Harness and CI/manual preflight | Baseline Alice launch, manifest, log, window, screenshot, and deterministic assertion evidence. |
| `first-lessons-real-ui-actions` | Instructors, students, and reviewers | First-lesson readiness evidence for original Alice and RabbitHole; the report summarizes shown evidence, optional desktop next-action evidence, not-yet-shown states, and explicit unproven claims. |
| `instructor-lesson-materials-remix` | Instructors and instructor agents | Teacher plan, student handout, exit ticket, acceptance probes, and review/remix language derived from Alice resources. |

Use the manifest from a real Alice run as setup evidence, then use the mission
artifact requirements to review learner or instructor work. A launch manifest is
not a creative assessment and does not grade a learner world.

## Main workflows

Validate assets:

```bash
cargo run -q -p eatme-cli -- assets validate --json
```

Check generated Gadugi adapters:

```bash
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
```

Understand how generated adapter counts stay aligned with the asset inventory:
[Generated Asset Consistency](generated-asset-consistency.md).

Audit instructor/student lesson-session readiness:
[Lesson Session Readiness](lesson-session-readiness.md).

Review the conservative first-lesson evidence boundary contract:
[First-Lesson Evidence Readiness](first-lesson-evidence-readiness.md).

Review the save/reopen artifact and reopened-state evidence boundary:
[Save/reopen Readiness](save-reopen-readiness.md).

Audit the readiness artifact shape and wording contract:
[Evidence Artifact Contract](evidence-artifact-contract.md).

Keep first-lesson readiness helper logic in focused Rust submodules:
[Lesson Readiness Module Boundary](lesson-readiness-module-boundary.md).

Record exact-head pull request readiness:
[Default-workflow PR Readiness](default-workflow-pr-readiness.md).

Recover PR #199 merge-readiness evidence after the manual-fallback violation:
[PR #199 Recovery Workflow](pr-199-recovery-workflow.md).

Review PR #160 gap-reporting recovery readiness:
[PR #160 Gap-Reporting Readiness](pr-160-gap-reporting-readiness.md).

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
