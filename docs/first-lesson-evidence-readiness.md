# [PLANNED - Implementation Pending] First-lesson evidence readiness

This document describes the first-lesson evidence boundary reporting feature to
build on top of the current lesson-readiness JSON API.

The feature will report whether first-lesson automation scenarios have enough
RabbitHole evidence to make each bounded first-lesson claim. The report is
conservative: missing, malformed, ambiguous, unsafe, or uncertain evidence stays
visible as a blocker. Boundary metadata can show that an automation boundary was
declared or observed, but it does not prove full Alice UI automation, visible
rendering correctness, bounded Save completion, grading, creative assessment, or
first-lesson completion unless the matching boundary evidence is present.

## Current implementation compatibility

Current `alice check-lesson-readiness` and `alice run-first-lesson-readiness`
commands emit schema `eatme.alice-lesson-session-readiness/v1` with
`evidence_progress.items[]`, `required_evidence`, `desktop_proof_contract`,
`lesson_session_readiness`, and `role_readiness`.

Current reports do not yet emit `evidence_boundaries[]`, and current plain output
does not yet use the scenario-focused examples below as its primary format.
Until the planned feature lands, consumers should read:

- `status`, `readiness_status`, `passed`, and `issues` for the top-level result;
- `evidence_progress.items[]` for current evidence states;
- `Save Project proof artifact` and `Select Project proof artifact` entries for
  current project proof-artifact availability or blockers;
- `limitations` for non-claims that remain true even when structural evidence is
  present.

The planned feature is additive. It must not remove or rename existing fields.

## Quick start

Run the current readiness check against a first-lesson comparison manifest:

```bash
cargo run -q -p eatme-cli -- alice check-lesson-readiness \
  --manifest runs/comparisons/first-lessons-real-ui-actions/local/comparison-manifest.json \
  --json
```

Run the bounded first-lesson comparison and readiness sequence:

```bash
export NODE_OPTIONS=--max-old-space-size=32768
export ALICE_BASELINE_HOME=/path/to/alice-reference
export ALICE_MODERNIZED_HOME=/path/to/alice-candidate

EATME_REAL_ALICE=1 cargo run -q -p eatme-cli -- alice run-first-lesson-readiness \
  --run-id local-first-lesson-readiness \
  --timeout 900 \
  --json \
  --no-memory \
  --offline-package \
  --execute
```

After this planned feature is implemented, the same commands will add
boundary-specific reporting to the current readiness report.

## What the planned report decides

The top-level readiness result answers one question:

> Do the automation scenarios have explicit evidence for the bounded
> first-lesson claims being reported?

It does not answer whether a learner completed the lesson, whether an Alice
world is creatively successful, whether a saved project should receive a grade,
or whether rendering is correct.

| Result | Meaning | What to do |
| --- | --- | --- |
| `ready` | Every required boundary has explicit evidence and no known blocker remains. | Use the report only for the bounded automation scenario claims it names. |
| `not_ready` | Required evidence is missing, malformed, ambiguous, unsafe, incomplete, or uncertain. | Treat each missing or invalid boundary as a blocker and collect or repair the evidence. |
| `blocked` | Evidence exists for some boundaries, but a known unsupported desktop action or explicit RabbitHole blocker prevents a claim. | Keep the blocker visible. Do not convert it into success or hide it as a generic failure. |

## Planned evidence boundaries

When implemented, `evidence_boundaries[]` is mandatory for first-lesson readiness
reports and must include every boundary id in this table. Each boundary is
reported independently so one present boundary cannot imply another.

| Boundary id | Human label | Required evidence | Must not imply |
| --- | --- | --- | --- |
| `select_project` | Select Project scenario evidence | Explicit evidence that the Select Project boundary produced a safe, auditable scenario signal. | Full Alice UI automation, project selection success beyond the named boundary, or first-lesson completion. |
| `procedure_edit` | Procedure/edit scenario evidence | Explicit evidence that a procedure or code edit boundary was completed or observed with a safe summary. | Code correctness, learner understanding, grading, or completed lesson work. |
| `save_project` | Save scenario evidence | Explicit bounded Save completion evidence, such as a safe saved-project summary from the evidence root. Dispatching a Save shortcut, declaring a Save boundary, or reporting artifact availability without a completion signal is not enough. | Lesson completion, grading, creative assessment, or broad desktop Save behavior beyond the bounded evidence. |
| `visible_rendering` | Visible rendering scenario evidence | Explicit visible rendering evidence from the run boundary. A screenshot may support this only when the evidence says what was observed. | Rendering correctness, animation correctness, creative quality, or complete visual validation. |
| `grading` | Grading scenario evidence | Explicit grading evidence from a scenario that owns grading. | Any automatic grade when no grading evidence exists. |
| `creative_assessment` | Creative assessment scenario evidence | Explicit creative assessment evidence from a scenario that owns creative review. | Automated creativity judgment, instructor judgment, or learner-world grading. |
| `first_lesson_completion` | First-lesson completion scenario evidence | Explicit completion evidence from the first-lesson scenario. Boundary declarations, observed substeps, launch evidence, rendering evidence, Save evidence, or grading evidence do not prove completion by themselves. | Completed first lesson unless the completion boundary itself is present. |

