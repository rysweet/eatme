# Lesson session readiness

Lesson session readiness is the executable evidence contract for
instructor/student Alice lesson sessions. Its CLI readiness path is scoped to the
student first-lesson action contract, while instructor scenarios provide
canonical classroom handoff, remix, and rubric assets that remain validated
through asset and adapter checks. The contract connects four surfaces:

| Surface | Purpose |
| --- | --- |
| Canonical scenario assets | Describe instructor/student intent, evidence, boundaries, and unsupported-action policies. |
| Generated Gadugi adapters | Keep external runners aligned with canonical scenario assets. |
| Alice comparison manifests | Record baseline and modernized launch and automation scenario evidence for the same lesson scenario. |
| Readiness reports | Normalize the result as `ready`, `not_ready`, or `blocked` for humans, CI, and adapters. |

The readiness contract is deliberately outside-in. It reports required assets,
manifests, first-lesson automation scenario evidence, and not-yet-shown states
in machine-readable and user-facing forms. It does not implement missing Alice
desktop affordances, does not automate a complete lesson, does not perform
creative assessment, and does not grade student worlds.

For the conservative original Alice and RabbitHole boundary contract for Select
Project, procedure/edit, Save option/action evidence, visible rendering, grading,
creative assessment, and first-lesson completion, see
[First-Lesson Evidence Readiness](first-lesson-evidence-readiness.md). That page
defines the user-facing `Desktop proof`, `Shown`, `Not yet shown`, optional
`Desktop next action`, optional `Original Alice action evidence`, and `Unproven`
output, plus additive JSON `shown_evidence[]`, `not_yet_shown[]`,
`desktop_next_action`, `original_alice_action_evidence`, and `unproven_claims`
fields while preserving legacy progress and boundary fields.
For the artifact shape, filler rejection, and non-claim wording contract for
those evidence surfaces, see
[Evidence Artifact Contract](evidence-artifact-contract.md).

## Scenario map

Use these canonical scenarios for instructor/student lesson-session evidence:

| Scenario | Role | Evidence contract |
| --- | --- | --- |
| `first-lessons-real-ui-actions` | Student | Real Alice launch, Alice window evidence, first object/edit/run/save expectations, readiness progress evidence, first-lesson automation scenario evidence boundaries, and plain not-yet-shown states for missing desktop affordances. |
| `instructor-lesson-materials-remix` | Instructor | Teacher plan, student handout, exit ticket, acceptance probes, and review/remix language derived from Alice resources without launching Alice or grading learner worlds. |
| `instructor-student-launch-evidence-handoff` | Instructor | Handoff card, readiness note, and student action prompt that explain what launch/action evidence proves and what still requires classroom observation. |
| `instructor-student-outcomes-rubric` | Instructor | Student-visible outcomes rubric, feedback frame, revision next step, and project discussion guide without claiming automated creative assessment. |

The `alice check-lesson-readiness` and `alice run-first-lesson-readiness`
commands bind to `first-lessons-real-ui-actions`. Instructor scenarios are
canonical lesson-session evidence assets, not separate executable readiness
targets unless a harness owns that behavior.

Instructor and teacher mean the same role in this contract unless a scenario
explicitly distinguishes them.

## Scenario-to-gap matrix

This matrix explains what each scenario is meant to show, what remains
unproven, and what evidence would be needed before claiming that capability. It
describes each gap as the missing proof a student or instructor would need to
see, not as an internal task. The silver-thread gap is the next plain-language
question a classroom user still needs answered before trusting the lesson
journey from setup, to first action, to reflection and sharing.

| Scenario | What the user is trying to do | Remaining gap | Evidence still needed |
| --- | --- | --- | --- |
| First lesson real UI actions | Start Alice, see the lesson window, and try the first visible classroom actions with clear stopping points. | The record does not yet show a learner moving from opening Alice to one visible action while knowing exactly where the current evidence stops. | A learner-facing run showing the visible action attempted, what appeared on screen, where the learner had to stop, and the prompt they saw next. |
| Starter project open, save, and export preflight | Open a starter project, choose one small change to try, and understand what still needs to be checked before relying on the project in class. | The record does not yet show a learner returning to the project later or sharing an exported artifact without help. | A reproducible learner journey showing the opened project, the change attempt, the later return or export attempt, and the exact result shown to the learner. |
| Instructor lesson materials remix | Prepare a teacher plan, student handout, and exit ticket from Alice lesson resources. | The record does not yet show students using the remixed packet in class without the instructor filling in hidden steps. | Instructor-facing classroom notes showing which prompts students used, what confused them, and what the instructor changed before reuse. |
| Instructor-to-student launch handoff | Turn launch and window evidence into a plain student prompt for recording one Alice action, visible result, and next revision. | The record does not yet show students separating "Alice opened on this machine" from "my project behavior worked." | Student-visible handoff samples and instructor notes showing what students recorded, where they needed help, and which evidence stayed environment-only. |
| Student outcomes discussion rubric | Discuss student Alice projects using visible behavior, learner explanation, process evidence, and one next revision. | The record does not yet show different instructors using the discussion frame consistently with visible evidence and learner explanation. | Instructor-visible rubric use notes showing examples of visible behavior, student explanation, process evidence, and revision feedback across varied projects. |
| Classroom gallery walk and rubric | Help students review one another's Alice projects with respectful evidence, questions, and revision ideas. | The record does not yet show a live group giving useful peer feedback while protecting student privacy and keeping revision expectations clear. | Classroom-facing gallery notes showing peer observations, creator responses, revision checkpoints, and instructor boundary notes without private student records. |

## First-lesson next action readiness

eatme checks whether RabbitHole has produced the evidence needed before
continuing to the next first-lesson action. It reports `ready`, `not_ready`, or
`blocked`. This is not a lesson-completion implementation.

The required evidence check separates repository-local readiness evidence from
RabbitHole-produced desktop evidence:

| Evidence class | Accepted source | What it proves |
| --- | --- | --- |
| Canonical scenario evidence | `assets/scenarios/eatme/first-lessons-real-ui-actions.yaml` | The first-lesson boundary, required artifacts, non-claims, and unsupported-action policy are part of the validated eatme asset set. |
| Generated adapter evidence | `assets/scenarios/gadugi/first-lessons-real-ui-actions.yaml` | Adapter freshness proves the generated Gadugi scenario matches the current canonical scenario. RabbitHole-specific wording reaches adapters only after the canonical scenario is updated and adapters are regenerated. |
| Repository readiness evidence | Asset validation, generated-adapter freshness checks, comparison manifests, launch manifests, launch assertions, first-lesson readiness progress evidence, and the modernized visible desktop screenshot check | The repository can describe, launch, resolve, and normalize first-lesson readiness evidence without claiming the lesson was completed. |
| RabbitHole desktop evidence | Baseline and modernized target evidence in `comparison-manifest.json`, with RabbitHole-specific assertions on the modernized target | RabbitHole produced the required desktop signals for the next first-lesson action boundary. |

Repository readiness evidence is necessary, but it cannot replace RabbitHole
evidence. Readiness marks the next first-lesson action `ready` only after
RabbitHole evidence files show launch, the Run window, desktop execution,
screenshot artifacts, log artifacts, window artifacts, a readable action
contract, project proof-artifact states, and explicit boundary states for the
selected project, editing step, saved-artifact signal, visible rendering,
review boundary areas, and first-lesson stopping point.

If that evidence is missing, invalid, incomplete, or insufficient, eatme reports
`not_ready`. If the evidence is present but shows a known unsupported desktop
action, eatme reports `blocked`. Some existing schema fields still use
`no_go`; in plain language, that means "do not continue because this action is
unsupported."

### RabbitHole evidence needed before continuing

RabbitHole evidence uses the existing comparison, launch, desktop, and readiness
artifact model. Do not introduce a separate RabbitHole schema for first-lesson
readiness.

The comparison evidence must include both `baseline` and `modernized` targets
for the same `first-lessons-real-ui-actions` scenario. Both targets must satisfy
the shared launch and action-contract checks. The modernized target also owns
the RabbitHole desktop execution check.

| Required evidence | Existing artifact or assertion | `not_ready` or `blocked` condition |
| --- | --- | --- |
| Target identity | `comparison-manifest.json` with `execute_requested: true`, `scenario_id: "first-lessons-real-ui-actions"`, and both `baseline` and `modernized` target entries | Missing target, wrong scenario id, missing lesson-session contract, or target produced without execution. |
| Real Alice launch evidence | Embedded target launch manifest with the required first-lesson assertions, including `real_alice_execution_evidence`, `specific_alice_window_detected`, `activate_alice_window_ui_action`, `save_project_desktop_shortcut_dispatch`, `place_object_candidate_hook_probe`, and `ui_action_artifact_captured` | Missing launch manifest, wrong launch-manifest scenario id, missing required assertion, or required assertion not passing. |
| Specific Alice window evidence | `specific_alice_window_detected` and `activate_alice_window_ui_action` assertions | No specific Alice Stage IDE window, or activation evidence is absent. |
| Modernized Run-window evidence | `run_world_desktop_toolbar_window_observed` assertion on the `modernized` launch manifest | No RabbitHole evidence that the Run window appeared after the toolbar dispatch, or only an unstructured claim that a Run window appeared. The older `run_world_desktop_window_observed` shortcut assertion may appear in action evidence, but it is not the modernized RabbitHole readiness check. |
| Modernized desktop execution evidence | `run_world_desktop_execution_observed` assertion on the `modernized` launch manifest | No RabbitHole desktop Run execution artifact with runtime statement evidence. |
| Action contract artifact | Readable `ui-action-contract.json` referenced by target evidence and safely resolved under the comparison evidence root | Missing file, unsafe path, malformed JSON, missing required action ids, or missing explicit unsupported-action entries. |
| Save Project proof artifact | `save_project_proof_artifact` declaration from desktop next-action evidence, normalized to `present`, `missing`, or `blocked` | Missing declaration, unsafe artifact path, absent artifact metadata, or blocked save-project proof state. A present artifact proves artifact availability only; it does not prove Save completion without distinct explicit Save-completion evidence. |
| Select Project proof artifact | `select_project_proof_artifact` declaration from desktop next-action evidence, normalized to `present`, `missing`, or `blocked` | Missing declaration, unsafe artifact path, absent artifact metadata, or blocked select-project proof state. |
| Screenshot artifact | `screenshots/run-window-after-dispatch.png` next to the modernized `ui-action-contract.json`, canonicalized under the comparison evidence root | Missing file, empty file, unreadable file, symlink escape, or artifact outside the expected evidence root. |
| Log and window artifacts | Log, window-list, and startup screenshot paths represented by launch-manifest assertions | Missing, invalid, incomplete, or insufficient launch evidence. |

Readiness directly resolves and validates the UI action contract path, the
modernized visible desktop screenshot, and the desktop next-action evidence
proof-artifact states. Other launch artifacts such as logs, window lists, and
startup screenshots are represented through launch-manifest assertions;
readiness does not independently revalidate every referenced launch artifact.

The required first-lesson action ids stay the same:

```text
verify-specific-alice-window
activate-specific-alice-window
place-object
edit-procedure-or-code-block
run-world
save-project
```

The readiness check validates bounded evidence only. The Run-window evidence
records that RabbitHole prepared or opened the desktop Run frame. The desktop
execution evidence records that desktop execution started and produced runtime
statement evidence. Neither evidence item proves rendered output correctness,
creative quality, learner understanding, saved-world grading, or completed
lesson execution.

### Evidence status vocabulary

Readiness reports use explicit evidence states so human readers and automation
do not have to infer whether evidence is available, absent, malformed, not yet
observed, or blocked:

| Evidence state | Where it appears | Meaning |
| --- | --- | --- |
| `present` | `evidence_progress.items[].state` and `evidence_progress.present` | The named artifact declaration or proof summary is available and safe to report. This is artifact availability only. |
| `missing` | `evidence_progress.items[].state` and `evidence_progress.missing` | The declaration, artifact metadata, or safe evidence-root-relative path is absent or unusable. |
| `invalid` | `evidence_progress.items[].state` and `evidence_progress.invalid` | A producer supplied malformed or explicitly invalid desktop evidence. |
| `not_observed` | `evidence_progress.items[].state` and `evidence_progress.not_observed` | A desktop evidence producer ran, but the expected visible observation was not made. |
| `blocked` | `evidence_progress.items[].state`, `evidence_progress.blocked`, and readiness `status` when applicable | RabbitHole supplied a normalized blocker, or a known unsupported desktop affordance prevents continuing. |

