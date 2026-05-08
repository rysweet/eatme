# First-lesson evidence readiness

First-lesson evidence readiness reports whether the automation scenarios have
enough RabbitHole evidence to make the next bounded first-lesson claim.

The report is conservative. Missing, malformed, ambiguous, unsafe, or uncertain
evidence is a blocker. Boundary metadata can show that an automation scenario
boundary was declared or observed, but it does not prove full Alice UI
automation, visible rendering correctness, desktop Save completion, grading,
creative assessment, or first-lesson completion unless the matching evidence
boundary is present.

## Quick start

Run a readiness check against a first-lesson comparison manifest:

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

Without `--json`, the same command prints a scenario-focused report for humans.
The human report names ready evidence and blockers in plain terms. It does not
print internal action ids, framework labels, raw artifact names, or absolute
host paths as the primary explanation.

## What the report decides

The top-level readiness result answers one question:

> Do the automation scenarios have explicit evidence for the bounded
> first-lesson claims being reported?

It does not answer whether a learner completed the lesson, whether an Alice
world is creatively successful, whether a saved project should receive a grade,
or whether rendering is correct.

| Result | Meaning | What to do |
| --- | --- | --- |
| `ready` | Every required boundary has explicit evidence and no known blocker remains. | Use the report only for the bounded automation scenario claims it names. |
| `not_ready` | Required evidence is missing, malformed, ambiguous, unsafe, incomplete, or uncertain. | Treat each missing boundary as a blocker and collect or repair the evidence. |
| `blocked` | Evidence exists for some boundaries, but a known unsupported desktop action or explicit RabbitHole blocker prevents a claim. | Keep the blocker visible. Do not convert it into success or hide it as a generic failure. |

## Evidence boundaries

Readiness consumes RabbitHole evidence into these boundary-specific states. Each
boundary is reported independently so one present boundary cannot imply another.

| Boundary id | Human label | Required evidence | Must not imply |
| --- | --- | --- | --- |
| `select_project` | Select Project scenario evidence | Explicit evidence that the Select Project boundary produced a safe, auditable scenario signal. | Full Alice UI automation, project selection success beyond the named boundary, or first-lesson completion. |
| `procedure_edit` | Procedure/edit scenario evidence | Explicit evidence that a procedure or code edit boundary was completed or observed with a safe summary. | Code correctness, learner understanding, grading, or completed lesson work. |
| `save_project` | Save scenario evidence | Explicit desktop Save completion evidence, such as a safe saved-project summary from the evidence root. Dispatching a Save shortcut or declaring a Save boundary is not enough. | Desktop Save completion when only dispatch, metadata, or artifact presence exists. |
| `visible_rendering` | Visible rendering scenario evidence | Explicit visible rendering evidence from the run boundary. A screenshot may support this only when the evidence says what was observed. | Rendering correctness, animation correctness, creative quality, or complete visual validation. |
| `grading` | Grading scenario evidence | Explicit grading evidence from a scenario that owns grading. | Any automatic grade when no grading evidence exists. |
| `creative_assessment` | Creative assessment scenario evidence | Explicit creative assessment evidence from a scenario that owns creative review. | Automated creativity judgment, instructor judgment, or learner-world grading. |
| `first_lesson_completion` | First-lesson completion scenario evidence | Explicit completion evidence from the first-lesson scenario. Boundary declarations, observed substeps, launch evidence, rendering evidence, Save evidence, or grading evidence do not prove completion by themselves. | Completed first lesson unless the completion boundary itself is present. |

If a boundary is not represented in the evidence, it remains visible as
`missing`. If a producer supplies malformed or unsafe evidence, the boundary is
`invalid`. If a producer ran but did not observe the expected result, the
boundary is `not_observed`. If RabbitHole reports a known blocker, the boundary
is `blocked`.

## Human output contract

Human output is written for reviewers who need to decide what is currently
blocked. It uses `scenarios` or `automation scenarios` wording and keeps
implementation details out of the primary report.

Example with missing evidence:

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
correctness, desktop Save completion, grading, creative assessment, or
first-lesson completion.
```

Example with a known blocker:

```text
First-lesson automation scenario readiness: blocked

Evidence present:
- Select Project scenario evidence is present.
- Procedure/edit scenario evidence is present.
- Visible rendering scenario evidence is present.

Blockers:
- Save scenario evidence is blocked: desktop Save completion evidence was not
  produced by this run.
