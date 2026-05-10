# First-lesson evidence readiness

This document describes the first-lesson readiness report built for original
Alice and RabbitHole. The report says what the first-lesson automation scenarios
have shown, what is not yet shown, and which claims remain explicitly unproven.
It consumes existing comparison, launch, desktop, and editable scenario evidence;
it does not generate new proof.

The same CLI readiness entry point also has a separate exact
`real-alice-launch-smoke` branch for bounded baseline launch-smoke readiness.
That branch is documented in
[Real Alice Launch-Smoke Readiness](real-alice-launch-smoke-readiness.md) and
does not use the first-lesson evidence boundaries below.

The Rust API and JSON output preserve legacy fields such as
`evidence_progress`, `evidence_boundaries`, `issues`, and `limitations` for
existing consumers while adding the user-facing report shape described here. The
plain CLI renders the user-facing sections directly.

The report is intentionally conservative. A launch, action declaration,
Save shortcut, artifact path, screenshot, or desktop observation can support only
the bounded claim named in the report. It never implies Full Alice UI automation,
grading, creative assessment, visible rendering correctness, Save completion, or
first-lesson completion unless explicit evidence for that exact claim exists. It
also does not imply full world execution or deployed sharing/platform success.
The artifact shape and wording rules for preserving this boundary are documented
in [Evidence Artifact Contract](evidence-artifact-contract.md).

When required evidence is missing, invalid, incomplete, or insufficient, and
the report therefore cannot confirm readiness, the report also prints one plain
evidence-gap line:

```text
Evidence gap: Missing evidence means this report cannot confirm first-lesson readiness, lesson completion, grading, or creative assessment.
```

That line is a gap notice, not a grade, score, certification, creative review, or
lesson-completion claim. A coherent `status: "blocked"` report that only contains
explicit known blockers does not need this line; if required evidence is also
missing, invalid, incomplete, or insufficient, the report stays `not_ready` and
includes the gap notice.

## Quick start

Check an existing first-lesson comparison manifest:

```bash
cargo run -q -p eatme-cli -- alice check-lesson-readiness \
  --manifest runs/comparisons/first-lessons-real-ui-actions/local/comparison-manifest.json
```

Run the bounded first-lesson comparison and readiness sequence:

```bash
export NODE_OPTIONS=--max-old-space-size=32768
export ALICE_BASELINE_HOME=/path/to/original-alice
export ALICE_MODERNIZED_HOME=/path/to/rabbithole-alice

EATME_REAL_ALICE=1 cargo run -q -p eatme-cli -- alice run-first-lesson-readiness \
  --run-id local-first-lesson-readiness \
  --no-memory \
  --offline-package \
  --execute
```

The sequence fixes the scenario to `first-lessons-real-ui-actions`, writes the
current comparison manifest under:

```text
runs/comparisons/first-lessons-real-ui-actions/<run-id>/comparison-manifest.json
```

and immediately applies the same readiness report to that manifest.

Use `--json` when a consumer needs the structured API instead of the plain human
report. The same commands remain the entry points for plain and structured
readiness reporting.

## What the report decides

The report answers one bounded question:

> What first-lesson readiness evidence has the selected comparison run shown,
> what is not yet shown, and what must remain unproven?

It does not answer whether a learner completed the lesson, whether an Alice
world is creatively successful, whether a saved project should receive a grade,
whether rendering is correct, or whether the entire Alice UI flow is automated.

When creative-assessment evidence is missing, limited, or unavailable, the
report can surface available evidence and suggest bounded next steps for the
learner's creative work in this scenario. It does not grade creativity, judge
quality, or mark the lesson complete.

| Result | Meaning | What to do |
| --- | --- | --- |
| `ready` | Required bounded evidence is present and valid, target and launch-manifest `failure_category` values are `null`, and unsupported-action no-go entries are absent. | Use only the named shown evidence. Keep the unproven claims attached. |
| `not_ready` | Required evidence is missing, malformed, unsafe, incomplete, manifest-only, out of order, or not observed. | Read `Not yet shown` and collect or repair that evidence. |
| `blocked` | Evidence was read, but RabbitHole or original Alice reported an explicit unsupported-action or next-action reason. | Preserve the reason as "not yet shown"; do not turn it into success. |

## Evidence consumption model

Readiness consumes the newest evidence for the selected first-lesson run
from the same surfaces already used by the readiness system:

| Evidence source | How it is consumed | What it can show |
| --- | --- | --- |
| Editable scenario asset | `assets/scenarios/eatme/first-lessons-real-ui-actions.yaml` remains the editable source for boundary expectations and non-claims. | The scenario owns the first-lesson evidence contract. |
| Comparison manifest | The manifest selected by `--manifest`, or the manifest just written by `alice run-first-lesson-readiness`. | The run, scenario id, baseline target, modernized/RabbitHole target, execution state, and embedded launch evidence. |
| Target launch/action evidence | Target-local launch manifests and `ui-action-contract.json`, resolved under the comparison evidence root. | Launch/action observations for original Alice and RabbitHole. |
| RabbitHole desktop evidence | Modernized target desktop evidence, including Run-window, desktop execution, visible screenshot, and project proof-artifact states. | RabbitHole observations for the next bounded first-lesson action. |
| Desktop next-action evidence | `desktop-first-lesson-next-action.json`, only when present, valid, safely rooted, and applicable to the current RabbitHole run. | Candidate next actions, explicit blockers, and proof-artifact availability. |

The report fails closed for unsafe or untrusted input. Absolute paths, parent
traversal, symlink escapes, unreadable files, malformed JSON, wrong schema
versions, empty artifacts, and artifact references outside the comparison
evidence root are not shown as evidence.

Generated Gadugi adapters remain generated artifacts. Change scenario intent in
the editable YAML under `assets/scenarios/eatme/`, then regenerate adapters
rather than hand-editing generated files.

## Plain report contract

Plain output from `alice run-first-lesson-readiness` without `--json` is for
reviewers, instructors, and PR readers. It renders the readiness heading, one
`Desktop proof` line, and then the user-facing sections in this order:

| Section | When it appears | Meaning |
| --- | --- | --- |
| `Desktop proof` | Always, as a single line after the readiness heading. | Machine-readable desktop proof status rendered for humans. It is not a completion claim. |
| `Shown` | One or more bounded evidence facts are present. | Evidence was read and is safe to summarize for the named claim only. |
| `Not yet shown` | Any required evidence is missing, invalid, not observed, or blocked. | The claim is not yet shown or not yet proven in user-facing wording. |
| `Desktop next action` | RabbitHole desktop next-action evidence exists, is valid, and applies to the current run. | RabbitHole reported observations, candidate next actions, or explicit next-action reasons. |
| `Original Alice action evidence` | `original_alice_action_evidence.status` is `missing`. | Explicitly reports `Original Alice action evidence is missing.` It is reportable state, not a completion claim. |
| `Unproven` | Always. | The eight required non-claims that the report must not imply. |

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
does not make grading present, and present Save option/action evidence does not
make first-lesson completion present.

## Human output contract

Plain output is written for reviewers who need to decide what is blocked. The
sequence command prints a first-lesson/grading gap-report heading; known blockers
stay visible in JSON `status: "blocked"` and user-facing gap sections when the
structured report can distinguish a coherent blocker from missing or invalid
evidence.

Illustrative plain-output excerpt with missing evidence:

```text
First-lesson/grading gap report: not ready
Gap report scope: missing/incomplete evidence, unsupported claims, and next actions only.
Evidence gap: Missing evidence means this report cannot confirm first-lesson readiness, lesson completion, grading, or creative assessment.
Desktop proof: launched_but_unverified (desktop_run_window_unverified) - desktop Run window dispatch lacks modernized-target proof

Shown:
- RabbitHole launch/action evidence is shown.
- RabbitHole Run-window observation is shown.
- Save option/action evidence is shown as observed option/action only.

Not yet shown:
- Save completion is not yet proven.
- Visible rendering correctness is not yet proven.
- Grading is not yet shown.
- Creative assessment is not yet shown. Available evidence does not yet show
  creative assessment or shows that creative-assessment evidence is unavailable;
  the report can surface available evidence and suggest bounded next steps for
  the learner's creative work in this scenario, but it does not grade
  creativity, judge quality, or mark the lesson complete.
- First-lesson completion is not yet shown.

Desktop next action:
- Desktop next-action evidence was shown as an observation only.
- Save option/action evidence is present as an observation only.
- Next evidence needed: Collect explicit Save completion evidence before reporting Save completion.

Original Alice action evidence:
- Original Alice action evidence is missing.
- Original Alice action evidence was not found in the comparison target evidence.

Unproven:
- Full Alice UI automation is not proven.
- Full world execution is not proven.
- Grading is not proven.
- Creative assessment is not proven.
- Visible rendering correctness is not proven.
- Save completion is not proven.
- Deployed sharing/platform success is not proven.
- First-lesson completion is not proven.
```