Save Project and Select Project proof-artifact entries intentionally use only
`present`, `missing`, or `blocked`. The broader readiness progress object can
also emit `invalid` and `not_observed` for desktop pixel evidence. Malformed,
unsafe, or out-of-root project proof-artifact declarations are not promoted to
`present`; they remain `missing` and may also appear in `issues` so the caller
knows what to repair. `blocked` remains separate from `missing`: blocked means
the report received an explicit reason proof collection could not proceed.

### Readiness results

Treat these states as the decision for the next first-lesson action:

| Readiness result | Meaning | Required response |
| --- | --- | --- |
| `status: "not_ready"` | RabbitHole evidence is absent, incomplete, malformed, outside the expected evidence root, or not linked to the first-lesson scenario. | Do not proceed. Produce or repair the RabbitHole evidence artifact, then rerun readiness. |
| `status: "blocked"` | Required evidence is readable and coherent, the target failure category is a known UI-action blocker, and unsupported actions remain represented by explicit `no_go` entries. | Do not claim full first-lesson automation. Keep the blocker visible until deterministic action evidence replaces the unsupported-action entry. |
| `status: "ready"` | Required RabbitHole and repository evidence is present, coherent, and free of known unsupported desktop action blockers. | Treat the report as readiness to proceed to the next bounded first-lesson action only. Do not treat it as lesson completion. |

Local validation, launcher readiness, archive recovery, Run-window evidence, and
desktop execution evidence can make the repository ready to consume RabbitHole
artifacts. They cannot substitute for RabbitHole evidence. A report that lacks
the expected RabbitHole artifact set must not imply success.

### Expected RabbitHole evidence shape

RabbitHole evidence is accepted through the existing readiness JSON API. This
excerpt shows the shape a consumer should use when it finds the modernized
target, inspects its launch manifest, loads `ui-action-contract.json`, and
normalizes the target evidence:

```json
{
  "scenario_id": "first-lessons-real-ui-actions",
  "status": "blocked",
  "target_evidence": [
    {
      "role": "modernized",
      "target_status": "failed",
      "launch_manifest_present": true,
      "ui_action_contract_readable": true,
      "action_assertions": [
        {
          "assertion_id": "specific_alice_window_detected",
          "action_id": "verify-specific-alice-window",
          "passed": true,
          "detail": "wmctrl window list contains an Alice Stage IDE window"
        },
        {
          "assertion_id": "run_world_desktop_toolbar_window_observed",
          "action_id": "observe-run-window-after-toolbar-button",
          "passed": true,
          "detail": "observed RabbitHole evidence that the Run window appeared after the Run toolbar click"
        },
        {
          "assertion_id": "run_world_desktop_execution_observed",
          "action_id": "observe-desktop-run-execution-after-toolbar-button",
          "passed": true,
          "detail": "observed RabbitHole desktop Run execution artifact with VM statement events"
        }
      ],
      "required_actions": [
        "verify-specific-alice-window",
        "activate-specific-alice-window",
        "place-object",
        "edit-procedure-or-code-block",
        "run-world",
        "save-project"
      ],
      "no_go_contracts": [
        {
          "code": "unsupported_desktop_action",
          "action": "modernized.object_placement",
          "reason": "deterministic object-placement affordance is unavailable",
          "message": "RabbitHole object-placement evidence is blocked until deterministic desktop support exists."
        }
      ]
    }
  ],
  "original_alice_action_evidence": {
    "status": "missing",
    "summary": "Original Alice action evidence is missing.",
    "detail": "Original Alice action evidence was not found in the comparison target evidence."
  }
}
```

This example is a valid blocked result, not a completed lesson pass. It shows
that the consumer found modernized RabbitHole desktop evidence and preserved the
remaining unsupported action as an explicit blocker. A complete report also
includes the baseline target; omitting it is `not_ready`.

## Usage

### Validate the canonical assets

Run asset validation before trusting instructor/student readiness evidence:

```bash
cargo run -q -p eatme-cli -- assets validate --json
```

For targeted authoring, validate one scenario:

```bash
cargo run -q -p eatme-cli -- assets validate \
  --path assets/scenarios/eatme/first-lessons-real-ui-actions.yaml \
  --json
```

Passing validation reports `"passed": true`. A failure is a blocking readiness
problem; do not treat generated adapters, comparison manifests, or classroom
outputs as current until the canonical asset validates.

### Check generated Gadugi adapter freshness

Generated Gadugi adapters must match canonical scenario assets:

```bash
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
```

Use check mode in CI and before opening a PR. If check mode reports stale or
missing adapters, regenerate from the canonical assets:

```bash
cargo run -q -p eatme-cli -- assets generate-gadugi --json
```

Do not hand-edit generated adapters to change scenario intent. Edit the
canonical file under `assets/scenarios/eatme/` and regenerate.

### Check a comparison manifest

After a first-lesson comparison run writes a comparison manifest, verify the
lesson-session contract:

```bash
cargo run -q -p eatme-cli -- alice check-lesson-session \
  --manifest runs/comparisons/first-lessons-real-ui-actions/local-comparison/comparison-manifest.json \
  --json
```

This verifies that the comparison manifest contains a matching
`lesson_session_contract`, required first-lesson steps, expected evidence paths,
and explicit non-claims.

### Check student first-lesson readiness

Student first-lesson readiness is checked from the same comparison manifest:

```bash
cargo run -q -p eatme-cli -- alice check-lesson-readiness \
  --manifest runs/comparisons/first-lessons-real-ui-actions/local-comparison/comparison-manifest.json \
  --json
```

The command consumes embedded target launch manifests and each
`ui-action-contract.json`. It requires real Alice execution evidence, specific
Alice window evidence, action assertions, and matching action ids for the
student first-lesson flow. JSON adds the matching user-facing arrays and the
structured `original_alice_action_evidence` object while still keeping Save
Project and Select Project proof-artifact categories in legacy
`evidence_progress.items[]`. The plain `Original Alice action evidence` section
belongs to the `alice run-first-lesson-readiness` sequence renderer and appears
only when `original_alice_action_evidence.status` is `missing`.
Declarations come from the modernized target's desktop next-action evidence; if
that evidence or a category declaration is absent, the category remains visible
as not yet shown.

### Check the RabbitHole evidence needed before continuing

Use the same readiness command for RabbitHole. The comparison manifest must
include the modernized/RabbitHole target and must reference the target launch
manifest plus evidence files showing launch, the Run window, desktop execution,
screenshot artifacts, log artifacts, window artifacts, and
`ui-action-contract.json`. The modernized target also reports the Save Project
and Select Project proof-artifact states as `present`, `missing`, or `blocked`:

```bash
cargo run -q -p eatme-cli -- alice check-lesson-readiness \
  --manifest runs/comparisons/first-lessons-real-ui-actions/rabbithole-next-action/comparison-manifest.json \
  --json
```

Interpret the result as the required evidence check:

| Result | RabbitHole evidence state |
| --- | --- |
| `status: "ready"` | Required RabbitHole evidence is present, valid, and sufficient for the named bounded claims. |
| `status: "not_ready"` | Required RabbitHole evidence is not yet shown because it is missing, invalid, incomplete, or insufficient. |
| `status: "blocked"` | Required RabbitHole evidence was read, but it includes an explicit reason a bounded claim is not yet shown. |

Do not promote repository-only evidence to RabbitHole success. Asset validation,
adapter freshness, launcher checks, archive recovery, Run-window evidence, and
desktop execution evidence are useful only when the readiness report can connect
them to the RabbitHole target in the comparison manifest. Keep full UI
automation, grading, creative assessment, visible rendering correctness, Save
completion, and first-lesson completion explicitly unproven unless exact
evidence exists.

### Run the bounded first-lesson readiness sequence

Use the sequence command when the comparison and readiness check should be run
as one local workflow:

```bash
export NODE_OPTIONS=--max-old-space-size=32768
export ALICE_BASELINE_HOME=/path/to/alice-reference
export ALICE_MODERNIZED_HOME=/path/to/alice-candidate

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

and immediately runs the readiness check against that manifest.

Without `--execute`, the command still writes a comparison manifest, but the
readiness result is `status: "not_ready"` because target launch evidence is
missing.

## Readiness states

Readiness reports expose both a normalized state and the detailed legacy reason.

| Field | Values | Meaning |
| --- | --- | --- |
| `status` | `ready`, `not_ready`, `blocked` | Normalized top-level readiness state for humans, CI, and adapters. |
| `readiness_status` | `ready`, `incomplete`, `blocked_until_ui_automation` | Detailed reason retained for compatibility and debugging. |
| `blocked_reason` | `null` or a string | Present when `status` is `blocked`; `blocked_until_ui_automation` identifies unsupported UI automation blockers. |
| `passed` | `true` or `false` | Structural check result. `true` means required evidence is present and coherent, even if the normalized status is `blocked`. |

Interpret the states this way:

| State | Meaning | Common next action |
| --- | --- | --- |
| `ready` | Required RabbitHole evidence is present, valid, and sufficient. | Use the report as readiness evidence for the selected first-lesson scenario. |
| `not_ready` | Required evidence is missing, invalid, incomplete, stale, inconsistent, insufficient, or was produced without execution. | Fix assets, regenerate adapters, rerun comparison with `--execute`, or inspect `issues`. |
| `blocked` | Required evidence is present, but at least one target reports a known unsupported desktop action. | Treat the blocker as the honest boundary; do not mark the lesson as fully automated. |

A missing Save Project or Select Project proof artifact is reported as missing
artifact availability, not as a failed save or select action. A blocked Save
Project or Select Project proof artifact is reported separately from missing
evidence and preserves a normalized blocker summary when RabbitHole supplies
one.

A report can have `passed: true`, `status: "blocked"`, and
`readiness_status: "blocked_until_ui_automation"`. That means structural
evidence exists, the target failure category is a known UI-action blocker, and
the remaining unsupported actions are represented by explicit blocker entries.

For `first-lessons-real-ui-actions`, that blocked-but-valid state is the
evidence-ready state when deterministic object placement, procedure editing,
world running, or project saving affordances are still represented by
unsupported-action entries. The `ready` state is the no-blocker state: the report
uses it only when required evidence is present and no unsupported-action blocker
remains.

## Readiness JSON API

The student first-lesson readiness report schema is
`eatme.alice-lesson-session-readiness/v1`. The fields below include the
user-facing additions. Existing legacy fields remain part of the compatibility
contract until consumers have moved to the additive report shape.

Top-level fields:

| Field | Type | Description |
| --- | --- | --- |
| `schema_version` | string | Report schema. |
| `manifest_path` | string | Comparison manifest that was inspected. |
| `scenario_id` | string or null | Scenario id from the comparison manifest. |
| `passed` | boolean | `true` when required structural evidence is present and coherent. |
| `status` | string | Normalized state: `ready`, `not_ready`, or `blocked`. |
| `readiness_status` | string | Detailed status such as `ready`, `incomplete`, or `blocked_until_ui_automation`. |
| `blocked_reason` | string or null | Machine-readable blocker reason when `status` is `blocked`. |
| `human_summary` | string | Single-sentence human explanation of the readiness result. |
| `shown_evidence` | array | User-facing evidence facts that are shown by accepted evidence. |
| `not_yet_shown` | array | User-facing missing, invalid, not-observed, or blocked claims. |
| `desktop_next_action` | object or omitted | RabbitHole desktop next-action summary, emitted only when valid, safe, current, and applicable to the current run. Missing or invalid evidence is represented through not-yet-shown, issues, or legacy fields instead. |
| `original_alice_action_evidence` | object | Structured original Alice action evidence summary. `status` is `missing` when a target evidence blocker has `code: "missing_real_action_evidence"`; otherwise it is `available`. |
| `unproven_claims` | array | Canonical non-claims that remain visible in plain output and JSON. |
| `desktop_proof_contract` | object | Machine-readable modernized desktop proof state: `skipped`, `unsupported_environment`, `launched_but_unverified`, or `verified`. |
| `evidence_progress` | object | Required-evidence counts, project proof-artifact entries, and next blocker/proof hints using explicit `present`, `missing`, `invalid`, `not_observed`, and `blocked` states. |
| `required_evidence` | array of strings | Durable evidence names required by the readiness check, including Save Project and Select Project proof-artifact state entries. |
| `no_go_contracts` | array | Aggregated unsupported-action entries from target evidence. |
| `lesson_session_readiness` | object | Backward-compatible normalized student readiness envelope. |
| `role_readiness` | array | Normalized readiness envelopes for `instructor` and `student`. |
| `contract_check` | object | Result from `alice check-lesson-session`. |
| `execute_requested` | boolean or null | Whether the comparison manifest was produced with execution enabled. |
| `evidence_progress.next_missing_real_desktop_proof` | string or omitted | Plain next missing real-desktop proof after the current window/action diagnostics, such as Alice window activation, Run-window observation, desktop execution, screenshot capture, Run pixel observation, desktop next-action evidence, Save Project proof artifact, or Select Project proof artifact. This value uses display-safe evidence labels rather than internal artifact paths. |
| `target_evidence` | array | Per-target launch/action evidence for baseline and modernized targets. |
| `issues` | array of strings | Blocking structural problems. |
| `limitations` | array of strings | Backward-compatible non-claims. May remain a legacy/superset list, but must include the canonical six `unproven_claims`. |

First-lesson boundary reporting adds `evidence_boundaries[]` to this schema.
Consumers that need the bounded automation scenarios contract should read
`evidence_boundaries[]`; older consumers can continue to use
`evidence_progress.items[]` and the project proof-artifact entries.

### Original Alice action evidence API

`original_alice_action_evidence` is the stable summary for original Alice action
evidence state. It is additive and does not remove target-local diagnostics from
`target_evidence[]`.

| Field | Type | Description |
| --- | --- | --- |
| `status` | string | `available` or `missing`. |
| `summary` | string | Fixed user-facing summary: `Original Alice action evidence is missing.` or `Original Alice action evidence is available.` |
| `detail` | string | Fixed bounded detail for the selected status. |

The readiness builder derives this field after building `target_evidence[]`.
The only missing-evidence source of truth is a target evidence blocker with
`code: "missing_real_action_evidence"`. The blocker remains in
`target_evidence[]`; the new field prevents the state from being hidden inside
nested diagnostics only. Missing original Alice action evidence is reportable
structured state. It is not fatal by itself and does not become a claim that full
UI automation, Save completion, lesson completion, grading, or creative
assessment succeeded. `available` only means no `missing_real_action_evidence`
blocker was found; it does not prove full UI automation, Save completion, lesson
completion, grading, or creative assessment.

### Evidence progress API

`evidence_progress` is the shared progress object used by JSON output and plain
CLI output. It reports observed evidence state only; it does not grade the
lesson, prove UI completion, or collapse blocked evidence into missing evidence.

| Field | Type | Description |
| --- | --- | --- |
| `total_required` | number | Number of required evidence items represented in `items`. |
| `present` | number | Count of items whose `state` is `present`. This is artifact availability, not lesson completion. |
| `missing` | number | Count of items whose `state` is `missing`. |
| `invalid` | number | Count of items whose `state` is `invalid`. |
| `not_observed` | number | Count of items whose `state` is `not_observed`. |
| `blocked` | number | Count of items whose `state` is `blocked`. |
| `summary` | string | Human-readable aggregate count summary. |
| `next_actionable_blocker` | string or omitted | Next unsupported action blocker reported by RabbitHole. |
| `items` | array | Required evidence entries. Each entry has `id`, `evidence`, `state`, and `detail`. |
| `next_missing_real_desktop_proof` | string or omitted | The next real-desktop proof to collect when evidence is missing or blocked. When the missing proof is the next-action artifact, the value includes the label `desktop next-action evidence`, not an internal artifact path. |

Across the full progress object, `items[].state` can be `present`, `missing`,
`invalid`, `not_observed`, or `blocked`. The Save Project and Select Project
proof-artifact entries are the narrower subset that use only `present`,
`missing`, or `blocked`: use `present` for observed artifact availability,
`missing` for absent or unusable evidence, and `blocked` only when RabbitHole
supplies an explicit blocker.

Project proof-artifact item example:

```json
{
  "id": "select_project_proof_artifact",
  "evidence": "Select Project proof artifact",
  "state": "blocked",
  "detail": "blocked: project selector proof is not available in this RabbitHole run; codes: select_project_proof_unavailable"
}
```

This means proof collection hit an explicit boundary. It does not mean Alice
completed a lesson, fully executed a learner world, saved a learner world through
full UI automation, deployed sharing, proved platform success, or graded
creative work.

### Desktop proof contract

`desktop_proof_contract` is intentionally narrower than full first-lesson
automation. It reports what happened to the modernized desktop proof attempt:

| Status | Meaning |
| --- | --- |
| `skipped` | Execution was not requested, or no modernized target evidence exists. This is a deliberate manual smoke skip, not a failed proof. |
| `unsupported_environment` | Execution was requested, but the modernized target could not launch desktop proof collection, for example because Alice home resolution or required target paths failed. |
| `launched_but_unverified` | Alice launch evidence exists, but Run-window, desktop execution, screenshot, or pixel-observation proof is missing, blocked, invalid, or not observed. |
| `verified` | The modernized evidence includes Run-window dispatch, desktop execution, visible screenshot, and observed pixel evidence. This still does not prove complete lesson automation, full world execution, rendering correctness, grading, save behavior, deployed sharing/platform success, or creative assessment. |

This legacy desktop proof contract includes `reason_code`, `detail`,
`target_role`, and optional `artifact` fields so CI and reports can preserve the
exact skip shape. First-lesson unsupported-action blockers use the canonical
`code`, `action`, `reason`, and safe `message` shape described below.

### Project proof-artifact states

Save Project and Select Project are proof-artifact categories. They describe
whether RabbitHole supplied an auditable artifact declaration for the action
boundary. They do not say that Alice UI automation succeeded, that a lesson was
completed, that a saved world was graded, or that creative quality was assessed.

The modernized target reads optional declarations from desktop next-action
evidence. The runner still resolves and validates the underlying artifact under
the comparison evidence root, but readiness output describes it with the
display-safe label `desktop next-action evidence`.

The accepted declaration fields are:

```json
{
  "save_project_proof_artifact": {
    "artifact": {
      "path": "project-save/saved-project.a3p",
      "size_bytes": 81342,
      "sha256": "2d6f6f7e9c5a..."
    },
    "metadata": {
      "source": "tools/eatme-save-project"
    }
  },
  "select_project_proof_artifact": {
    "blocker": {
      "reason": "project selector proof is not available in this RabbitHole run",
      "codes": ["select_project_proof_unavailable"]
    }
  }
}
```

A minimal blocked declaration without detail is also valid:

```json
{
  "save_project_proof_artifact": {
    "status": "blocked"
  }
}
```

Normalization uses this precedence for each category:

| Normalized state | Condition | Boundary |
| --- | --- | --- |
| `blocked` | The declaration has blocker metadata, or declares `status: "blocked"`. | Report the blocker reason, codes, next action, or component state when present. If no detail exists, report only that the proof artifact is blocked. |
| `present` | The declaration has `ArtifactInfo` metadata or a safe comparison-evidence-root-relative artifact path. | Report artifact availability only. These entries count toward `evidence_progress.present`. Path, `size_bytes`, `sha256`, and normalized metadata summaries may be included. |
| `missing` | No declaration exists, the declaration has no usable artifact metadata, or the path is absent or unsafe. | Report missing artifact availability plainly. Do not convert this to success language. |

`blocked` remains distinct from `missing`: blocked means RabbitHole supplied an
explicit reason that proof collection could not proceed; missing means the
readiness report did not receive a usable proof-artifact declaration. An unsafe
absolute path, traversal path, or artifact outside the evidence root is not
treated as present.

Both project proof-artifact categories are reported even when desktop
next-action evidence is missing or does not declare them. In that case the Save
Project and Select Project entries appear as `missing` instead of disappearing
from the report.

Artifact paths emitted by readiness are evidence-root-relative paths from the
declaration. The readiness report must not emit absolute host paths for Save
Project or Select Project proof artifacts, must not read the referenced project
artifact contents, and must not embed artifact contents in JSON or plain output.
RabbitHole metadata and blockers are normalized into summaries; raw metadata and
raw blocker objects are not part of the shared progress-item schema.

The shared progress entries use stable evidence labels:

| Declaration key | Human label | States |
| --- | --- | --- |
| `save_project_proof_artifact` | Save Project proof artifact | `present`, `missing`, `blocked` |
| `select_project_proof_artifact` | Select Project proof artifact | `present`, `missing`, `blocked` |

Example `evidence_progress.items[]` entries:

```json
[
  {
    "evidence": "Save Project proof artifact",
    "state": "present",
    "detail": "artifact path project-save/saved-project.a3p, size_bytes=81342, sha256=2d6f6f7e9c5a...; presence is not proof of full UI automation"
  },
  {
    "evidence": "Select Project proof artifact",
    "state": "blocked",
    "detail": "blocked: project selector proof is not available in this RabbitHole run; codes: select_project_proof_unavailable"
  }
]
```

A minimal `status: "blocked"` declaration without detail emits a progress item
like:

```json
{
  "evidence": "Save Project proof artifact",
  "state": "blocked",
  "detail": "blocked"
}
```

Consumers that only need a human-readable report can read `evidence`, `state`,
and `detail`. Consumers that audit artifacts should inspect the original
RabbitHole evidence artifact separately; the readiness progress API emits
normalized summaries, not raw artifact metadata, raw blocker JSON, or artifact
contents.

### Normalized envelope

Downstream consumers should prefer `role_readiness` when they need explicit
instructor/student readiness states. `lesson_session_readiness` remains the
backward-compatible student envelope.

```json
{
  "role_readiness": [
    {
      "scenario_id": "first-lessons-real-ui-actions",
      "role": "instructor",
      "status": "blocked",
      "blocked_reason": "blocked_until_ui_automation",
      "human_summary": "first-lessons-real-ui-actions has launch/action-contract evidence but is blocked until deterministic desktop UI automation exists (blocked_until_ui_automation).",
      "required_evidence": [
        "comparison-manifest.json",
        "ui-action-contract.json",
        "Save Project proof artifact",
        "Select Project proof artifact"
      ],
      "no_go_contracts": [
        {
          "code": "unsupported_desktop_action",
          "action": "baseline.object_placement",
          "reason": "deterministic object-placement affordance is unavailable",
          "message": "Original Alice object-placement evidence is blocked until deterministic desktop support exists."
        },
        {
          "code": "unsupported_desktop_action",
          "action": "baseline.procedure_edit",
          "reason": "deterministic procedure-edit affordance is unavailable",
          "message": "Original Alice procedure/edit evidence is blocked until deterministic desktop support exists."
        },
        {
          "code": "unsupported_desktop_action",
          "action": "baseline.world_run",
          "reason": "deterministic world-run affordance is unavailable",
          "message": "Original Alice world-run evidence is blocked until deterministic desktop support exists."
        },
        {
          "code": "unsupported_desktop_action",
          "action": "baseline.project_save",
          "reason": "deterministic project-save affordance is unavailable",
          "message": "Original Alice project-save evidence is blocked until deterministic desktop support exists."
        }
      ]
    },
    {
      "scenario_id": "first-lessons-real-ui-actions",
      "role": "student",
      "status": "blocked",
      "blocked_reason": "blocked_until_ui_automation",
      "human_summary": "first-lessons-real-ui-actions has launch/action-contract evidence but is blocked until deterministic desktop UI automation exists (blocked_until_ui_automation).",
      "required_evidence": [
        "comparison-manifest.json",
        "ui-action-contract.json",
        "Save Project proof artifact",
        "Select Project proof artifact"
      ],
      "no_go_contracts": [
        {
          "code": "unsupported_desktop_action",
          "action": "baseline.object_placement",
          "reason": "deterministic object-placement affordance is unavailable",
          "message": "Original Alice object-placement evidence is blocked until deterministic desktop support exists."
        }
      ]
    }
  ],
  "lesson_session_readiness": {
    "scenario_id": "first-lessons-real-ui-actions",
    "role": "student",
    "status": "blocked",
    "blocked_reason": "blocked_until_ui_automation",
    "human_summary": "first-lessons-real-ui-actions has launch/action-contract evidence but is blocked until deterministic desktop UI automation exists (blocked_until_ui_automation).",
    "required_evidence": [
      "comparison-manifest.json",
      "ui-action-contract.json",
      "Save Project proof artifact",
      "Select Project proof artifact"
    ],
    "no_go_contracts": [
      {
        "code": "unsupported_desktop_action",
        "action": "baseline.object_placement",
        "reason": "deterministic object-placement affordance is unavailable",
        "message": "Original Alice object-placement evidence is blocked until deterministic desktop support exists."
      }
    ]
  }
}
```

Envelope fields:

| Field | Type | Description |
| --- | --- | --- |
| `scenario_id` | string or null | Scenario being checked. |
| `role` | string | Readiness role for the envelope; reports include `instructor` and `student`. |
| `status` | string | `ready`, `not_ready`, or `blocked`. |
| `blocked_reason` | string or null | Blocker reason when status is `blocked`. |
| `human_summary` | string | Human-readable state summary. |
| `required_evidence` | array of strings | Required durable evidence artifacts and proof-artifact state entries. |
| `no_go_contracts` | array | Unsupported-action entries that prevent silent success. |

### Target evidence

Each `target_evidence[]` entry describes one comparison target:

| Field | Type | Description |
| --- | --- | --- |
| `role` | string | Comparison role, usually `baseline` or `modernized`. |
| `target_id` | string or null | Target id from the comparison manifest. |
| `target_status` | string or null | Target comparison status. |
| `failure_category` | string or null | Launch/action failure category from the target manifest. |
| `launch_manifest_present` | boolean | Whether the target has a readable launch manifest reference. |
| `ui_action_contract_path` | string or null | Path to the target `ui-action-contract.json`. |
| `ui_action_contract_readable` | boolean | Whether the UI action contract could be parsed. |
| `desktop_first_lesson_next_action` | object or null | Parsed desktop next-action evidence for the modernized target. Save Project and Select Project categories still appear in `evidence_progress.items[]` as `missing` when declarations are absent. |
| `action_assertions` | array | Required action assertions and their pass/fail status. |
| `required_actions` | array of strings | Action ids discovered from the UI action contract. |
| `missing_assertions` | array of strings | Required assertions absent from the target evidence. |
| `missing_required_actions` | array of strings | Required action ids absent from the UI action contract. |
| `blockers` | array | Target-local structured blockers. A blocker with `code: "missing_real_action_evidence"` is also summarized by top-level `original_alice_action_evidence.status: "missing"`. |
| `no_go_contracts` | array | Target-local unsupported-action entries. |

Required action ids for the first-lesson flow are:

```text
verify-specific-alice-window
activate-specific-alice-window
place-object
edit-procedure-or-code-block
run-world
save-project
```

## Unsupported-action (`no_go`) contract API

The JSON schema uses structured unsupported-action blockers for known unsupported
desktop actions. Those entries make missing desktop action support explicit. They
are aggregated from UI action precondition probes and required-action entries
whose source contract says `decision: "no_go"` or whose
`contract_required.unsafe_until_available` flag is true. They prevent the
harness, adapters, and docs from converting unsupported actions into silent
success.

Each unsupported-action entry uses the canonical structured blocker shape:

| Field | Type | Description |
| --- | --- | --- |
| `code` | string | Stable machine-readable blocker category, such as `unsupported_desktop_action`. |
| `action` | string or null | Target-qualified action or boundary, such as `baseline.object_placement`, `baseline.procedure_edit`, `baseline.world_run`, or `baseline.project_save`. |
| `reason` | string | Stable reason phrase suitable for logs and CI. |
| `message` | string | Safe human-readable message. It must not expose absolute paths, raw artifact contents, screenshots, logs, environment variables, secrets, or raw framework internals. |

Known unsupported action reasons:

| Reason | Action | Meaning |
| --- | --- | --- |
| `deterministic object-placement affordance is unavailable` | `object_placement` | A stable Alice-side way to place a named gallery object and produce durable evidence is unavailable. |
| `deterministic procedure-edit affordance is unavailable` | `procedure_edit` | A stable Alice-side way to edit a procedure or code block and prove the edit is unavailable. |
| `deterministic world-run affordance is unavailable` | `world_run` | A stable Alice-side way to prove the world ran after student edits is unavailable. |
| `deterministic project-save affordance is unavailable` | `project_save` | A stable Alice-side way to prove a project save artifact is unavailable. |

Consumers must treat `code: "unsupported_desktop_action"` as a blocked action,
not as a failed test to hide and not as a pass to override.

## Configuration

### Environment variables

| Variable | Required for | Description |
| --- | --- | --- |
| `NODE_OPTIONS=--max-old-space-size=32768` | Agentic/Gadugi-heavy local runs | Keeps Node-based runners from failing under large prompt or adapter workloads. |
| `EATME_REAL_ALICE=1` | Non-baseline real Alice execution | Explicit opt-in gate for real desktop runs. |
| `ALICE_HOME` | Single-target Alice commands | Alice checkout for `alice discover`, `alice package`, and `alice launch-smoke`. |
| `ALICE_BASELINE_HOME` | Comparison/readiness sequence | Reference Alice checkout used as the baseline target. |
| `ALICE_MODERNIZED_HOME` | Comparison/readiness sequence | Candidate Alice checkout used as the modernized target. |

The saved local preference for Node-based runner capacity is:

```bash
export NODE_OPTIONS=--max-old-space-size=32768
```

Change that preference in your local Amplihack config when local agentic or
wrapper tooling needs a different heap. The Rust readiness commands do not use
Node to parse Save Project or Select Project proof artifacts.

### Real desktop requirements

Real Alice launch/action evidence requires the desktop dependency set documented
in [Alice Integration](alice-integration.md): Java 21, Maven, Xvfb, `xdpyinfo`,
`wmctrl`, `xwininfo`, `xdotool`, screenshot tooling, and software OpenGL support.

CI should keep validation, adapter freshness, Rust checks, and docs builds fast.
Do not add unconditional real desktop execution to required pull-request jobs.
Real Alice smoke and readiness runs remain explicit local or self-hosted gates.

## Examples

### Student flow: inspect a blocked-but-valid readiness report

```bash
cargo run -q -p eatme-cli -- alice check-lesson-readiness \
  --manifest runs/comparisons/first-lessons-real-ui-actions/local/comparison-manifest.json \
  --json
