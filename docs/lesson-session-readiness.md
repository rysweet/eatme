# Lesson session readiness

Lesson session readiness is the executable evidence contract for the
instructor/student Alice lesson-session feature being built. Its executable CLI
readiness path is currently scoped to the student first-lesson action contract,
while instructor scenarios provide canonical classroom handoff, remix, and rubric
assets that remain validated through asset and adapter checks. The contract
connects four surfaces:

| Surface | Purpose |
| --- | --- |
| Canonical scenario assets | Describe instructor/student intent, evidence, boundaries, and no-go policies. |
| Generated Gadugi adapters | Keep external runners aligned with canonical scenario assets. |
| Alice comparison manifests | Record baseline and modernized launch/action-contract evidence for the same lesson scenario. |
| Readiness reports | Normalize the result as `ready`, `not_ready`, or `blocked` for humans, CI, and adapters. |

The readiness contract is deliberately outside-in. It proves that required
assets, manifests, UI action contracts, and no-go blockers are visible and
machine-readable. It does not implement missing Alice desktop affordances, does
not automate a complete lesson, does not perform creative assessment, and does
not grade student worlds.

## Scenario map

Use these canonical scenarios for instructor/student lesson-session evidence:

| Scenario | Role | Evidence contract |
| --- | --- | --- |
| `first-lessons-real-ui-actions` | Student | Real Alice launch, Alice window evidence, `ui-action-contract.json`, first object/edit/run/save expectations, and explicit action-level `no_go` decisions for missing desktop affordances. |
| `instructor-lesson-materials-remix` | Instructor | Teacher plan, student handout, exit ticket, acceptance probes, and review/remix language derived from Alice resources without launching Alice or grading learner worlds. |
| `instructor-student-launch-evidence-handoff` | Instructor | Handoff card, readiness note, and student action prompt that explain what launch/action evidence proves and what still requires classroom observation. |
| `instructor-student-outcomes-rubric` | Instructor | Student-visible outcomes rubric, feedback frame, revision next step, and project discussion guide without claiming automated creative assessment. |

The `alice check-lesson-readiness` and `alice run-first-lesson-readiness`
commands currently bind to `first-lessons-real-ui-actions`. Instructor scenarios
are canonical lesson-session evidence assets, not separate executable readiness
targets, until a future instructor-specific harness owns that behavior.

Instructor and teacher mean the same role in this contract unless a future
scenario explicitly distinguishes them.

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
student first-lesson flow.

### Run the bounded first-lesson readiness sequence

Use the sequence command when the comparison and readiness check should be run
as one local workflow:

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
| `blocked_reason` | `null` or a string | Present when `status` is `blocked`; currently `blocked_until_ui_automation`. |
| `passed` | `true` or `false` | Structural check result. `true` means required evidence is present and coherent, even if the normalized status is `blocked`. |

Interpret the states this way:

| State | Meaning | Common next action |
| --- | --- | --- |
| `ready` | Required comparison and UI action evidence is present, coherent, and has no accepted blockers. | Use the report as completed no-blocker readiness evidence for the selected first-lesson scenario. |
| `not_ready` | Required evidence is missing, invalid, stale, inconsistent, or was produced without execution. | Fix assets, regenerate adapters, rerun comparison with `--execute`, or inspect `issues`. |
| `blocked` | Required evidence is present, but the current blocker is an explicit missing desktop affordance. | Treat the no-go contract as the honest boundary; do not mark the lesson as fully automated. |

A report can have `passed: true`, `status: "blocked"`, and
`readiness_status: "blocked_until_ui_automation"`. That means the executable
evidence exists and the only accepted blocker is the documented lack of
deterministic Alice desktop actions.

For the current `first-lessons-real-ui-actions` implementation, that
blocked-but-valid state is the expected evidence-ready state until deterministic
object placement, procedure editing, world running, and project saving
affordances replace the no-go entries. The `ready` state is part of the stable
schema for the future no-blocker state; if the harness starts producing it,
update the readiness checks and this page together so the design and behavior
stay aligned.

