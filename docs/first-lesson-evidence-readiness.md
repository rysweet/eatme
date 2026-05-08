# First-lesson evidence readiness

First-lesson evidence readiness is the comparison-runner contract for original
Alice and RabbitHole automation scenarios. It combines per-target launch/action
evidence with scenario boundary states so the runner only reports bounded
first-lesson action claims that have explicit, executable evidence.

The report is conservative. Missing, malformed, ambiguous, unsafe, incomplete,
manifest-only, out-of-order, unsupported, or uncertain evidence stays visible as
a blocker. Boundary metadata can show that an action was declared or observed,
but metadata alone does not prove full Alice UI automation, visible rendering
correctness, bounded Save completion, grading, creative assessment, or
first-lesson completion.

## Quick start

Run the readiness check against a first-lesson comparison manifest:

```bash
cargo run -q -p eatme-cli -- alice check-lesson-readiness \
  --manifest runs/comparisons/first-lessons-real-ui-actions/local/comparison-manifest.json \
  --json
```

Run the bounded first-lesson comparison and readiness sequence:

```bash
export NODE_OPTIONS=--max-old-space-size=32768
export ALICE_BASELINE_HOME=/path/to/original-alice
export ALICE_MODERNIZED_HOME=/path/to/rabbithole-alice

EATME_REAL_ALICE=1 cargo run -q -p eatme-cli -- alice run-first-lesson-readiness \
  --run-id local-first-lesson-readiness \
  --timeout 900 \
  --json \
  --no-memory \
  --offline-package \
  --execute
```

The sequence fixes the scenario to `first-lessons-real-ui-actions`, writes a
comparison manifest under:

```text
runs/comparisons/first-lessons-real-ui-actions/<run-id>/comparison-manifest.json
```

and immediately applies the same readiness check to that manifest.

## What the report decides

The top-level readiness result answers one question:

> Do the original Alice and RabbitHole automation scenarios have explicit,
> target-local evidence for every bounded first-lesson claim being reported?

It does not answer whether a learner completed the lesson, whether an Alice
world is creatively successful, whether a saved project should receive a grade,
or whether rendering is correct.

| Result | Meaning | What to do |
| --- | --- | --- |
| `ready` | Every required target and boundary has explicit evidence and no known blocker remains. | Use the report only for the bounded claims named by the automation scenarios. |
| `not_ready` | Required evidence is missing, malformed, ambiguous, unsafe, incomplete, manifest-only, out of order, or uncertain. | Treat each invalid or missing boundary as a blocker and collect or repair the evidence. |
| `blocked` | Evidence exists for some boundaries, but a known unsupported desktop action or explicit RabbitHole blocker prevents a claim. | Keep the blocker visible. Do not convert it into success or hide it as a generic failure. |

## Required comparison evidence

First-lesson readiness requires both comparison targets:

| Target role | Meaning | Required evidence |
| --- | --- | --- |
| `baseline` | Original Alice target | Launch manifest, `ui-action-contract.json`, required action entries, action assertions, and any unsupported-action entries for original Alice. |
| `modernized` | RabbitHole target | The same launch/action evidence as baseline, plus RabbitHole desktop evidence such as Run-window observation, desktop execution observation, visible screenshot evidence, and project proof-artifact states. |

The comparison manifest must use `scenario_id: "first-lessons-real-ui-actions"`.
It must be produced with execution enabled to be `ready` or to support present
executable evidence. Manifest-only comparisons are valid inputs, but they report
`not_ready` because they do not contain executable evidence for either target.

Readiness fails closed for evidence that cannot be safely resolved under the
comparison evidence root. Absolute paths, parent traversal, symlink escapes,
empty artifacts, unreadable files, malformed JSON, and artifact references
outside the evidence root are not accepted as present evidence.

## Evidence boundaries

First-lesson readiness has two evidence layers:

