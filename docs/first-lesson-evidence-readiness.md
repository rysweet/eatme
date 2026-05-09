# First-lesson evidence readiness

This document describes the first-lesson readiness report built for original
Alice and RabbitHole. The report says what the first-lesson automation scenarios
have shown, what is not yet shown, and which claims remain explicitly unproven.
It consumes existing comparison, launch, desktop, and editable scenario evidence;
it does not generate new proof.

The Rust API and JSON output preserve legacy fields such as
`evidence_progress`, `evidence_boundaries`, `issues`, and `limitations` for
existing consumers while adding the user-facing report shape described here. The
`alice run-first-lesson-readiness` command renders those user-facing sections
directly when it is run without `--json`.

The report is intentionally conservative. A launch, action declaration,
Save shortcut, artifact path, screenshot, or desktop observation can support only
the bounded claim named in the report. It never implies full UI automation,
grading, creative assessment, visible rendering correctness, Save completion, or
first-lesson completion unless explicit evidence for that exact claim exists.

## Quick start

Check an existing first-lesson comparison manifest:

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

Use `alice check-lesson-readiness --json` for the structured readiness API. Use
`alice run-first-lesson-readiness` without `--json` when a reviewer needs the
plain human report; add `--json` to that sequence command when a consumer needs
the structured wrapper around the same readiness result.

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
| `ready` | Required bounded evidence is present, valid, and free of known unsupported-action states. | Use only the named shown evidence. Keep the unproven claims attached. |
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

The formal executable validation surface is the readiness report itself.
`contract_evidence[]` represents required contract evidence; missing, invalid,
unsafe, not-observed, blocked-without-structure, or stale required entries remain
visible and produce structured `diagnostics[]`.
`alice check-lesson-readiness` emits the structured JSON report.
`alice run-first-lesson-readiness` emits either the structured wrapper with
`--json` or the plain human report without it. Both commands exit non-zero when
required contract evidence fails. A report may still exit zero with
`status: "blocked"` only when the blocker is explicit, structured, and attached
to the bounded claim that is not yet shown.

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
| `Unproven` | Always. | The six required non-claims that the report must not imply. |

Example plain report:

```text
First-lesson automation scenario readiness: not ready
Desktop proof: launched_but_unverified (desktop_run_window_unverified) - desktop Run window dispatch lacks modernized-target proof

Shown:
- Original Alice launch/action evidence is shown.
- RabbitHole launch/action evidence is shown.
- RabbitHole Run-window observation is shown.
- Save option evidence is shown as an observed option/action only.

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

Unproven:
- Full Alice UI automation is not proven.
- Grading is not proven.
- Creative assessment is not proven.
- Visible rendering correctness is not proven.
- Save completion is not proven.
- First-lesson completion is not proven.
```

If RabbitHole desktop next-action evidence is absent, invalid, unsafe, stale, or
not applicable, the top-level `Desktop next action` section is omitted. That
omission is not silent: the missing or invalid condition must still appear in
`Not yet shown`, `issues`, legacy progress fields, or the relevant boundary item
when that evidence is required for the current claim.

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
| `select_project` | Select Project scenario evidence | Explicit evidence that the Select Project boundary produced a safe, auditable scenario signal. | Full UI automation, project-selection success beyond the named boundary, or first-lesson completion. |
| `procedure_edit` | Procedure/edit scenario evidence | Explicit evidence that a procedure or code edit boundary was completed or observed with a safe summary. | Code correctness, learner understanding, grading, or completed lesson work. |
| `save_project` | Save option/action scenario evidence | Explicit bounded evidence that a Save affordance, action, declaration, or proof artifact was observed. This is not a completion signal. | Save completion, grading, creative assessment, or first-lesson completion. |
| `visible_rendering` | Visible rendering scenario evidence | Explicit visible rendering observation from the run boundary. | Visible rendering correctness, animation correctness, creative quality, or complete visual validation. |
| `grading` | Grading scenario evidence | Explicit grading evidence from a scenario that owns grading. | Any automatic grade when no grading evidence exists. |
| `creative_assessment` | Creative assessment scenario evidence | Explicit creative assessment evidence from a scenario that owns creative review. When evidence is missing, limited, or unavailable, the report can surface available evidence and suggest bounded next steps for the learner's creative work in this scenario. | Creativity grading, quality judgment, learner-world grading, instructor judgment, or marked lesson completion. |
| `first_lesson_completion` | First-lesson completion scenario evidence | Explicit first-lesson completion evidence from the completion boundary. | Completed first lesson from launch, Save, rendering, grading, or substep evidence alone. |

### User-facing state wording

Structured JSON keeps machine states, but the plain sequence output maps them
to user-facing language:

| JSON state | Plain wording | Meaning |
| --- | --- | --- |
| `present` | `shown` | The named bounded evidence is present and safe to summarize. |
| `missing` | `not yet shown` | The required evidence is absent or incomplete. |
| `invalid` | `not yet shown` | The evidence exists but cannot be trusted or safely summarized. |
| `not_observed` | `not yet shown` | A producer ran, but the expected observation was not made. |
| `blocked` | `not yet shown` or `not yet proven` with the supplied reason | RabbitHole or original Alice supplied an explicit reason the claim cannot yet be shown. |