## Planned status vocabulary

All planned boundary entries use the same status vocabulary.

| Status | Use when | Readiness effect |
| --- | --- | --- |
| `present` | Explicit evidence exists for the named boundary and is safe to summarize. | Supports only that boundary's bounded claim. |
| `missing` | Evidence is absent, incomplete, has no safe summary, or only declares metadata without proof for the required claim. | Blocks readiness. |
| `invalid` | Evidence is malformed, unsafe, contradictory, outside the evidence root, or ambiguous. | Blocks readiness and should appear in `issues`. |
| `not_observed` | A producer ran but did not observe the expected boundary result. | Blocks readiness. |
| `blocked` | RabbitHole supplied an explicit blocker, or the scenario still lacks deterministic desktop support. | Produces `status: "blocked"` when all other required structure is coherent; otherwise contributes to `not_ready`. |

Current Save Project and Select Project proof-artifact entries only distinguish
`present`, `missing`, and `blocked` in `evidence_progress.items[]`. The planned
boundary layer may map unsafe or malformed project evidence to `invalid`, but it
must not treat artifact availability alone as Save completion.

Presence never bubbles up across boundaries. For example, present visible
rendering evidence does not make grading present, and present Save scenario
evidence does not make first-lesson completion present.

## Planned human output contract

Human output will be written for reviewers who need to decide what is currently
blocked. It should use `scenarios` or `automation scenarios` wording and keep
implementation details out of the primary report.

Planned example with missing evidence:

```text
First-lesson automation scenario readiness: not ready

Evidence present:
- Alice launch scenario evidence is present.
- Visible rendering scenario evidence is present.

Blockers:
- Select Project scenario evidence is missing.
- Procedure/edit scenario evidence is missing.
- Save scenario evidence is missing.
- Grading scenario evidence is missing.
- Creative assessment scenario evidence is missing.
- First-lesson completion scenario evidence is missing.

This report does not prove full Alice UI automation, visible rendering
correctness, bounded Save completion, grading, creative assessment, or
first-lesson completion.
```

Planned example with a known blocker:

```text
First-lesson automation scenario readiness: blocked

Evidence present:
- Select Project scenario evidence is present.
- Procedure/edit scenario evidence is present.
- Visible rendering scenario evidence is present.

Blockers:
- Save scenario evidence is blocked: bounded Save completion evidence was not
  produced by this run.
- First-lesson completion scenario evidence is missing.

This report does not prove bounded Save completion or first-lesson completion.
```

Human output may include a short evidence-root-relative summary after the plain
label, but it must not expose absolute paths, raw artifact contents, action ids,
framework labels, or implementation-detail artifact names as the main message.

## Planned JSON API addition

The readiness schema remains `eatme.alice-lesson-session-readiness/v1`. Existing
fields stay stable. Boundary reporting is additive and appears in
`evidence_boundaries` after this feature is implemented. Existing
`evidence_progress.items[]` entries remain available for older consumers.

Top-level fields after implementation:

| Field | Type | Description |
| --- | --- | --- |
| `schema_version` | string | Readiness report schema. |
| `scenario_id` | string or null | Scenario being checked. |
| `passed` | boolean | Structural evidence check result. A blocked report can still have `passed: true` when the remaining blocker is explicit. |
| `status` | string | `ready`, `not_ready`, or `blocked`. |
| `readiness_status` | string | Backward-compatible detailed status. |
| `human_summary` | string | Plain scenario-focused summary. |
| `evidence_progress` | object | Backward-compatible progress counts and items. |
| `evidence_boundaries` | array | Mandatory boundary-specific evidence states for first-lesson readiness after this feature is implemented. |
| `role_readiness` | array | Role-specific readiness envelopes. |
| `lesson_session_readiness` | object | Backward-compatible student readiness envelope. |
| `issues` | array of strings | Blocking structural problems. |
| `limitations` | array of strings | Non-claims that remain true even when evidence is present. |