## Readiness JSON API

The student first-lesson readiness report schema is
`eatme.alice-lesson-session-readiness/v1`.

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
| `required_evidence` | array of strings | Durable artifact names required by the readiness check. |
| `no_go_contracts` | array | Aggregated action-level no-go contracts from target evidence. |
| `lesson_session_readiness` | object | Backward-compatible normalized student readiness envelope. |
| `role_readiness` | array | Normalized readiness envelopes for `instructor` and `student`. |
| `contract_check` | object | Result from `alice check-lesson-session`. |
| `execute_requested` | boolean or null | Whether the comparison manifest was produced with execution enabled. |
| `target_evidence` | array | Per-target launch/action evidence for baseline and modernized targets. |
| `issues` | array of strings | Blocking structural problems. |
| `limitations` | array of strings | Non-claims that remain true even when the report passes. |

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
        "ui-action-contract.json"
      ],
      "no_go_contracts": [
        {
          "target_role": "baseline",
          "affordance": "object_placement",
          "decision": "no_go",
          "reason": "missing deterministic desktop affordance for artifact proves a named object was added to the scene and placed without coordinate guessing",
          "missing_affordance_id": "deterministic-alice-object-gallery-placement-affordance"
        },
        {
          "target_role": "baseline",
          "affordance": "procedure_edit",
          "decision": "no_go",
          "reason": "missing deterministic desktop affordance for artifact proves a procedure or code block was edited",
          "missing_affordance_id": "deterministic-alice-procedure-edit-affordance"
        },
        {
          "target_role": "baseline",
          "affordance": "world_run",
          "decision": "no_go",
          "reason": "missing deterministic desktop affordance for artifact proves the world run control or equivalent runtime entry point executed after the first-lesson edit",
          "missing_affordance_id": "deterministic-alice-world-run-affordance"
        },
        {
          "target_role": "baseline",
          "affordance": "project_save",
          "decision": "no_go",
          "reason": "missing deterministic desktop affordance for saved .a3p project artifact exists, is non-empty, and can be read after the first-lesson run proof",
          "missing_affordance_id": "deterministic-alice-project-save-affordance"
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
        "ui-action-contract.json"
      ],
      "no_go_contracts": [
        {
          "target_role": "baseline",
          "affordance": "object_placement",
          "decision": "no_go",
          "reason": "missing deterministic desktop affordance for artifact proves a named object was added to the scene and placed without coordinate guessing",
          "missing_affordance_id": "deterministic-alice-object-gallery-placement-affordance"
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
      "ui-action-contract.json"
    ],
    "no_go_contracts": [
      {
        "target_role": "baseline",
        "affordance": "object_placement",
        "decision": "no_go",
        "reason": "missing deterministic desktop affordance for artifact proves a named object was added to the scene and placed without coordinate guessing",
        "missing_affordance_id": "deterministic-alice-object-gallery-placement-affordance"
      }
    ]
  }
}
```

Envelope fields:

| Field | Type | Description |
| --- | --- | --- |
| `scenario_id` | string or null | Scenario being checked. |
| `role` | string | Readiness role for the envelope; current reports include `instructor` and `student`. |
| `status` | string | `ready`, `not_ready`, or `blocked`. |
| `blocked_reason` | string or null | Blocker reason when status is `blocked`. |
| `human_summary` | string | Human-readable state summary. |
| `required_evidence` | array of strings | Required durable evidence artifacts. |
| `no_go_contracts` | array | Missing-affordance contracts that prevent silent success. |

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
| `action_assertions` | array | Required action assertions and their pass/fail status. |
| `required_actions` | array of strings | Action ids discovered from the UI action contract. |
| `missing_assertions` | array of strings | Required assertions absent from the target evidence. |
| `missing_required_actions` | array of strings | Required action ids absent from the UI action contract. |
| `no_go_contracts` | array | Target-local no-go contracts. |

Required action ids for the current first-lesson flow are:

```text
verify-specific-alice-window
activate-specific-alice-window
place-object
edit-procedure-or-code-block
run-world
save-project
```

## No-go contract API

No-go contracts make missing desktop affordances explicit. They are aggregated
from UI action precondition probes and required-action entries whose
`decision` is `no_go` or whose `contract_required.unsafe_until_available` flag
is true. They prevent the harness, adapters, and docs from converting
unsupported actions into silent success.

Each no-go entry has this shape:

| Field | Type | Description |
| --- | --- | --- |
| `target_role` | string | Target that reported the blocker, usually `baseline` or `modernized`. |
| `affordance` | string | Stable affordance name such as `object_placement`, `procedure_edit`, `world_run`, or `project_save`. |
| `decision` | string | Always `no_go` for this contract. |
| `reason` | string | Human-readable reason the action cannot be claimed. |
| `missing_affordance_id` | string or null | Specific deterministic affordance that must exist before the action can pass. |

Known missing affordance ids:

| Missing affordance id | Affordance | Meaning |
| --- | --- | --- |
| `deterministic-alice-object-gallery-placement-affordance` | `object_placement` | A stable Alice-side way to place a named gallery object and produce durable evidence is unavailable. |
| `deterministic-alice-procedure-edit-affordance` | `procedure_edit` | A stable Alice-side way to edit a procedure or code block and prove the edit is unavailable. |
| `deterministic-alice-world-run-affordance` | `world_run` | A stable Alice-side way to prove the world ran after student edits is unavailable. |
| `deterministic-alice-project-save-affordance` | `project_save` | A stable Alice-side way to prove a project save artifact is unavailable. |

Consumers must treat `decision: "no_go"` as a blocked action, not as a failed
test to hide and not as a pass to override.

## Configuration

### Environment variables

| Variable | Required for | Description |
| --- | --- | --- |
| `NODE_OPTIONS=--max-old-space-size=32768` | Agentic/Gadugi-heavy local runs | Keeps Node-based runners from failing under large prompt or adapter workloads. |
| `EATME_REAL_ALICE=1` | Non-baseline real Alice execution | Explicit opt-in gate for real desktop runs. |
| `ALICE_HOME` | Single-target Alice commands | Alice checkout for `alice discover`, `alice package`, and `alice launch-smoke`. |
| `ALICE_BASELINE_HOME` | Comparison/readiness sequence | Reference Alice checkout used as the baseline target. |
| `ALICE_MODERNIZED_HOME` | Comparison/readiness sequence | Candidate Alice checkout used as the modernized target. |

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
`ui-action-contract.json` evidence and action-level `no_go_contracts`. It is not
a completed UI automation pass.

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
4. Add explicit no-go language for missing desktop affordances instead of
   implying silent success.
5. Validate the changed asset:

   ```bash
   cargo run -q -p eatme-cli -- assets validate \
     --path assets/scenarios/eatme/<scenario-id>.yaml \
     --json
   ```

6. Check generated adapter freshness:

   ```bash
   cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
   ```

7. If adapters are stale, regenerate them:

   ```bash
   cargo run -q -p eatme-cli -- assets generate-gadugi --json
   ```

8. For student first-lesson changes, run or inspect a readiness report and confirm
   it exposes `status`, `lesson_session_readiness`, and `no_go_contracts`. For
   instructor-only asset changes, keep the evidence boundary in scenario
   validation and generated adapters unless an executable instructor harness is
   added.
9. Run the repository quality gate before handoff:

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
| Readiness output | Student first-lesson reports expose normalized `status` and `lesson_session_readiness`; instructor-only changes do not claim a readiness report unless a harness produces one. |
| No-go contracts | Unsupported desktop affordances are explicit `decision: "no_go"` entries. |
| Boundaries | The change does not claim full UI automation, creative assessment, learner-world grading, complete Alice coverage, or deployed-service status. |
| Quality gate | `./scripts/quality-gates.sh` passed. |