| Layer | Granularity | Purpose |
| --- | --- | --- |
| `target_evidence[]` | One entry per comparison target. | Shows original Alice and RabbitHole launch/action evidence, missing actions, and unsupported-action blockers. |
| `evidence_boundaries[]` | One entry per bounded scenario claim. | Shows whether the named first-lesson claim is present, missing, invalid, not observed, or blocked after the required target evidence is considered. |

`evidence_boundaries[]` is mandatory in first-lesson readiness reports. It names
each bounded scenario evidence claim independently so one present boundary cannot
imply another. Boundary entries are not the per-target API; use
`target_evidence[]` for target-local launch/action diagnostics.

| Boundary id | Human label | Required evidence | Must not imply |
| --- | --- | --- | --- |
| `select_project` | Select Project scenario evidence | Explicit evidence that the Select Project boundary produced a safe, auditable scenario signal. | Full Alice UI automation, project selection success beyond the named boundary, or first-lesson completion. |
| `procedure_edit` | Procedure/edit scenario evidence | Explicit evidence that a procedure or code edit boundary was completed or observed with a safe summary. | Code correctness, learner understanding, grading, or completed lesson work. |
| `save_project` | Save scenario evidence | Explicit bounded Save evidence, such as a safe saved-project summary from the evidence root. Dispatching a Save shortcut, declaring a Save boundary, or reporting artifact availability without a completion signal is not enough. | Lesson completion, grading, creative assessment, or broad desktop Save behavior beyond the bounded evidence. |
| `visible_rendering` | Visible rendering scenario evidence | Explicit visible rendering evidence from the run boundary. A screenshot may support this only when the evidence says what was observed. | Rendering correctness, animation correctness, creative quality, or complete visual validation. |
| `grading` | Grading scenario evidence | Explicit grading evidence from a scenario that owns grading. | Any automatic grade when no grading evidence exists. |
| `creative_assessment` | Creative assessment scenario evidence | Explicit creative assessment evidence from a scenario that owns creative review. | Automated creativity judgment, instructor judgment, or learner-world grading. |
| `first_lesson_completion` | First-lesson completion scenario evidence | Explicit completion evidence from the first-lesson scenario. Boundary declarations, observed substeps, launch evidence, rendering evidence, Save evidence, or grading evidence do not prove completion by themselves. | Completed first lesson unless the completion boundary itself is present. |

## Status vocabulary

All boundary entries use the same status vocabulary.

| Status | Use when | Readiness effect |
| --- | --- | --- |
| `present` | Explicit evidence exists for the named boundary and is safe to summarize. | Supports only that boundary's bounded claim. |
| `missing` | Evidence is absent, incomplete, has no safe summary, or only declares metadata without proof for the required claim. | Blocks readiness. |
| `invalid` | Evidence is malformed, unsafe, contradictory, outside the evidence root, ambiguous, or out of order. | Blocks readiness and appears in `issues`. |
| `not_observed` | A producer ran but did not observe the expected boundary result. | Blocks readiness. |
| `blocked` | RabbitHole supplied an explicit blocker, original Alice reports a known unsupported action, or the scenario lacks deterministic desktop support. | Produces `status: "blocked"` when all other required structure is coherent; otherwise contributes to `not_ready`. |

Presence never bubbles up across boundaries. Present visible rendering evidence
does not make grading present, and present Save scenario evidence does not make
first-lesson completion present.

## Human output contract

Plain output is written for reviewers who need to decide what is blocked. The
sequence command prints `ready` or `not ready`; known blockers stay visible in
the blocker lines and in JSON `status: "blocked"` when the structured report can
distinguish a coherent blocker from missing or invalid evidence.

Example with missing evidence:

```text
First-lesson automation scenario readiness: not ready

Evidence present:
- Alice launch scenario evidence is present for original Alice.
- Alice launch scenario evidence is present for RabbitHole.
- Visible rendering scenario evidence is present for RabbitHole.

Blockers:
- Select Project scenario evidence is missing for original Alice.
- Procedure/edit scenario evidence is missing for original Alice.
- Save scenario evidence is missing for original Alice.
- Grading scenario evidence is missing.
- Creative assessment scenario evidence is missing.
- First-lesson completion scenario evidence is missing.

This report does not prove full Alice UI automation, visible rendering
correctness, bounded Save completion, grading, creative assessment, or
first-lesson completion.
```