### `evidence_boundaries[]`

Each boundary entry has this planned shape:

| Field | Type | Description |
| --- | --- | --- |
| `id` | string | Stable boundary id: `select_project`, `procedure_edit`, `save_project`, `visible_rendering`, `grading`, `creative_assessment`, or `first_lesson_completion`. |
| `label` | string | Human-readable scenario evidence label. |
| `status` | string | `present`, `missing`, `invalid`, `not_observed`, or `blocked`. |
| `source` | string or null | Short source category, such as `rabbithole`, `comparison_manifest`, or `scenario_asset`. |
| `metadata_state` | string or null | Optional boundary metadata state, such as `declared` or `observed`. Metadata state never upgrades the boundary to completion proof. |
| `detail` | string | Plain detail safe for JSON consumers and CLI summaries. |
| `claim` | string | The exact bounded claim this boundary supports when `status` is `present`. |
| `does_not_prove` | array of strings | Claims that remain unsupported by this boundary. |

Planned excerpt:

```json
{
  "schema_version": "eatme.alice-lesson-session-readiness/v1",
  "scenario_id": "first-lessons-real-ui-actions",
  "status": "not_ready",
  "passed": false,
  "human_summary": "First-lesson automation scenarios are not ready because required scenario evidence is missing.",
  "evidence_boundaries": [
    {
      "id": "select_project",
      "label": "Select Project scenario evidence",
      "status": "present",
      "source": "rabbithole",
      "metadata_state": "observed",
      "detail": "Select Project scenario evidence is present.",
      "claim": "The Select Project boundary has auditable scenario evidence.",
      "does_not_prove": [
        "full Alice UI automation",
        "first-lesson completion"
      ]
    },
    {
      "id": "save_project",
      "label": "Save scenario evidence",
      "status": "missing",
      "source": "rabbithole",
      "metadata_state": "declared",
      "detail": "Save scenario metadata was declared, but bounded Save completion evidence is missing.",
      "claim": "No Save completion claim is supported.",
      "does_not_prove": [
        "bounded Save completion",
        "first-lesson completion",
        "learner-world grading"
      ]
    },
    {
      "id": "first_lesson_completion",
      "label": "First-lesson completion scenario evidence",
      "status": "missing",
      "source": "rabbithole",
      "metadata_state": null,
      "detail": "First-lesson completion scenario evidence is missing.",
      "claim": "No first-lesson completion claim is supported.",
      "does_not_prove": [
        "completed first lesson"
      ]
    }
  ]
}
```

## Tutorials

### Evaluate current blockers

1. Run the readiness command with `--json`.
2. Read the top-level `status`.
3. Until `evidence_boundaries[]` exists, inspect `evidence_progress.items[]`.
4. After `evidence_boundaries[]` exists, treat every `missing`, `invalid`,
   `not_observed`, or `blocked` boundary as a blocker.
5. Do not infer completion from counts, artifact presence, screenshot presence,
   Save dispatch, action ids, or boundary declarations.

### Review a present boundary safely

When a planned boundary reports `present`, use the `claim` field as the complete
claim. Then read `does_not_prove` before writing a PR, issue, or classroom
handoff.

For example, a present `visible_rendering` boundary can support:

```text
Visible rendering scenario evidence is present.
```

It cannot support:

```text
Rendering is correct.
The animation is visually correct.
The first lesson is complete.
```

### Repair a missing Save boundary

If Save scenario evidence is `missing`, collect or repair bounded Save
completion evidence and rerun readiness. Do not treat any of these as Save
completion:

- a Save command dispatch without completion evidence;
- a boundary declaration without an observed completion signal;
- a saved-project artifact availability entry without a completion signal;
- a saved-project path outside the evidence root;
- a screenshot that does not explicitly prove Save completion;
- grading or completion evidence from another boundary.

## Writing readiness-related docs and PRs

Use scenario-focused wording in user-facing text:

| Say | Avoid in primary human output |
| --- | --- |
| `Select Project scenario evidence is missing.` | `select_project_proof_artifact declaration is missing.` |
| `Save scenario evidence is blocked.` | `save_project_desktop_shortcut_dispatch failed.` |
| `First-lesson automation scenarios are not ready.` | `ui-action-contract no_go status failed.` |
| `Visible rendering scenario evidence is present, but correctness is not proven.` | `pixel proof passed, so rendering is correct.` |

It is acceptable for JSON reference sections to document stable field names.
Primary human output should stay plain, scenario-focused, and conservative.