- First-lesson completion scenario evidence is missing.

This report does not prove desktop Save completion or first-lesson completion.
```

Human output may include a short evidence-root-relative summary after the plain
label, but it must not expose absolute paths, raw artifact contents, action ids,
framework labels, or implementation-detail artifact names as the main message.

## JSON API

The readiness schema remains
`eatme.alice-lesson-session-readiness/v1`. Existing fields stay stable.
Boundary reporting is additive and appears in `evidence_boundaries`. Existing
`evidence_progress.items[]` entries may mirror the same states for older
consumers.

Top-level fields:

| Field | Type | Description |
| --- | --- | --- |
| `schema_version` | string | Readiness report schema. |
| `scenario_id` | string or null | Scenario being checked. |
| `passed` | boolean | Structural evidence check result. A blocked report can still have `passed: true` when the remaining blocker is explicit. |
| `status` | string | `ready`, `not_ready`, or `blocked`. |
| `readiness_status` | string | Backward-compatible detailed status. |
| `human_summary` | string | Plain scenario-focused summary. |
| `evidence_progress` | object | Backward-compatible progress counts and items. |
| `evidence_boundaries` | array | Boundary-specific evidence states for first-lesson readiness. |
| `role_readiness` | array | Role-specific readiness envelopes. |
| `lesson_session_readiness` | object | Backward-compatible student readiness envelope. |
| `issues` | array of strings | Blocking structural problems. |
| `limitations` | array of strings | Non-claims that remain true even when evidence is present. |

### `evidence_boundaries[]`

Each boundary entry has this shape:

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

Example:

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
      "detail": "Save scenario metadata was declared, but desktop Save completion evidence is missing.",
      "claim": "No Save completion claim is supported.",
      "does_not_prove": [
        "desktop Save completion",
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

### Status normalization

Normalize evidence conservatively:

| Status | Use when | Readiness effect |
| --- | --- | --- |
| `present` | Explicit evidence exists for the named boundary and is safe to summarize. | Supports only that boundary's bounded claim. |
| `missing` | Evidence is absent, incomplete, has no safe summary, or only declares metadata without proof for the required claim. | Blocks readiness. |
| `invalid` | Evidence is malformed, unsafe, contradictory, outside the evidence root, or ambiguous. | Blocks readiness and should appear in `issues`. |
| `not_observed` | A producer ran but did not observe the expected boundary result. | Blocks readiness. |
| `blocked` | RabbitHole supplied an explicit blocker or the scenario still lacks deterministic desktop support. | Produces `status: "blocked"` when all other required structure is coherent; otherwise contributes to `not_ready`. |

Presence never bubbles up across boundaries. For example, present visible
rendering evidence does not make grading present, and present Save scenario
evidence does not make first-lesson completion present.

## Configuration

| Variable | Required for | Description |
| --- | --- | --- |
| `NODE_OPTIONS=--max-old-space-size=32768` | Agentic or adapter-heavy local workflows | Saved local preference for Node-backed runners. The Rust readiness parser does not require Node. |
| `EATME_REAL_ALICE=1` | Non-baseline real Alice execution | Explicit opt-in for desktop runs. |
| `ALICE_BASELINE_HOME` | `alice run-first-lesson-readiness --execute` | Baseline Alice checkout. |
| `ALICE_MODERNIZED_HOME` | `alice run-first-lesson-readiness --execute` | RabbitHole or candidate Alice checkout. |
| `ALICE_HOME` | Single-target launch smoke commands | Alice checkout for direct launch smoke runs. |

Real desktop evidence also requires the Alice desktop dependency set documented
in [Alice Integration](alice-integration.md).

## Tutorials

### Evaluate current blockers

1. Run the readiness command with `--json`.
2. Read the top-level `status`.
3. Inspect `evidence_boundaries[]`.
4. Treat every `missing`, `invalid`, `not_observed`, or `blocked` boundary as a
   blocker.
5. Do not infer completion from counts, artifact presence, screenshot presence,
   Save dispatch, action ids, or boundary declarations.

### Review a present boundary safely

When a boundary reports `present`, use the `claim` field as the complete claim.
Then read `does_not_prove` before writing a PR, issue, or classroom handoff.

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

If Save scenario evidence is `missing`, collect or repair the desktop Save
completion evidence and rerun readiness. Do not treat any of these as Save
completion:

- a Save command dispatch without completion evidence;
- a boundary declaration without an observed completion signal;
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