Example with a known blocker:

```text
First-lesson automation scenario readiness: not ready

Evidence present:
- Select Project scenario evidence is present.
- Procedure/edit scenario evidence is present.
- Visible rendering scenario evidence is present.

Blockers:
- Procedure/edit scenario evidence is blocked for original Alice: deterministic
  procedure edit evidence is not available in this run.
- Save scenario evidence is blocked for RabbitHole: bounded Save evidence was not
  produced by this run.
- First-lesson completion scenario evidence is missing.

This report does not prove bounded Save completion or first-lesson completion.
```

Human output may include short evidence-root-relative summaries. It must not
expose absolute paths, raw artifact contents, screenshots, logs, environment
variables, secrets, framework-internal names, raw blocker objects, or internal
next-action artifact paths. When the next missing proof is the first-lesson
next-action artifact, human output and progress details say `desktop
next-action evidence`.

## JSON API

The readiness schema is `eatme.alice-lesson-session-readiness/v1`. Existing
fields remain stable. Boundary reporting appears in `evidence_boundaries` and
the legacy `evidence_progress.items[]` entries remain available for older
consumers.

Top-level fields:

| Field | Type | Description |
| --- | --- | --- |
| `schema_version` | string | Readiness report schema. |
| `manifest_path` | string | Comparison manifest inspected by the runner. |
| `scenario_id` | string or null | Scenario being checked. |
| `passed` | boolean | Structural evidence check result. A blocked report can still have `passed: true` when the remaining blocker is explicit. |
| `status` | string | `ready`, `not_ready`, or `blocked`. |
| `readiness_status` | string | Backward-compatible detailed status. |
| `human_summary` | string | Plain scenario-focused summary. |
| `evidence_progress` | object | Backward-compatible progress counts and project proof-artifact entries. |
| `evidence_boundaries` | array | Boundary-specific evidence states for first-lesson readiness. |
| `target_evidence` | array | Per-target original Alice and RabbitHole launch/action evidence. |
| `role_readiness` | array | Role-specific readiness envelopes. |
| `lesson_session_readiness` | object | Backward-compatible student readiness envelope. |
| `issues` | array of strings | Blocking structural problems. |
| `limitations` | array of strings | Non-claims that remain true even when evidence is present. |

### Structured blocker shape

When a first-lesson report exposes a structured blocker, it uses the same field
names everywhere:

| Field | Type | Description |
| --- | --- | --- |
| `code` | string | Stable machine-readable blocker category. |
| `action` | string or null | Scenario action or boundary affected by the blocker, such as `save_project` or `procedure_edit`. |
| `reason` | string | Stable reason phrase suitable for logs and CI. |
| `message` | string | Safe human-readable message. It must not contain absolute paths, raw artifact contents, screenshots, logs, environment variables, secrets, or raw framework internals. |

### `evidence_boundaries[]`

Each boundary entry has this shape:

| Field | Type | Description |
| --- | --- | --- |
| `id` | string | Stable boundary id: `select_project`, `procedure_edit`, `save_project`, `visible_rendering`, `grading`, `creative_assessment`, or `first_lesson_completion`. |
| `label` | string | Human-readable scenario evidence label. |
| `status` | string | `present`, `missing`, `invalid`, `not_observed`, or `blocked`. |
| `source` | string or null | Short source category, such as `ui_action_contract`, `rabbithole`, `comparison_manifest`, or `scenario_asset`. |
| `metadata_state` | string or null | Optional boundary metadata state, such as `declared` or `observed`. Metadata state never upgrades the boundary to completion evidence. |
| `detail` | string | Plain boundary summary safe for JSON consumers and CLI summaries. Structured blockers use `message` in the blocker shape. |
| `claim` | string | The exact bounded claim this boundary supports when `status` is `present`. |
| `does_not_prove` | array of strings | Claims that remain unsupported by this boundary. |
| `artifact` | object or omitted | Safe artifact metadata when the boundary has accepted evidence rooted under the comparison evidence directory. |

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
      "source": "ui_action_contract",
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
      "detail": "Save scenario metadata was declared, but bounded Save evidence is missing.",
      "claim": "No Save completion claim is supported.",
      "does_not_prove": [
        "bounded Save completion",
        "first-lesson completion",
        "learner-world grading"
      ]
    }
  ]
}
```

## Configuration

| Setting | Required for | Description |
| --- | --- | --- |
| `NODE_OPTIONS=--max-old-space-size=32768` | Agentic/Gadugi-heavy local runs | Keeps Node-based runners from failing under large prompt or adapter workloads. |
| `EATME_REAL_ALICE=1` | Non-baseline real Alice execution | Explicit opt-in gate for desktop execution. |
| `ALICE_BASELINE_HOME` | `alice run-first-lesson-readiness --execute` | Original Alice checkout. |
| `ALICE_MODERNIZED_HOME` | `alice run-first-lesson-readiness --execute` | RabbitHole Alice checkout. |

Real desktop evidence also requires the Alice dependency set documented in
[Alice Integration](alice-integration.md): Java 21, Maven, Xvfb, `xdpyinfo`,
`wmctrl`, `xwininfo`, `xdotool`, screenshot tooling, and software OpenGL
support.

## Tutorials

### Evaluate current blockers

1. Run the readiness command with `--json`.
2. Read the top-level `status`.
3. Inspect every `evidence_boundaries[]` entry.
4. Treat every `missing`, `invalid`, `not_observed`, or `blocked` boundary as a
   blocker.
5. Do not infer completion from counts, artifact presence, screenshot presence,
   Save dispatch, action ids, or boundary declarations.

### Review a present boundary safely

When a boundary reports `present`, use the `claim` field as the complete claim.
Then read `does_not_prove` before writing a PR, issue, classroom handoff, or
release note.

For example, a present `visible_rendering` boundary can support:

```text
Visible rendering scenario evidence is present for RabbitHole.
```

It cannot support:

```text
Rendering is correct.
The animation is visually correct.
The first lesson is complete.
```

### Repair a missing original Alice action boundary

If target-local original Alice action evidence is missing, repair the relevant
`target_evidence[]` entry or structured blocker `action`, then rerun readiness.
Do not treat any of these as evidence for the action:

- a comparison manifest without execution;
- a launch manifest without a readable `ui-action-contract.json`;
- a required action id listed in metadata but missing from the action contract;
- a required action entry without executable evidence or an explicit blocker;
- an artifact path outside the comparison evidence root;
- a screenshot that does not explicitly describe the observed boundary.

### Repair a missing Save boundary

If Save scenario evidence is `missing`, collect or repair bounded Save evidence
and rerun readiness. Do not treat any of these as bounded Save completion:

- a Save command dispatch without completion evidence;
- a boundary declaration without an observed completion signal;
- saved-project artifact availability without a completion signal;
- a saved-project path outside the evidence root;
- a screenshot that does not explicitly prove Save completion;
- grading or completion evidence from another boundary.

## Writing readiness-related docs and PRs

Use scenario-focused wording in user-facing text:

| Say | Avoid in primary human output |
| --- | --- |
| `First-lesson automation scenarios are not ready.` | `ui-action-contract no_go status failed.` |
| `Select Project scenario evidence is missing for original Alice.` | `select_project_proof_artifact declaration is missing.` |
| `Save scenario evidence is blocked for RabbitHole.` | `save_project_desktop_shortcut_dispatch failed.` |
| `desktop next-action evidence is missing.` | Internal next-action evidence paths. |
| `Visible rendering scenario evidence is present, but correctness is not proven.` | `pixel proof passed, so rendering is correct.` |

It is acceptable for JSON reference sections to document stable field names.
Primary human output should stay plain, scenario-focused, and conservative.