If RabbitHole desktop next-action evidence is absent, invalid, unsafe, stale, or
not applicable, the top-level `Desktop next action` section is omitted. That
omission is not silent: the missing or invalid condition must still appear in
`Not yet shown`, `issues`, legacy progress fields, or the relevant boundary item
when that evidence is required for the current claim.

If original Alice action evidence is missing, the plain report always includes
the `Original Alice action evidence` section before `Unproven` with this fixed
summary:

```text
Original Alice action evidence is missing.
```

Missing original Alice action evidence is not fatal by itself and does not
change readiness or exit behavior on its own. It also does not turn missing
evidence into a completion, assessment, grading, Save, or full-UI claim. When
original Alice action evidence is available, the section is omitted and the
structured JSON field remains available for automation consumers.

For a pure known-blocker report, do not expect the evidence-gap line; use JSON
`status: "blocked"` and structured blocker entries for exact automation state.

Human output may include short evidence-root-relative summaries. It must not
expose absolute paths, raw artifact contents, screenshots, logs, environment
variables, secrets, raw blocker objects, framework-internal names, or internal
next-action artifact paths. Use `desktop next-action evidence` as the display
label instead of artifact filenames.

## Evidence boundaries

First-lesson readiness keeps each scenario claim independent. A present boundary
does not make another boundary present.

| Boundary id | Human label | Evidence required to show it | Must not imply |
| --- | --- | --- | --- |
| `select_project` | Select Project scenario evidence | Explicit evidence that the Select Project boundary produced a safe, auditable scenario signal. | Full Alice UI automation, project-selection success beyond the named boundary, or first-lesson completion. |
| `procedure_edit` | Procedure/edit scenario evidence | Explicit evidence that a procedure or code edit boundary was completed or observed with a safe summary. | Code correctness, learner understanding, grading, or completed lesson work. |
| `save_project` | Save option/action scenario evidence | Explicit bounded evidence that a Save affordance, action, declaration, or proof artifact was observed. This is not a completion signal. | Save completion, grading, creative assessment, or first-lesson completion. |
| `visible_rendering` | Visible rendering scenario evidence | Explicit visible rendering observation from the run boundary. | Visible rendering correctness, animation correctness, creative quality, or complete visual validation. |
| `grading` | Grading scenario evidence | Explicit grading evidence from a scenario that owns grading. | Any automatic grade when no grading evidence exists. |
| `creative_assessment` | Creative assessment scenario evidence | Explicit creative assessment evidence from a scenario that owns creative review. When evidence is missing, limited, or unavailable, the report can surface available evidence and suggest bounded next steps for the learner's creative work in this scenario. | Creativity grading, quality judgment, learner-world grading, instructor judgment, or marked lesson completion. |
| `first_lesson_completion` | First-lesson completion scenario evidence | Explicit first-lesson completion evidence from the completion boundary. | Completed first lesson from launch, Save, rendering, grading, substep evidence, full world execution, deployed sharing, or platform success alone. |

### User-facing state wording

Structured JSON keeps machine states, but plain output maps them to user-facing
language. These are readiness output states; artifact input status values are
defined separately in the
[Evidence Artifact Contract](evidence-artifact-contract.md).

| JSON state | Plain wording | Meaning |
| --- | --- | --- |
| `present` | `shown` | The named bounded evidence is present and safe to summarize. |
| `missing` | `not yet shown` | The required evidence is absent or incomplete. |
| `invalid` | `not yet shown` | The evidence exists but cannot be trusted or safely summarized. |
| `not_observed` | `not yet shown` | A producer ran, but the expected observation was not made. |
| `blocked` | `not yet shown` or `not yet proven` with the supplied reason | RabbitHole or original Alice supplied an explicit reason the claim cannot yet be shown. |

Legacy boundary artifact inputs may use `declared` or `observed` to describe
metadata availability. Those values are not readiness output states; they
normalize to output `missing` unless distinct boundary evidence is present, with
the metadata state preserved for diagnostics.

