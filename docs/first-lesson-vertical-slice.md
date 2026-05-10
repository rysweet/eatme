# First-lesson vertical slice

The `first_lesson_vertical_slice` integration test exercises the complete
first-lesson pipeline step by step, validating per-action go/no\_go evidence,
structured reporting, and contract advancement. It runs in three modes: a
fake-toolchain vertical slice with sequential step evidence, a real-Alice gated
vertical slice with screenshot and step-report capture, and a fake-toolchain
variant that proves the contract advances when an object-placement hook is
present.

The test file is
`crates/eatme-alice/tests/first_lesson_vertical_slice.rs`. It uses the
`first-lessons-real-ui-actions` scenario exclusively.

## Contents

- [Usage](#usage)
- [Environment gate](#environment-gate)
- [What the tests prove](#what-the-tests-prove)
- [Per-step evidence model](#per-step-evidence-model)
- [ui-action-contract.json schema](#ui-action-contractjson-schema)
- [API surface](#api-surface)
- [Configuration](#configuration)
- [Examples](#examples)
- [Troubleshooting](#troubleshooting)
- [Related documentation](#related-documentation)

## Usage

Run all three vertical-slice tests (real-Alice test auto-skips when the gate is
absent):

```bash
cargo test -p eatme-alice --test first_lesson_vertical_slice
```

Run only the fake-toolchain tests (no Alice or desktop dependencies required):

```bash
cargo test -p eatme-alice --test first_lesson_vertical_slice \
  -- fake_toolchain
```

Run the real-Alice vertical slice on a self-hosted runner:

```bash
EATME_REAL_ALICE=1 cargo test -p eatme-alice \
  --test first_lesson_vertical_slice \
  -- real_alice_vertical_slice
```

All tests use the `first-lessons-real-ui-actions` scenario id. The test binary
always compiles; runtime gates determine which tests execute.

## Environment gate

| Variable | Required value | Effect |
| --- | --- | --- |
| `EATME_REAL_ALICE` | `1` | Enables `real_alice_vertical_slice_captures_per_step_evidence`. Any other value or absence causes the test to skip. |
| `ALICE_HOME` | Path to Alice checkout | The Alice checkout directory. Defaults to `../alice3-modernization` when not set. |

The two `fake_toolchain_*` tests run unconditionally. They use `PathOverride`
from `launch_smoke_support` to inject fake tools so no real Alice, Java, Maven,
or X11 dependencies are needed.

## What the tests prove

### fake\_toolchain\_vertical\_slice\_reports\_step\_by\_step\_evidence

Runs `run_launch_smoke` with the `first-lessons-real-ui-actions` scenario using
fake tools. Deserializes `ui-action-contract.json` and validates each action's
go/no\_go decision in the expected sequence:

| Step | Expected decision | Reason |
| --- | --- | --- |
| `verify-specific-alice-window` | `go` | Fake wmctrl reports an Alice window. Derived from `executed_action_probes`. |
| `activate-specific-alice-window` | `go` | Fake xdotool activates the detected window. Derived from `executed_action_probes`. |
| `place-object` | `no_go` | No object-placement affordance exists. `missing_affordance.id` is `deterministic-alice-object-gallery-placement-affordance`. Found in `action_precondition_probes`. |
| `edit-procedure-or-code-block` | `blocked` | Blocked by `place-object` no\_go. Inferred from `required_actions` having `no_go` with no corresponding probe. |
| `run-world` | `blocked` | Blocked by `edit-procedure-or-code-block` blocked. |
| `save-project` | `blocked` | Blocked by prior steps. |

The test asserts:

- The manifest `failure_category` is `ui_action_automation_unimplemented`.
- The `ui-action-contract.json` file exists, is valid JSON, and is non-empty.
- The `action_precondition_probes` array contains `place-object` with `no_go`.
- The `executed_action_probes` array contains window verification and activation.
- The `required_actions` array lists all six step ids.

### real\_alice\_vertical\_slice\_captures\_per\_step\_evidence

Gated behind `EATME_REAL_ALICE=1`. Runs a real Alice process with the
`first-lessons-real-ui-actions` scenario under Xvfb. Validates:

- A startup screenshot exists and is a valid PNG (magic-byte check).
- Per-step assertions exist in the manifest for all six action ids.
- `ui-action-contract.json` is written with per-step evidence entries.
- The manifest `scenario_id` equals `first-lessons-real-ui-actions`.

This test does not assert that all steps pass. Real Alice may or may not have
the object-placement affordance wired. The test proves the harness exercises
each step and captures evidence regardless of outcome.

### fake\_toolchain\_vertical\_slice\_advances\_with\_object\_placement\_hook

Uses `write_fake_object_placement_hook()` to simulate an object-placement
backend proof hook, then runs `run_launch_smoke`. Validates that the contract
frontier advances:

| Step | Expected decision | Change from baseline |
| --- | --- | --- |
| `verify-specific-alice-window` | `go` | Unchanged. |
| `activate-specific-alice-window` | `go` | Unchanged. |
| `place-object` | `passed` | **Advanced** — hook provides placement proof. Found in `candidate_affordance_probes`. |
| `edit-procedure-or-code-block` | `no_go` | **New frontier** — `missing_affordance.id` is `deterministic-alice-procedure-edit-affordance`. Found in `action_precondition_probes`. |
| `run-world` | `blocked` | Now blocked by `edit-procedure-or-code-block`. |
| `save-project` | `blocked` | Still blocked by prior steps. |

The test asserts:

- The manifest `failure_category` shifts to
  `ui_action_remaining_steps_unimplemented`.
- `place-object` decision is `passed`, not `no_go`.
- `edit-procedure-or-code-block` is the new `no_go` frontier with the correct
  `missing_affordance.id`.

## Per-step evidence model

The vertical-slice tests validate a sequential, ordered pipeline. Each step in
the `first-lessons-real-ui-actions` scenario has a precondition gate:

```text
verify-specific-alice-window
  └→ activate-specific-alice-window
       └→ place-object
            └→ edit-procedure-or-code-block
                 └→ run-world
                      └→ save-project
```

A step produces one of four decisions:

| Decision | Meaning |
| --- | --- |
| `go` | The step succeeded. The next step may proceed. |
| `passed` | The step succeeded via a backend proof hook (not interactive UI). |
| `no_go` | The step cannot proceed. A `missing_affordance` identifies what is needed. This is the **contract frontier**. |
| `blocked` | The step is unreachable because a prior step is `no_go` or `blocked`. |

Only one step in the pipeline can be `no_go` at a time — it is the frontier.
All steps after the frontier are `blocked`. Steps before the frontier are `go`
or `passed`.

The `assert_step_evidence` helper enforces this invariant by walking the
expected step order and checking that decisions follow the
go\* → no\_go → blocked\* pattern.

## ui-action-contract.json schema

The `ui-action-contract.json` artifact is written to the scenario run directory
alongside the manifest by `write_ui_action_contract` in
`crates/eatme-alice/src/launch_ui_action_contract.rs`. It records per-action
evidence for the vertical slice. The `failure_category` field is in the
**manifest**, not in this contract file.

```json
{
  "schema_version": "eatme.ui-action-contract/v1",
  "status": "blocked",
  "blocking_reason": "The harness can activate a detected Alice window ...",
  "preflight_evidence": {
    "specific_alice_window_detected": true,
    "visual_evidence_captured": true,
    "log_captured": true
  },
  "executed_action_probes": [
    { "id": "verify-specific-alice-window", "status": "passed", "detail": "..." },
    { "id": "activate-specific-alice-window", "status": "passed", "detail": "..." }
  ],
  "action_precondition_probes": [
    {
      "action_id": "place-object",
      "decision": "no_go",
      "missing_affordance": {
        "id": "deterministic-alice-object-gallery-placement-affordance",
        "next_implementation": "named gallery selector ..."
      }
    }
  ],
  "candidate_affordance_probes": [],
  "required_actions": [
    {
      "id": "place-object",
      "decision": "no_go",
      "required_evidence": "artifact proves a named object was added ...",
      "missing_affordance_id": "deterministic-alice-object-gallery-placement-affordance",
      "contract_required": { "candidate_backend": "eatme-place-object", "..." : "..." }
    }
  ]
}
```

| Field | Type | Meaning |
| --- | --- | --- |
| `schema_version` | string | Always `eatme.ui-action-contract/v1`. |
| `status` | string | Always `blocked` while any required action is unproven. |
| `blocking_reason` | string | Human-readable explanation of what is missing. |
| `preflight_evidence` | object | Booleans for window detection, visual capture, and log capture. |
| `executed_action_probes` | array | Probes for actions that were executed (window verification, activation, shortcut dispatch). Each has `id`, `status`, `detail`, `window_id`, `command`, `exit_status`, `stdout`, and `stderr`. The `status` field is `"passed"` when the probe succeeded or `"blocked"` when it failed. |
| `action_precondition_probes` | array | No-go probes for actions whose preconditions are unmet. Each has `id` (probe id), `action_id` (action being tested), `status` (`"blocked"`), `decision` (`"no_go"`), `blocking_reason`, `required_evidence`, `missing_affordance`, and `preconditions`. |
| `action_precondition_probes[].missing_affordance.id` | string | Machine-readable affordance identifier. |
| `candidate_affordance_probes` | array | Probes for backend hooks that proved an action (object placement, edit, run, save). |
| `required_actions` | array | Full roster of required actions with `id`, `decision` (`no_go` or `ready`), `required_evidence`, and `contract_required` backend specification. |

The `assert_step_evidence` helper in the test interprets this structure to
derive the per-step `go` / `passed` / `no_go` / `blocked` decisions described
in the [per-step evidence model](#per-step-evidence-model). It maps
`executed_action_probes` entries with `status: "passed"` to the `go` decision,
`candidate_affordance_probes` entries to the `passed` decision,
`action_precondition_probes` entries to `no_go`, and infers `blocked` for steps
that appear only in `required_actions` with `decision: "no_go"` and no
corresponding probe.

The manifest `failure_category` (not in this JSON) is
`ui_action_automation_unimplemented` when `place-object` is the frontier, or
`ui_action_remaining_steps_unimplemented` when a later step is the frontier.

## API surface

The vertical-slice tests use the same public API as the existing launch-smoke
tests:

```rust
use eatme_alice::{LaunchSmokeOptions, LaunchSmokeScenario, run_launch_smoke};
```

| Type | Crate | Purpose |
| --- | --- | --- |
| `run_launch_smoke(options)` | `eatme-alice` | Runs the full launch smoke pipeline and returns a `LaunchSmokeManifest`. |
| `LaunchSmokeOptions` | `eatme-alice` | Configuration for Alice home, run id, runs directory, timeout, scenario, and packaging options. |
| `LaunchSmokeScenario` | `eatme-alice` | Identifies the scenario by id and starter project path. |
| `LaunchSmokeManifest` | `eatme-core` | Evidence manifest containing assertions, artifacts, failure category, and launch metadata. |

Test-only helpers from `launch_smoke_support`:

| Helper | Purpose |
| --- | --- |
| `TestFixture::new()` | Creates an isolated test work directory with nonce-based paths under `target/test-work/`. |
| `PathOverride` | Injects fake tool paths so tests run without real desktop dependencies. |
| `write_fake_object_placement_hook()` | Writes a fake backend proof hook that causes `place-object` to advance to `passed`. |

No new public API is introduced. The test file is an integration test that
consumes existing crate APIs.

## Configuration

### Fake-toolchain test options

| Option | Value | Rationale |
| --- | --- | --- |
| `scenario` | `first-lessons-real-ui-actions` | Exercises the first-lesson UI-action pipeline. |
| `run_id` | `vertical-slice-fake` | Identifies the evidence directory. |
| `runs_dir` | `fixture.root.join("runs")` | Nonce-based under `target/test-work/launch-smoke/` via `TestFixture`. |
| `timeout_seconds` | `120` | Fake tools complete quickly; the timeout is a safety bound. |
| `json` | `true` | Machine-readable output for assertion parsing. |
| `no_memory` | `true` | No persistent memory side effects. |

### Real-Alice test options

| Option | Value | Rationale |
| --- | --- | --- |
| `alice_home` | `ALICE_HOME` env var or `../alice3-modernization` | Standard Alice checkout location. |
| `scenario` | `first-lessons-real-ui-actions` | Same scenario as the fake tests. |
| `run_id` | `vertical-slice-real` | Identifies the evidence directory. |
| `runs_dir` | `target/test-work/vertical-slice-real/runs` | Isolated under `target/`. |
| `timeout_seconds` | `900` | 15-minute timeout for Maven builds and Java startup. |
| `json` | `true` | Machine-readable output. |
| `no_memory` | `true` | No persistent memory side effects. |
| `offline_package` | `true` | Uses cached Maven dependencies, no network access. |

### Host requirements for real-Alice test

The real-Alice vertical slice requires the same host dependencies as
[Deterministic Real-Alice Smoke Test](deterministic-real-alice-smoke-test.md#host-requirements).

## Examples

### Run the fake-toolchain vertical slice and inspect the contract

```bash
cargo test -p eatme-alice --test first_lesson_vertical_slice \
  -- fake_toolchain_vertical_slice_reports --nocapture
```

After the test passes, inspect the no-go probes in the generated contract.
Because `TestFixture` uses nonce-based paths, locate the contract first:

```bash
CONTRACT=$(find target/test-work/launch-smoke -name ui-action-contract.json \
  -path '*/first-lessons-real-ui-actions/vertical-slice-fake/*' | head -1)
cat "$CONTRACT" | jq '.action_precondition_probes[] | {action_id, decision}'
```

Expected output (only the no-go frontier appears):

```json
{"action_id": "place-object", "decision": "no_go"}
```

Inspect the required actions roster:

```bash
cat "$CONTRACT" | jq '.required_actions[] | {id, decision}'
```

Expected output:

```json
{"id": "verify-specific-alice-window", "decision": null}
{"id": "activate-specific-alice-window", "decision": null}
{"id": "place-object", "decision": "no_go"}
{"id": "edit-procedure-or-code-block", "decision": "no_go"}
{"id": "run-world", "decision": "no_go"}
{"id": "save-project", "decision": "no_go"}
```

### Run the object-placement advancement test

```bash
cargo test -p eatme-alice --test first_lesson_vertical_slice \
  -- fake_toolchain_vertical_slice_advances --nocapture
```

Inspect the shifted frontier in the manifest and contract:

```bash
MANIFEST=$(find target/test-work/launch-smoke -name manifest.json \
  -path '*/first-lessons-real-ui-actions/vertical-slice-fake-advanced/*' | head -1)
cat "$MANIFEST" | jq '.failure_category'
```

Expected output:

```json
"ui_action_remaining_steps_unimplemented"
```

```bash
ADV_CONTRACT=$(find target/test-work/launch-smoke -name ui-action-contract.json \
  -path '*/first-lessons-real-ui-actions/vertical-slice-fake-advanced/*' | head -1)
cat "$ADV_CONTRACT" | jq '.action_precondition_probes[] | {action_id, decision}'
```

Expected output (frontier has shifted to edit):

```json
{"action_id": "edit-procedure-or-code-block", "decision": "no_go"}
```

### Run the real-Alice vertical slice on a self-hosted runner

```bash
export ALICE_HOME=/opt/alice3-modernization
TMPDIR=/tmp EATME_REAL_ALICE=1 cargo test -p eatme-alice \
  --test first_lesson_vertical_slice \
  -- real_alice_vertical_slice --nocapture
```

Inspect per-step evidence:

```bash
cat target/test-work/vertical-slice-real/runs/first-lessons-real-ui-actions/vertical-slice-real/ui-action-contract.json \
  | jq '{executed: [.executed_action_probes[] | .id], no_go: [.action_precondition_probes[] | .action_id]}'
```

### Verify screenshot captured during real-Alice run

```bash
file target/test-work/vertical-slice-real/runs/first-lessons-real-ui-actions/vertical-slice-real/screenshots/startup.png
```

Expected output:

```text
.../startup.png: PNG image data, 1024 x 768, 8-bit/color RGB, non-interlaced
```

## Troubleshooting

### Fake-toolchain tests fail with "scenario not found"

The test uses `first-lessons-real-ui-actions` as the scenario id. Verify the
scenario asset exists:

```bash
cargo run -q -p eatme-cli -- assets validate --json \
  | jq '.scenarios[] | select(.id == "first-lessons-real-ui-actions")'
```

### Contract JSON is missing or empty

The `ui-action-contract.json` is written by `write_ui_action_contract` in
`crates/eatme-alice/src/launch_ui_action_contract.rs`. If the file is missing,
the launch-smoke pipeline did not reach the UI-action contract phase. Check the
manifest `failure_category` for an earlier failure (note: `failure_category` is
in the manifest, not in the contract JSON):

```bash
MANIFEST=$(find target/test-work/launch-smoke -name manifest.json \
  -path '*/first-lessons-real-ui-actions/vertical-slice-fake/*' | head -1)
cat "$MANIFEST" | jq '.failure_category'
```

### Real-Alice test skips unexpectedly

Verify the environment variable is set to exactly `1`:

```bash
echo $EATME_REAL_ALICE   # should print: 1
```

The check is `std::env::var("EATME_REAL_ALICE") == Ok("1".into())`. Values
like `true`, `yes`, or empty string do not activate the test.

### Unix socket path too long

In deep worktree paths, the X display socket path may exceed the 108-character
Unix socket limit. Use `TMPDIR=/tmp`:

```bash
TMPDIR=/tmp EATME_REAL_ALICE=1 cargo test -p eatme-alice \
  --test first_lesson_vertical_slice
```

### Object-placement hook test fails with unexpected decision

The `write_fake_object_placement_hook()` helper must write a proof file that
the harness recognizes. If `place-object` remains `no_go` after the hook is
written, verify the hook output path matches the harness expectation by
checking `launch_smoke_support::write_fake_object_placement_hook`.

### 500-line module limit

The test file targets approximately 400 lines with 3 tests and 1 helper. If
the quality gate reports a line-count violation, split test helpers into a
shared support module. The existing pattern is
`crates/eatme-alice/tests/launch_smoke_support.rs`.

## Related documentation

- [Deterministic Real-Alice Smoke Test](deterministic-real-alice-smoke-test.md) —
  The baseline real-Alice integration test that this vertical slice extends.
- [First-Lesson Evidence Readiness](first-lesson-evidence-readiness.md) —
  Readiness report that consumes the evidence produced by these tests.
- [Evidence Artifact Contract](evidence-artifact-contract.md) — Schema and
  validation rules for evidence artifacts including `ui-action-contract.json`.
- [Alice Lesson Smoke](alice-lesson-smoke.md) — Desktop scenario roster and
  evidence contracts.
- [Scenario Authoring](scenario-authoring.md) — How to author and modify
  scenario YAML files like `first-lessons-real-ui-actions`.
- [Validation and Quality Gates](validation-quality-gates.md) — Repository
  quality gates including the 500-line module limit.
- [Outside-in Alice Test Modules](outside-in-alice-test-modules.md) — Rust
  test module layout and authoring workflow.