```

Expected interpretation:

```json
{
  "passed": true,
  "status": "blocked",
  "readiness_status": "blocked_until_ui_automation",
  "blocked_reason": "blocked_until_ui_automation"
}
```

This is acceptable first-lesson evidence when the report also includes
`ui-action-contract.json` evidence and action-level `no_go_contracts`
(unsupported-action entries). It is not a completed UI automation pass.

### Student flow: inspect first-lesson scenario evidence

The first-lesson sequence plain renderer names shown and not-yet-shown scenario
evidence boundaries:

```text
First-lesson automation scenario readiness: not ready
Desktop proof: launched_but_unverified (desktop_run_window_unverified) - desktop Run window dispatch lacks modernized-target proof

Shown:
- Select Project scenario evidence is shown.
- Procedure/edit scenario evidence is shown.
- Visible rendering scenario evidence is shown.

Not yet shown:
- Save completion is not yet proven.
- First-lesson completion is not yet shown.

Original Alice action evidence:
- Original Alice action evidence is missing.
- Original Alice action evidence was not found in the comparison target evidence.

Unproven:
- Full Alice UI automation is not proven.
- Grading is not proven.
- Creative assessment is not proven.
- Visible rendering correctness is not proven.
- Save completion is not proven.
- First-lesson completion is not proven.
```

JSON output exposes the same conservative states in `shown_evidence[]`,
`not_yet_shown[]`, and `evidence_boundaries[]` entries. Target-local
launch/action details remain in `target_evidence[]`. This excerpt shows only
two entries from the longer boundary array:

```json
[
  {
    "id": "select_project",
    "label": "Select Project scenario evidence",
    "status": "present",
    "detail": "Select Project scenario evidence is present."
  },
  {
    "id": "save_project",
    "label": "Save option/action scenario evidence",
    "status": "blocked",
    "detail": "Save completion is not yet proven for this run."
  }
]
```

The Select Project line says only that the Select Project boundary has scenario
evidence. The Save line says Save completion is not yet proven. If a boundary
has no evidence, only ambiguous metadata, or no usable relative summary, it
remains visible as not yet shown. Neither line proves lesson completion, full UI
automation, rendering correctness, grading, creative assessment, or learner-world
grading.

### Student flow: fix a not-ready report

```json
{
  "passed": false,
  "status": "not_ready",
  "readiness_status": "incomplete",
  "issues": [
    "comparison manifest must be produced with --execute to contain target launch evidence"
  ]
}
```

Rerun the comparison/readiness sequence with `--execute` and
`EATME_REAL_ALICE=1`, then re-check readiness.

### Instructor asset flow: prepare the classroom handoff

Validate the instructor/student handoff asset:

```bash
cargo run -q -p eatme-cli -- assets validate \
  --path assets/scenarios/eatme/instructor-student-launch-evidence-handoff.yaml \
  --json