Primary human output avoids internal terms such as `no_go`,
`ui-action-contract`, `desktop-run-pixel`, and raw artifact paths. JSON reference
sections may document those stable field names for automation consumers.

## Canonical unproven claims

`unproven_claims` is the canonical home for the eight non-claims that must remain
visible in plain output and JSON:

```text
Full Alice UI automation is not proven.
Full world execution is not proven.
Grading is not proven.
Creative assessment is not proven.
Visible rendering correctness is not proven.
Save completion is not proven.
Deployed sharing/platform success is not proven.
First-lesson completion is not proven.
```

Legacy `limitations` remains for compatibility. It may be a broader or superset
list for older consumers, but it must include these eight claims exactly enough for
automation to preserve them. New consumers should read `unproven_claims` first.
The canonical non-claims are produced by readiness output even when a
next-action artifact omits its optional `does_not_claim`/`doesNotClaim` input.
If that input is present, it is validated and merged into the desktop
next-action non-claims instead of replacing the canonical list.

Save wording has one extra rule: Save action, Save option, Save shortcut, and
Save proof-artifact availability may be shown, but Save completion remains
unproven unless a distinct explicit Save-completion evidence item exists.

## JSON API

The readiness schema emitted by `alice check-lesson-readiness` is
`eatme.alice-lesson-session-readiness/v1`. Existing fields remain available for
older consumers. The user-facing report shape adds shown evidence, missing
evidence, optional desktop next-action evidence, and canonical unproven claims.
The `alice run-first-lesson-readiness` sequence wraps the same readiness result
in `eatme.first-lesson-readiness-sequence/v1`, using
`comparison_manifest_path` for the manifest it wrote and `readiness_report` for
the nested readiness report. The sequence also exposes
`original_alice_action_evidence` at the sequence top level, copied from
`readiness_report.original_alice_action_evidence`, so callers do not have to
open the nested readiness payload to preserve the missing-evidence state.

Top-level fields:

| Field | Type | Description |
| --- | --- | --- |
| `schema_version` | string | Readiness report schema. |
| `manifest_path` | string | Comparison manifest inspected by the runner. |
| `scenario_id` | string or null | Scenario being checked. |
| `passed` | boolean | `true` only when the normalized readiness `status` is `ready`. |
| `status` | string | `ready`, `not_ready`, or `blocked`. |
| `readiness_status` | string | Backward-compatible detailed status. |
| `blocked_reason` | string or null | Machine-readable blocker reason when `status` is `blocked`. |
| `human_summary` | string | Plain scenario-focused summary. |
| `evidence_gap_message` | string or null | Plain user-facing gap notice when required first-lesson evidence is missing, invalid, incomplete, or insufficient. Pure known blockers use `status: "blocked"` and structured blocker fields; include this line only when the blocker also leaves required evidence unconfirmed. The value is `Evidence gap: Missing evidence means this report cannot confirm first-lesson readiness, lesson completion, grading, or creative assessment.` It is `null` for `ready`. |
| `desktop_proof_contract` | object | Modernized desktop proof status rendered as the plain `Desktop proof` line. |
| `original_alice_action_evidence` | object | Structured original Alice action evidence state. Reports `missing` when target evidence contains a blocker with `code: "missing_real_action_evidence"`; otherwise reports `available`. |
| `shown_evidence` | array | User-facing facts that were shown by accepted evidence. |
| `not_yet_shown` | array | User-facing missing, invalid, not-observed, or blocked claims. |
| `desktop_next_action` | object or omitted | RabbitHole desktop next-action summary, emitted only when valid, safe, current, and applicable. |
| `unproven_claims` | array | Canonical non-claims that always remain visible. |
| `evidence_boundaries` | array | Boundary-specific evidence states. |
| `evidence_progress` | object | Backward-compatible progress counts and project proof-artifact entries. |
| `required_evidence` | array of strings | Durable evidence names required by the readiness check. |
| `no_go_contracts` | array | Aggregated unsupported-action entries from target evidence. |
| `target_evidence` | array | Per-target original Alice and RabbitHole launch/action evidence. |
| `role_readiness` | array | Role-specific readiness envelopes. |
| `lesson_session_readiness` | object | Backward-compatible student readiness envelope. |
| `contract_check` | object | Result from `alice check-lesson-session`. |
| `execute_requested` | boolean or null | Whether the comparison manifest was produced with execution enabled. |
| `issues` | array of strings | Blocking structural problems for automation and debug consumers. |
| `limitations` | array of strings | Backward-compatible non-claims. May remain a legacy/superset list, but must include the eight canonical `unproven_claims`. |