Primary human output avoids internal terms such as `no_go`,
`ui-action-contract`, `desktop-run-pixel`, and raw artifact paths. JSON reference
sections may document those stable field names for automation consumers.

## Canonical unproven claims

`unproven_claims` is the canonical home for the six non-claims that must remain
visible in JSON and in the plain sequence report:

```text
Full Alice UI automation is not proven.
Grading is not proven.
Creative assessment is not proven.
Visible rendering correctness is not proven.
Save completion is not proven.
First-lesson completion is not proven.
```

Legacy `limitations` remains for compatibility. It may be a broader or superset
list for older consumers, but it must include these six claims exactly enough for
automation to preserve them. New consumers should read `unproven_claims` first.

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
the nested readiness report.

Top-level fields:

| Field | Type | Description |
| --- | --- | --- |
| `schema_version` | string | Readiness report schema. |
| `manifest_path` | string | Comparison manifest inspected by the runner. |
| `scenario_id` | string or null | Scenario being checked. |
| `passed` | boolean | Structural evidence check result. |
| `status` | string | `ready`, `not_ready`, or `blocked`. |
| `readiness_status` | string | Backward-compatible detailed status. |
| `blocked_reason` | string or null | Machine-readable blocker reason when `status` is `blocked`. |
| `human_summary` | string | Plain scenario-focused summary. |
| `desktop_proof_contract` | object | Modernized desktop proof status rendered as the plain `Desktop proof` line. |
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
| `contract_evidence` | array | Required executable evidence checklist. Each item has `id`, `state`, `required`, and `summary`; required entries must be `present` or explicitly `blocked` with a structured blocker to avoid `passed: false`. |
| `diagnostics` | array | Structured contract diagnostics with `code`, `severity`, `field`, optional `expected`, and `message`. Any `error` diagnostic explains why the command exits non-zero. |
| `execute_requested` | boolean or null | Whether the comparison manifest was produced with execution enabled. |
| `issues` | array of strings | Blocking structural problems for automation and debug consumers. |
| `limitations` | array of strings | Backward-compatible non-claims. May remain a legacy/superset list, but must include the six canonical `unproven_claims`. |

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
| `candidate_actions` | array of strings | Candidate next actions reported by RabbitHole. |
| `requires_next_evidence` | array of strings | Evidence RabbitHole says must be collected next. |
| `observations` | array of strings | Plain observations from the next-action evidence. |
| `does_not_prove` | array of strings | Non-claims preserved for the desktop next-action section. |

Example:

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
4. Read `Desktop next action` when it appears. It describes RabbitHole's next
   observations or candidate actions, not completion.
5. Keep every `Unproven` line in handoffs, PRs, and release notes.

### Interpret Save evidence safely

Save-related evidence can show that a Save option, Save action, Save shortcut, or
Save artifact availability was observed. It can support a Save-completion claim
only when a distinct explicit Save-completion evidence item exists.

Safe wording:

```text
Save option evidence is shown as an observed option/action only.
Save completion is not yet proven.
```

Unsafe wording:

```text
The project was saved successfully.
The first lesson was completed.
```

### Review a creative-assessment gap safely

When the `creative_assessment` boundary reports `missing`, `invalid`,
`not_observed`, or `blocked`, treat the entry as a gap report. Use its `detail`
text to find available evidence and next steps for the learner's creative work
in this scenario, then collect or repair the scenario evidence that a human
reviewer needs.

Do not translate a creative-assessment gap into:

```text
The learner's world was graded.
The creative work is good or bad.
The first lesson is complete.
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
| `RabbitHole launch/action evidence is shown.` | `modernized ui-action-contract passed.` |
| `Desktop next-action evidence is not yet shown.` | Internal next-action artifact paths. |
| `Save option evidence is shown as an observed option/action only.` | `Save completed.` |
| `Visible rendering evidence is shown, but correctness is not proven.` | `Rendering is correct.` |
| `First-lesson completion is not yet shown.` | `The lesson is complete.` |

The durable rule is simple: report what the evidence explicitly shows, report
missing states as not yet shown or not yet proven, and keep the six unproven
claims visible.

## Implementation contract

The Rust implementation:

1. Emits `shown_evidence[]`, `not_yet_shown[]`, optional
   `desktop_next_action`, `unproven_claims`, and boundary-facing evidence items.
2. Maps existing progress and boundary states to user-facing `shown`, `not yet
   shown`, and `not yet proven` wording without exposing internal artifact paths
   in the plain sequence output.
3. Emits top-level `desktop_next_action` only for valid, safe, current RabbitHole
   evidence; otherwise it leaves the condition in `not_yet_shown`, `issues`, or
   legacy progress/boundary fields.
4. Preserves legacy JSON fields including `evidence_progress`, `target_evidence`,
   `lesson_session_readiness`, `role_readiness`, `issues`, and `limitations`.
5. Makes `unproven_claims` the canonical six non-claims and keeps `limitations`
   as compatibility output that includes those six.
6. Renders `alice run-first-lesson-readiness` without `--json` as readiness
   heading, `Desktop proof`, `Shown`, `Not yet shown`, optional `Desktop next
   action`, and `Unproven`.
7. Keeps Save action/artifact evidence separate from Save completion unless an
   explicit Save-completion evidence item exists.
