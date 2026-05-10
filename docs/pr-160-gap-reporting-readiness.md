# PR #160 gap-reporting readiness

PR #160 gap-reporting readiness is the PR-specific implementation contract and
handoff for first-lesson and grading gap reports. It explains the behavior this
feature must provide: how to run the report, how to read the plain and JSON
output, which configuration is required for real Alice evidence, and which
evidence must remain visible before this feature's pull request can be called
merge-ready.

The feature is about truthful evidence reporting. It reports what the current
first-lesson evidence shows, what is still missing or blocked, and which claims
remain unsupported. It does not add or imply finished UI automation, grading,
creative review, Save completion, visible rendering correctness, completed
first-lesson work, or full Tweedle/player decode.

## Contents

- [Scope](#scope)
- [Relationship to default PR readiness](#relationship-to-default-pr-readiness)
- [Quick start](#quick-start)
- [Usage](#usage)
- [Reading the result](#reading-the-result)
- [State design rule](#state-design-rule)
- [JSON API](#json-api)
- [Configuration](#configuration)
- [Examples](#examples)
- [Merge-ready evidence checklist](#merge-ready-evidence-checklist)
- [Validation commands](#validation-commands)

## Scope

PR #160 readiness includes only first-lesson and grading gap-reporting behavior.

| In scope | Out of scope |
| --- | --- |
| Reporting missing, malformed, incomplete, unsupported, or insufficient evidence. | Claiming the UI flow is fully automated. |
| Reporting unsupported claims and specific next actions. | Claiming grading, creative assessment, Save completion, or first-lesson completion without explicit evidence. |
| Keeping readiness language scoped to PR #160 gap reporting when it describes this feature. | Keeping stale status language from another PR or recovery effort. |
| Providing a PR-specific recovery and evidence handoff for this feature. | Replacing the reusable exact-head PR gate in `docs/default-workflow-pr-readiness.md`. |
| Validating touched Rust, asset, adapter, and docs surfaces. | Adding network, credential, persistence, privileged, grading, or automation behavior. |
| Confirming the implementation branch still merges with the current `master` base before handoff. | Treating an unmerged, conflicted, aborted, or stale checkout as proof that the feature is ready. |

## Relationship to default PR readiness

This page is a case-study and feature contract for PR #160. Use it when the
review is specifically about first-lesson/grading gap reporting, evidence
boundaries, and the conservative non-claims that belong to this feature.

Use [Default-workflow PR Readiness](default-workflow-pr-readiness.md) for the
evergreen exact-head gate that applies to any pull request. PR #160 readiness
adds feature-specific evidence expectations; it does not supersede the generic
head-SHA, checks, mergeability, generated-adapter, bounded-comment,
`Files modified:`/`No-op justification:`, and `NOT_MERGE_READY` outcome gates.

## Quick start

Check an existing comparison manifest and print the human report:

```bash
cargo run -q -p eatme-cli -- alice check-lesson-readiness \
  --manifest runs/comparisons/first-lessons-real-ui-actions/local/comparison-manifest.json
```

Check the same manifest as JSON:

```bash
cargo run -q -p eatme-cli -- alice check-lesson-readiness \
  --manifest runs/comparisons/first-lessons-real-ui-actions/local/comparison-manifest.json \
  --json
```

Run the comparison and readiness report as one sequence when both Alice targets
are available:

```bash
export NODE_OPTIONS=--max-old-space-size=32768
export ALICE_BASELINE_HOME=/path/to/original-alice
export ALICE_MODERNIZED_HOME=/path/to/rabbithole-alice

EATME_REAL_ALICE=1 cargo run -q -p eatme-cli -- alice run-first-lesson-readiness \
  --run-id local-first-lesson-readiness \
  --json \
  --no-memory \
  --offline-package \
  --execute
```

The sequence fixes the scenario to `first-lessons-real-ui-actions`, writes:

```text
runs/comparisons/first-lessons-real-ui-actions/<run-id>/comparison-manifest.json
```

and immediately applies the same readiness check to that manifest.

## Usage

Use the report as an evidence gate, not as a lesson grade.

1. Run `alice check-lesson-readiness` for an existing comparison manifest, or
   `alice run-first-lesson-readiness` when collecting a new comparison and report.
2. Read the top-level `status` or the plain heading.
3. Treat every `Shown` line as a bounded evidence fact for that named claim only.
4. Treat every `Not yet shown` line as evidence to collect, repair, or leave
   blocked.
5. Preserve every `Unproven` line in handoffs, PR descriptions, and release notes.
6. Do not infer readiness from screenshots, artifact paths, action ids, Save
   dispatch, or boundary declarations alone.

The CLI remains wiring and presentation. Readiness semantics live in
`eatme-alice` so JSON, plain output, tests, and downstream runners all apply the
same conservative classifications.

## Reading the result

The report fails closed. Missing evidence, invalid evidence, unsupported desktop
actions, and partial evidence remain visible as gaps or blockers instead of
becoming readiness claims.

| Result | Meaning | Required response |
| --- | --- | --- |
| `ready` | Required bounded evidence is present and valid, every target and launch manifest has `failure_category: null`, and no unsupported-action no-go contract remains. | Use only the named shown evidence. Keep all non-claims attached. |
| `not_ready` | Required evidence is missing, malformed, unsafe, incomplete, manifest-only, out of order, or not observed. | Read `Not yet shown`, collect or repair evidence, then rerun the report. |
| `blocked` | Evidence was read, but original Alice or RabbitHole reported an explicit unsupported-action or next-action blocker. | Keep the blocker visible. Do not turn it into success wording. |

When required evidence is missing, invalid, incomplete, or insufficient, JSON
includes `evidence_gap_message` and plain output prints:

```text
Evidence gap: Missing evidence means this report cannot confirm first-lesson readiness, lesson completion, grading, or creative assessment.
```

A pure known-blocker report can be `blocked` without this gap line. If a report
has both an explicit blocker and missing, invalid, incomplete, or insufficient
required evidence, the result remains `not_ready` and includes the gap line.

## State design rule

The readiness states are intentionally stricter than artifact existence:

| State | Target and no-go condition | Evidence condition |
| --- | --- | --- |
| `ready` | Baseline and RabbitHole target records have `failure_category: null`, their launch manifests have `failure_category: null`, and `no_go_contracts[]` has no unsupported-action entries. | Required first-lesson evidence is present, valid, safe, current, and sufficient. |
| `blocked` | Required target evidence is structurally coherent, but at least one target or launch manifest reports a known unsupported UI-action `failure_category`, or a required no-go contract preserves that blocker. | Missing or invalid evidence is not the cause of the result; the blocker itself is the honest boundary. |
| `not_ready` | Target/no-go state is missing, malformed, stale, inconsistent, incomplete, or mixes blockers with missing required evidence. | Required evidence must be collected or repaired before readiness can be assessed. |

For `first-lessons-real-ui-actions`, current blocked-but-valid reports are
expected while deterministic object placement, procedure editing, visible
rendering, or project-saving affordances still surface unsupported-action
entries. Do not report `ready` merely because a manifest exists, screenshots
exist, or a blocked target has structurally valid evidence.

### Gap-reporting wording

The feature reports whether evidence supports a bounded claim. It does not turn
partial evidence into success.

| Say | Do not say unless explicit evidence exists |
| --- | --- |
| `Evidence gap: Missing evidence means this report cannot confirm first-lesson readiness, lesson completion, grading, or creative assessment.` | A completed-first-lesson claim. |
| `Grading scenario evidence is missing.` | A complete-grading claim. |
| `Creative assessment scenario evidence is missing.` | A complete-creative-assessment claim. |
| `Save option/action evidence is missing or blocked.` | A confirmed-Save-completion claim. |
| `Visible rendering scenario evidence is present, but correctness is not proven.` | `Rendering is correct.` |
| `The report is blocked by unsupported desktop action evidence.` | A full-UI-automation-complete claim. |

When reviewing readiness wording, classify each occurrence:

| Classification | Meaning | Action |
| --- | --- | --- |
| Valid negative assertion | The text says a capability is not proven or remains out of scope. | Keep it if it is clear and user-facing. |
| Stale readiness wording | The text describes another PR, old recovery state, or hard-coded readiness outside this contract. | Replace it with PR #160 gap-reporting wording or remove it. |
| Unsupported product claim | The text presents missing evidence as completion, grading, creative assessment, Save completion, or full automation. | Reword it as a gap, blocker, limitation, or next action. |

## JSON API

The readiness schema remains
`eatme.alice-lesson-session-readiness/v1`. The first-lesson sequence schema
remains `eatme.first-lesson-readiness-sequence/v1`.

The report surfaces the same conservative state in JSON and plain CLI output:

| Surface | Required behavior |
| --- | --- |
| `status` | Uses `ready`, `not_ready`, or `blocked` for the bounded evidence check. `ready` requires `failure_category: null` for each target and launch manifest plus no unsupported-action no-go contract; it is not a lesson-completion claim. |
| `evidence_gap_message` | Present when required first-lesson evidence is missing, invalid, incomplete, or insufficient. |
| `evidence_boundaries[]` | Reports Select Project, procedure/edit, Save, visible rendering, grading, creative assessment, and first-lesson completion independently. |
| `target_evidence[]` | Keeps original Alice and RabbitHole launch/action diagnostics target-local, including whether each target's `failure_category` is `null` or a known blocker. |
| `no_go_contracts[]` | Preserves unsupported desktop actions as explicit blockers. It must be empty of unsupported-action blockers before `ready` is valid. |
| Plain CLI output | Mirrors the conservative human wording and never upgrades partial evidence into success language. |

Top-level readiness fields:

| Field | Type | Description |
| --- | --- | --- |
| `schema_version` | string | `eatme.alice-lesson-session-readiness/v1`. |
| `manifest_path` | string | Comparison manifest inspected by the command. |
| `scenario_id` | string or null | Scenario being checked. |
| `passed` | boolean | `true` only when the normalized readiness `status` is `ready`; blocked or incomplete evidence stays false. |
| `status` | string | `ready`, `not_ready`, or `blocked`. |
| `readiness_status` | string | Detailed compatibility status for existing consumers. |
| `evidence_gap_message` | string or null | The fixed evidence-gap line when required first-lesson evidence is missing, invalid, incomplete, or insufficient. Pure known blockers are represented by `status: "blocked"`, `blocked_reason`, `not_yet_shown`, and `no_go_contracts`; include the gap line only when a blocker also leaves required evidence unconfirmed. |
| `desktop_proof_contract` | object | Desktop proof state rendered as the plain `Desktop proof` line. |
| `shown_evidence[]` | array | Display-safe bounded evidence facts that are shown. |
| `not_yet_shown[]` | array | Missing, invalid, not-observed, insufficient, or blocked evidence claims. |
| `desktop_next_action` | object or omitted | RabbitHole next-action summary, emitted only when valid, safe, current, and applicable. |
| `unproven_claims[]` | array | Canonical non-claims that must remain visible. |
| `evidence_boundaries[]` | array | Independent boundary states for each first-lesson claim. |
| `target_evidence[]` | array | Original Alice and RabbitHole launch/action diagnostics. |
| `no_go_contracts[]` | array | Unsupported-action entries preserved as blockers. |
| `issues[]` | array | Structural or safety problems that prevent trust in the evidence. |

Boundary entries use the same state vocabulary everywhere:

| State | Meaning |
| --- | --- |
| `present` | Explicit evidence exists for the named boundary and is safe to summarize. |
| `missing` | Evidence is absent, incomplete, or only metadata without proof for the required claim. |
| `invalid` | Evidence is malformed, unsafe, contradictory, outside the evidence root, ambiguous, or out of order. |
| `not_observed` | A producer ran but did not observe the expected boundary result. |
| `blocked` | A known unsupported desktop action or explicit next-action blocker prevents the claim from being shown. |

Presence never bubbles across boundaries. Present visible rendering evidence does
not make grading present, and present Save option/action evidence does not make
Save completion or first-lesson completion present.

The sequence schema `eatme.first-lesson-readiness-sequence/v1` wraps the same
readiness report under `readiness_report` and repeats
`evidence_gap_message` at the sequence top level. Both values must match when
the gap message is present.

## Configuration

Use the repository's saved Node heap preference for agentic and Gadugi-heavy
local runs:

```bash
export NODE_OPTIONS=--max-old-space-size=32768
```

Real Alice execution remains explicitly gated:

```bash
export EATME_REAL_ALICE=1
export ALICE_BASELINE_HOME=/path/to/original-alice
export ALICE_MODERNIZED_HOME=/path/to/rabbithole-alice
```

| Setting | Required for | Description |
| --- | --- | --- |
| `NODE_OPTIONS=--max-old-space-size=32768` | Agentic/Gadugi-heavy local runs | Prevents surrounding Node-based runner workloads from running with a small heap. |
| `EATME_REAL_ALICE=1` | Non-baseline desktop execution | Explicit opt-in for real Alice scenarios. |
| `ALICE_BASELINE_HOME` | `alice run-first-lesson-readiness --execute` | Original Alice checkout. |
| `ALICE_MODERNIZED_HOME` | `alice run-first-lesson-readiness --execute` | RabbitHole Alice checkout. |

No configuration setting changes the evidence semantics. Missing evidence remains
missing, blockers remain blockers, and no setting enables automated creative
grading, quality judgment, Save completion, visible rendering correctness, or
lesson completion marking.

Do not add workflow timeout settings for this feature. The report consumes
existing evidence for the selected run and uses the repository's normal command
behavior.

## Examples

### Missing grading and creative-assessment evidence

Example JSON excerpt for grading and creative assessment gaps:

```json
{
  "schema_version": "eatme.alice-lesson-session-readiness/v1",
  "status": "not_ready",
  "evidence_gap_message": "Evidence gap: Missing evidence means this report cannot confirm first-lesson readiness, lesson completion, grading, or creative assessment.",
  "evidence_boundaries": [
    {
      "id": "grading",
      "label": "Grading scenario evidence",
      "status": "missing",
      "detail": "Grading scenario evidence is missing.",
      "claim": "Grading scenario evidence is not proven; gap reports must collect explicit scenario evidence before this can be reported as present.",
      "does_not_prove": [
        "creative assessment",
        "first-lesson completion"
      ]
    },
    {
      "id": "creative_assessment",
      "label": "Creative assessment scenario evidence",
      "status": "missing",
      "detail": "Creative assessment scenario evidence is missing. The report can surface available evidence and suggest next steps for the learner's creative work in this scenario, but it does not grade creativity, judge quality, or mark the lesson complete.",
      "claim": "Creative assessment scenario evidence is not proven; gap reports must collect explicit scenario evidence before this can be reported as present.",
      "does_not_prove": [
        "instructor judgment",
        "first-lesson completion"
      ]
    }
  ]
}
```

Example plain output excerpt:

```text
First-lesson/grading gap report: not ready
Gap report scope: missing/incomplete evidence, unsupported claims, and next actions only.
Evidence gap: Missing evidence means this report cannot confirm first-lesson readiness, lesson completion, grading, or creative assessment.

Not yet shown:
- Grading is not yet shown.
- Creative assessment is not yet shown.

Unproven:
- Full Alice UI automation is not proven.
- Grading is not proven.
- Creative assessment is not proven.
- Visible rendering correctness is not proven.
- Save completion is not proven.
- First-lesson completion is not proven.
```

Use JSON for machine checks.

### Blocked desktop action

When a desktop action is explicitly unsupported, keep the blocker visible:

```json
{
  "schema_version": "eatme.alice-lesson-session-readiness/v1",
  "status": "blocked",
  "blocked_reason": "action_contract_blocked_until_ui_automation",
  "not_yet_shown": [
    {
      "id": "desktop_action",
      "state": "blocked",
      "summary": "Desktop action evidence is blocked by an unsupported UI action.",
      "detail": "The report found an explicit unsupported-action blocker and did not convert it into readiness.",
      "does_not_prove": [
        "full Alice UI automation",
        "first-lesson completion"
      ]
    }
  ],
  "unproven_claims": [
    "Full Alice UI automation is not proven.",
    "Grading is not proven.",
    "Creative assessment is not proven.",
    "Visible rendering correctness is not proven.",
    "Save completion is not proven.",
    "First-lesson completion is not proven."
  ]
}
```

Plain output keeps the same meaning:

```text
First-lesson/grading gap report: blocked
Gap report scope: missing/incomplete evidence, unsupported claims, and next actions only.

Not yet shown:
- Desktop action evidence is blocked by an unsupported UI action.

Unproven:
- Full Alice UI automation is not proven.
- Grading is not proven.
- Creative assessment is not proven.
- Visible rendering correctness is not proven.
- Save completion is not proven.
- First-lesson completion is not proven.
```

### Safe Save wording

Save option/action evidence, Save shortcut evidence, and Save proof-artifact availability
can be shown without proving Save completion.

Safe wording:

```text
Save option/action evidence is shown as observed option/action only.
Save completion is not yet proven.
```

Unsafe wording:

```text
The project was saved successfully.
The first lesson was completed.
```

### Review a pull request for readiness

Use this workflow before calling a gap-reporting PR merge-ready:

1. Confirm the branch is the current remote PR head.
2. Review the diff against the base branch and confirm it is focused on
   first-lesson gap reporting, readiness presentation, tests, configuration, and
   directly related docs.
3. Run the PR #160 local evidence commands from
   [Validation commands](#validation-commands), not only a subset inferred from
   the final diff.
4. Perform at least three SEEK/VALIDATE/FIX cycles. `FIX: no repository change
   required` is acceptable only when the validation evidence proves no defect for
   that cycle.
5. Confirm the final quality-audit cycle is clean.
6. Confirm GitHub Actions are green.
7. Confirm the PR description contains bounded evidence for QA/scenarios, docs
   impact, audit cycles, Actions, diff scope, and non-claims.
8. If any item is missing, mark the PR with an explicit `NOT_MERGE_READY`
   blocker instead of claiming readiness.

## Merge-ready evidence checklist

Green checks and workflow completion are required, but they are not sufficient.
A PR using this feature is merge-ready only when all evidence is present:

| Evidence | Required content |
| --- | --- |
| Runnable QA/scenario evidence | Output from applicable repository commands, bounded to what each command proves. |
| Docs impact | Statement that changed docs were reviewed and the docs build was relevant, or a bounded explanation that docs were not touched. |
| Quality-audit cycles | At least three SEEK/VALIDATE/FIX cycles, with a clean final cycle. |
| GitHub Actions | Green Actions for the current PR head. |
| Focused diff scope | Diff against the base branch is limited to the gap-reporting/readiness feature and directly related checks/docs. |
| PR description evidence | PR body records the bounded evidence and keeps non-claims visible. |

Use `NOT_MERGE_READY` when evidence is missing, stale, unavailable, or broader
than what was proven. Do not claim full UI automation, visible rendering
correctness, grading, creative assessment, Save completion, first-lesson
completion, or full Tweedle/player decode unless directly proven by explicit
evidence for that exact claim.

## Recovery no-op guard for requested head

PR #160 previously reached a clean branch state without repository file changes.
That is not an accepted default-workflow implementation result unless the
handoff prints a literal exact-head section like this. Fill
`<exact-head-sha>` from `git rev-parse HEAD` and confirm it matches the PR
`headRefOid` before using this as evidence:

```text
No-op justification:
- Exact head: <exact-head-sha> matches the PR head.
- GitHub Actions: green for that exact head.
- Focused diff: focused PR diff against base `master` is limited to
  first-lesson gap reporting, readiness presentation, tests, docs, and directly
  related configuration.
- Runnable QA/scenario evidence: direct, no-timeout local evidence commands were
  run for applicable Rust, asset, generated-adapter, docs, and full quality-gate
  surfaces, with each result bounded to what it proves.
- Docs impact: documentation impact was checked; changed docs were built with
  `mkdocs build --strict`, or unchanged docs were explicitly recorded as such.
- Quality-audit cycles: at least three SEEK/VALIDATE/FIX cycles were recorded,
  and the final cycle was clean.
- PR description evidence: the pull request body records current-head evidence
  for QA/scenarios, docs impact, audit cycles, Actions, diff scope, and
  non-claims.
- Remaining blockers: list every missing or inconclusive merge-ready gate, or
  state that there are no remaining blockers only when all gates above are
  current for the exact head.
```

If that exact section cannot be supported by current-head evidence, the recovery
must modify repository files or emit `NOT_MERGE_READY` with explicit blockers.
The branch must not silently succeed, and green checks alone must not be used to
claim merge readiness from a zero-file diff.

## Validation commands

For PR #160 recovery or final handoff, run these local evidence commands from
the repository root and record the bounded result for each:

```bash
TMPDIR=/tmp cargo test -p eatme-alice --all-features
TMPDIR=/tmp cargo test -p eatme-cli --all-features
cargo run -q -p eatme-cli -- assets validate --json
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
mkdocs build --strict
TMPDIR=/tmp ./scripts/quality-gates.sh
```

The full quality gate does not replace the explicit evidence list in the handoff:
reviewers still need to see which Rust, asset, adapter, and docs checks were run
and what each command proves.

For future, narrower changes, run the checks that match the touched surfaces:

| Surface touched | Required check |
| --- | --- |
| First-lesson or lesson-readiness Rust | `TMPDIR=/tmp cargo test -p eatme-alice --all-features` |
| CLI rendering | `TMPDIR=/tmp cargo test -p eatme-cli --all-features` |
| Persona or scenario assets | `cargo run -q -p eatme-cli -- assets validate --json` |
| Generated Gadugi adapters or scenario assets | `cargo run -q -p eatme-cli -- assets generate-gadugi --check --json` |
| Documentation | `mkdocs build --strict` |
| Full handoff | `TMPDIR=/tmp ./scripts/quality-gates.sh` |

If a validation failure is unrelated to PR #160, leave the failure visible in
the handoff instead of weakening the gap-reporting contract.