### `original_alice_action_evidence`

`original_alice_action_evidence` is an additive top-level readiness field. It
keeps the original Alice action evidence state visible even when the same
condition also appears inside `target_evidence[]` blockers.

The field is derived only from existing target evidence blockers. If any
`target_evidence[].blockers[]` entry has `code: "missing_real_action_evidence"`,
the report emits `status: "missing"`. If no such blocker exists, it emits
`status: "available"`. Unknown blocker codes do not affect this field.
`available` only means no `missing_real_action_evidence` blocker was found; it
does not prove full UI automation, Save completion, lesson completion, grading,
or creative assessment.

| Field | Type | Description |
| --- | --- | --- |
| `status` | string | `available` or `missing`. |
| `summary` | string | Fixed plain summary: `Original Alice action evidence is missing.` or `Original Alice action evidence is available.` |
| `detail` | string | Fixed bounded detail for the selected status. |

Missing example:

```json
{
  "original_alice_action_evidence": {
    "status": "missing",
    "summary": "Original Alice action evidence is missing.",
    "detail": "Original Alice action evidence was not found in the comparison target evidence."
  }
}
```

Available example:

```json
{
  "original_alice_action_evidence": {
    "status": "available",
    "summary": "Original Alice action evidence is available.",
    "detail": "The readiness report did not find a missing original Alice action evidence blocker."
  }
}
```

This field does not replace `target_evidence[]`. Consumers that need per-target
diagnostics should still read `target_evidence[]`, including its blockers and
action assertions. Missing original Alice action evidence is not fatal by itself;
it reports state without changing exit/readiness behavior on its own. The new
field is the stable summary for readiness dashboards and the first-lesson
readiness sequence wrapper, including that wrapper's plain CLI output.

First-lesson sequence example:

```json
{
  "schema_version": "eatme.first-lesson-readiness-sequence/v1",
  "comparison_manifest_path": "runs/comparisons/first-lessons-real-ui-actions/local-first-lesson-readiness/comparison-manifest.json",
  "original_alice_action_evidence": {
    "status": "missing",
    "summary": "Original Alice action evidence is missing.",
    "detail": "Original Alice action evidence was not found in the comparison target evidence."
  },
  "readiness_report": {
    "original_alice_action_evidence": {
      "status": "missing",
      "summary": "Original Alice action evidence is missing.",
      "detail": "Original Alice action evidence was not found in the comparison target evidence."
    }
  }
}
```

The `alice run-first-lesson-readiness --json` sequence report uses schema
`eatme.first-lesson-readiness-sequence/v1`. It exposes the same
`evidence_gap_message` at the sequence top level and again under
`readiness_report.evidence_gap_message`; both values must match. Consumers that
only care about the sequence result can read the top-level field. Consumers that
need the embedded readiness report can read the nested field.

### User-facing evidence item

`shown_evidence[]` and `not_yet_shown[]` use the same item shape:

| Field | Type | Description |
| --- | --- | --- |
| `id` | string | Stable evidence or boundary id. |
| `state` | string | Machine state: `present`, `missing`, `invalid`, `not_observed`, or `blocked`. |
| `summary` | string | User-facing sentence safe for CLI output. |
| `detail` | string | Display-safe detail for automation and debugging. |
| `does_not_prove` | array of strings, omitted when empty | Claims still unsupported by this evidence item. |

Example:

```json
{
  "id": "creative_assessment",
  "state": "missing",
  "summary": "Creative assessment is not yet shown.",
  "detail": "Available evidence does not yet show creative assessment or shows that creative-assessment evidence is unavailable; the report can surface available evidence and suggest bounded next steps for the learner's creative work in this scenario. The report does not grade creativity, judge quality, or mark the lesson complete.",
  "does_not_prove": [
    "creative assessment",
    "creativity grading",
    "quality judgment",
    "first-lesson completion"
  ]
}
```

### `desktop_next_action`