```

The instructor handoff asset uses the real Alice manifest, log, window list,
screenshot, and `ui-action-contract.json` as evidence inputs. It produces:

| Output | Required content |
| --- | --- |
| `real_alice_evidence_handoff_card` | What each artifact proves and what it does not prove. |
| `instructor_readiness_note` | Which signals indicate environment readiness and which observations remain classroom work. |
| `student_action_prompt` | One Alice action, visible result after running, and one next revision for the student to record. |

### Instructor asset flow: discuss student outcomes

Validate the outcomes rubric asset:

```bash
cargo run -q -p eatme-cli -- assets validate \
  --path assets/scenarios/eatme/instructor-student-outcomes-rubric.yaml \
  --json
```

The instructor outcomes asset produces:

| Output | Required content |
| --- | --- |
| `student_outcomes_rubric` | Instructor-friendly levels tied to visible Alice behavior and concept evidence. |
| `feedback_frame` | Feedback that names learner explanation, process evidence, reflection, and accessibility or audience when relevant. |
| `revision_next_step` | One student-owned, testable next revision. |
| `project_discussion_guide` | Student evidence questions and an explicit instructor boundary note. |

The rubric can support instructor judgment. It must not claim automated creative
assessment, learner-world grading, complete Alice coverage, full UI automation,
or deployed-service status.

## Tutorial: add or revise lesson-session evidence

1. Edit the canonical scenario under `assets/scenarios/eatme/`.
2. State the role, expected outputs, evidence artifacts, and unsupported policy in
   the scenario YAML.
3. Use `ready`, `not_ready`, and `blocked` wording for readiness outputs.
4. Add explicit blockers for missing desktop affordances instead of implying
   silent success; explain that they report `blocked`.
5. If the scenario consumes RabbitHole first-lesson evidence, preserve
   `target_evidence[]`, `evidence_boundaries[]`, `evidence_progress.items[]`,
   and project proof-artifact states. Report Select Project, procedure/edit,
   Save, visible rendering, grading, creative assessment, and first-lesson
   completion separately as `present`, `missing`, `invalid`, `not_observed`, or
   `blocked`.
   Preserve blocker information as a normalized summary and keep boundary
   evidence separate from UI success, rendering correctness, grading, creative
   assessment, and completion language.
6. Validate the changed asset:

   ```bash
   cargo run -q -p eatme-cli -- assets validate \
     --path assets/scenarios/eatme/<scenario-id>.yaml \
     --json
   ```

7. Check generated adapter freshness:

   ```bash
   cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
   ```

8. If adapters are stale, regenerate them:

   ```bash
   cargo run -q -p eatme-cli -- assets generate-gadugi --json
   ```

9. For student first-lesson changes, run or inspect a readiness report and
   confirm it exposes `status`, `lesson_session_readiness`, and the
   first-lesson scenario evidence boundaries. For instructor-only asset changes,
   keep the evidence boundary in scenario validation and generated adapters
   unless an executable instructor harness is added.
10. Run the repository quality gate before handoff:

   ```bash
   ./scripts/quality-gates.sh
   ```

## PR-ready documentation checklist

Before opening a PR for instructor/student lesson-session evidence, include this
summary in the PR description:

| Item | Required statement |
| --- | --- |
| Scenario validation | `cargo run -q -p eatme-cli -- assets validate --json` passed. |
| Gadugi freshness | `cargo run -q -p eatme-cli -- assets generate-gadugi --check --json` passed, or adapters were regenerated and committed. |
| Readiness output | Student first-lesson reports expose normalized `status`, `lesson_session_readiness`, `target_evidence[]`, `evidence_boundaries[]`, and `evidence_progress.items[]`; instructor-only changes do not claim a readiness report unless a harness produces one. |
| First-lesson scenario evidence | Reports preserve target-local launch/action diagnostics, project proof-artifact, readiness progress, and boundary states for Select Project, procedure/edit, Save, visible rendering, grading, creative assessment, and first-lesson completion as `present`, `missing`, `invalid`, `not_observed`, or `blocked`, with normalized blocker summaries when supplied. |
| Unsupported desktop actions | Unsupported desktop actions are explicit blockers that report `blocked`. |
| Boundaries | The change does not claim full UI automation, full world execution, visible rendering correctness, Save completion, grading, creative assessment, learner-world grading, first-lesson completion, complete Alice coverage, deployed sharing/platform success, or deployed-service status unless distinct explicit evidence exists. |
| Quality gate | `./scripts/quality-gates.sh` passed. |