The `desktop_next_action` object is conditional. It is omitted when the
artifact is absent, invalid, unsafe, stale, or not applicable to the current
RabbitHole target. When present, it summarizes observations without promoting
them to completion claims. When omitted, the reason must still be represented in
`not_yet_shown`, `issues`, `evidence_progress`, or `evidence_boundaries[]` as
appropriate for the failure mode.

| Field | Type | Description |
| --- | --- | --- |
| `status` | string | RabbitHole next-action state, such as `present` or `blocked`. |
| `summary` | string | Safe user-facing summary. |
| `candidate_actions` | array of strings | Candidate next actions reported by RabbitHole; empty when the optional artifact input is absent. |
| `requires_next_evidence` | array of strings | Evidence RabbitHole says must be collected next; empty when the optional artifact input is absent. |
| `observations` | array of strings | Plain observations from the next-action evidence. |
| `does_not_prove` | array of strings | Canonical non-claims plus any validated optional `does_not_claim`/`doesNotClaim` input values preserved for the desktop next-action section. |

Example excerpt:

```json
{
  "desktop_next_action": {
    "status": "present",
    "summary": "Desktop next-action evidence was shown as an observation only.",
    "candidate_actions": ["save-project"],
    "requires_next_evidence": [
      "Collect explicit Save completion evidence before reporting Save completion."
    ],
    "observations": [
      "Desktop next-action evidence was shown with status present.",
      "Save option/action evidence is present as an observation only.",
      "Select Project option/action evidence is missing as an observation only."
    ],
    "does_not_prove": [
      "full Alice UI automation",
      "Save completion",
      "first-lesson completion"
    ]
  }
}
```

### `evidence_boundaries[]`

Boundary entries remain available for consumers that need the scenario contract.

| Field | Type | Description |
| --- | --- | --- |
| `id` | string | Stable boundary id. |
| `label` | string | Human-readable scenario evidence label. |
| `status` | string | `present`, `missing`, `invalid`, `not_observed`, or `blocked`. |
| `source` | string or null | Short source category. |
| `metadata_state` | string or null | Optional boundary metadata state, such as `declared` or `observed`. Metadata state never upgrades the boundary to completion evidence. |
| `detail` | string | Display-safe boundary summary. |
| `claim` | string | Exact bounded claim supported when `status` is `present`; otherwise a statement that the claim is not proven. |
| `does_not_prove` | array of strings | Claims that remain unsupported by this boundary. |
| `artifact` | object or omitted | Safe artifact metadata rooted under the comparison evidence directory. |

## Configuration

| Setting | Required for | Description |
| --- | --- | --- |
| `NODE_OPTIONS=--max-old-space-size=32768` | Agentic/Gadugi-heavy local runs | Recommended Node heap setting for agentic/Gadugi-heavy local runs. |
| `EATME_REAL_ALICE=1` | Non-baseline real Alice execution | Explicit opt-in gate for desktop execution. |
| `ALICE_BASELINE_HOME` | `alice run-first-lesson-readiness --execute` | Original Alice checkout. |
| `ALICE_MODERNIZED_HOME` | `alice run-first-lesson-readiness --execute` | RabbitHole Alice checkout. |

Real desktop evidence also requires the Alice dependency set documented in
[Alice Integration](alice-integration.md): Java 21, Maven, Xvfb, `xdpyinfo`,
`wmctrl`, `xwininfo`, `xdotool`, screenshot tooling, and software OpenGL
support.

### Wording ownership

The first-lesson readiness report owns the fixed evidence-gap sentence. Scenario
assets continue to own editable labels, evidence boundaries, unsupported-action
policy, and classroom-facing wording; the gap line is not configurable and is not
a grading or completion setting.

Do not add workflow timeout settings for first-lesson readiness reporting. The
report consumes existing evidence for the selected run; it does not introduce
new proof-generation timing policy.

No setting enables automated creative grading, quality judgment, or lesson
completion marking. Creative-assessment gap wording is part of the first-lesson
readiness report contract.

## Tutorials

### Review RabbitHole first-lesson readiness after implementation

1. Run `alice check-lesson-readiness` against the current comparison manifest.
2. Read `Shown` first. Treat each line as a bounded evidence fact only.
3. Read every `Not yet shown` line before deciding what to collect next.
4. If `evidence_gap_message` is a string, treat it as the report's plain warning
   that missing, invalid, incomplete, or insufficient evidence prevents
   confirmation.
5. Inspect every `evidence_boundaries[]` entry and treat every `missing`,
   `invalid`, `not_observed`, or `blocked` boundary as a
   blocker.
6. Do not infer completion from counts, artifact presence, screenshot presence,
   Save dispatch, action ids, or boundary declarations.
7. Read `Desktop next action` when it appears. It describes RabbitHole's next
   observations or candidate actions, not completion.
8. Keep every `Unproven` line in handoffs, PRs, and release notes.

### Interpret Save evidence safely

Save-related evidence can show that a Save option, Save action, Save shortcut, or
Save artifact availability was observed. It can support a Save-completion claim
only when a distinct explicit Save-completion evidence item exists.

Safe wording:

```text
Save option/action evidence is shown as observed option/action only.
Save completion is not yet proven.
```

Unsafe wording treats a Save affordance or first-lesson boundary as completion.
Keep those outcomes phrased as not yet proven until distinct completion evidence
exists.

### Review a creative-assessment gap safely

When the `creative_assessment` boundary reports `missing`, `invalid`,
`not_observed`, or `blocked`, treat the entry as a gap report. Use its `detail`
text to find available evidence and next steps for the learner's creative work
in this scenario, then collect or repair the scenario evidence that a human
reviewer needs.

Do not translate a creative-assessment gap into grading, creative-quality, or
first-lesson completion claims.

Or equivalently:

```text
The learner's world was graded.
The creative work is good or bad.
The learner finished the first lesson.
```

### Keep evidence assets editable

When wording or expected evidence changes, edit the canonical scenario asset:

```bash
$EDITOR assets/scenarios/eatme/first-lessons-real-ui-actions.yaml
cargo run -q -p eatme-cli -- assets validate \
  --path assets/scenarios/eatme/first-lessons-real-ui-actions.yaml \
  --json
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
```

If generated adapters are stale, regenerate them from the canonical assets:

```bash
cargo run -q -p eatme-cli -- assets generate-gadugi --json
```

Do not encode readiness facts only in generated adapters, binary artifacts, or
one-off run outputs.

## Writing readiness-related docs and PRs

Use user-facing wording:

| Say | Avoid |
| --- | --- |
| `RabbitHole launch/action evidence is shown.` | Internal contract status shorthand as a user-facing success claim. |
| `Desktop next-action evidence is not yet shown.` | Internal next-action artifact paths. |
| `Save option evidence is shown as an observed option/action only.` | Any wording that treats observed Save affordance evidence as Save completion. |
| `Visible rendering evidence is shown, but correctness is not proven.` | Any wording that treats screenshot or pixel evidence as rendering correctness. |
| `First-lesson completion is not yet shown.` | Any wording that treats partial evidence as first-lesson completion. |
| `Save option/action evidence is shown as observed option/action only.` | `Save completed.` |
| `Visible rendering evidence is shown, but correctness is not proven.` | `Rendering is correct.` |
| `First-lesson completion is not yet shown.` | `The lesson is complete.` |

The durable rule is simple: report what the evidence explicitly shows, report
missing states as not yet shown or not yet proven, and keep the eight unproven
claims visible.

## Implementation contract

The Rust implementation:

1. Emits `shown_evidence[]`, `not_yet_shown[]`, optional
   `desktop_next_action`, `unproven_claims`,
   `original_alice_action_evidence`, and boundary-facing evidence items.
2. Maps existing progress and boundary states to user-facing `shown`, `not yet
   shown`, and `not yet proven` wording without exposing internal artifact paths
   in plain output.
3. Emits top-level `desktop_next_action` only for valid, safe, current RabbitHole
   evidence; otherwise it leaves the condition in `not_yet_shown`, `issues`, or
   legacy progress/boundary fields.
4. Preserves legacy JSON fields including `evidence_progress`, `target_evidence`,
   `lesson_session_readiness`, `role_readiness`, `issues`, and `limitations`.
5. Makes `unproven_claims` the canonical eight non-claims and keeps
   `limitations` as compatibility output that includes those eight.
6. Renders `alice run-first-lesson-readiness` plain output as readiness heading,
   `Desktop proof`, `Shown`, `Not yet shown`, optional `Desktop next action`,
   optional `Original Alice action evidence`, and `Unproven`. The original Alice
   section appears only when original Alice action evidence is missing.
7. Keeps Save action/artifact evidence separate from Save completion unless an
   explicit Save-completion evidence item exists.
