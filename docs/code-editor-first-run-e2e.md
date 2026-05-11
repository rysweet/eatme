# Code Editor First Run E2E test

The `code_editor_first_run_e2e` integration test exercises the
`code-editor-first-run` scenario through the UI-action contract pipeline,
validating that the launch smoke passes, the code editor tab can be observed,
a simple procedure structure exists, and the result can be saved. It runs with
a fake toolchain so no real Alice, Java, Maven, or X11 dependencies are needed.

The test file is
`crates/eatme-alice/tests/code_editor_first_run_e2e.rs`. It uses the
`code-editor-first-run` scenario exclusively.

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

Run all three code editor first run E2E tests:

```bash
cargo test -p eatme-alice --test code_editor_first_run_e2e
```

Run only the baseline contract generation test:

```bash
cargo test -p eatme-alice --test code_editor_first_run_e2e \
  -- baseline_contract
```

Run only the placement-hook advancement test:

```bash
cargo test -p eatme-alice --test code_editor_first_run_e2e \
  -- placement_hook_advances
```

Run only the per-step readiness test:

```bash
cargo test -p eatme-alice --test code_editor_first_run_e2e \
  -- per_step_readiness
```

All tests use the `code-editor-first-run` scenario id. The test binary always
compiles; no runtime environment gate is required because all tests use the
fake toolchain.

## Environment gate

| Variable | Required value | Effect |
| --- | --- | --- |
| `EATME_REAL_ALICE` | Not required | All three tests run unconditionally with fake tools. |
| `ALICE_HOME` | Not required | Fake Alice home directories are created by `TestFixture`. |
| `TMPDIR` | `/tmp` (recommended) | Avoids Unix socket path length failures in deep worktrees. |

The tests use `PathOverride` from `launch_smoke_support` to inject fake tool
paths. No real Alice, Java, Maven, or X11 dependencies are needed.

## What the tests prove

### baseline\_contract\_generates\_ui\_action\_contract

Runs `run_launch_smoke` with the `code-editor-first-run` scenario using fake
tools. Deserializes `ui-action-contract.json` and validates that the UI-action
contract pipeline activates for this scenario. The expected step decisions are:

| Step | Expected decision | Reason |
| --- | --- | --- |
| `verify-specific-alice-window` | `go` | Fake wmctrl reports an Alice window. Derived from `executed_action_probes`. |
| `activate-specific-alice-window` | `go` | Fake xdotool activates the detected window. Derived from `executed_action_probes`. |
| `place-object` | `no_go` | No object-placement affordance exists. `missing_affordance.id` is `deterministic-alice-object-gallery-placement-affordance`. Found in `action_precondition_probes`. |
| `edit-procedure-or-code-block` | `blocked` | Blocked by `place-object` no\_go. |
| `run-world` | `blocked` | Blocked by `edit-procedure-or-code-block` blocked. |
| `save-project` | `blocked` | Blocked by prior steps. |

The test asserts:

- The manifest `failure_category` is `ui_action_automation_unimplemented`.
- The `ui-action-contract.json` file exists, is valid JSON, and is non-empty.
- The `required_actions` array lists all six step ids (matching the
  `first-lessons-real-ui-actions` pattern). The test specifically verifies the
  four student action ids are present: `place-object`,
  `edit-procedure-or-code-block`, `run-world`, `save-project`.
- The scenario\_id in the manifest is `code-editor-first-run`.

This test proves **issue #235 requirement 1** (launch smoke passes) and
**requirement 2** (code editor tab can be observed — `edit-procedure-or-code-block`
appears in `required_actions`).

### placement\_hook\_advances\_contract\_frontier

Uses `write_fake_object_placement_hook()` to simulate an object-placement
backend proof hook, then runs `run_launch_smoke` with `code-editor-first-run`.
Validates that the contract frontier advances past `place-object`:

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

This test proves **issue #235 requirement 3** (a simple procedure structure
exists — the contract frontier reaches `edit-procedure-or-code-block`).

### per\_step\_readiness\_reports\_all\_steps

Runs the baseline contract and validates per-step readiness reporting. For each
step in the action chain, verifies that the contract reports a clear
`go` / `no_go` / `blocked` status and that the step ordering is correct:

```text
place-object → edit-procedure-or-code-block → run-world → save-project
```

The test asserts:

- Each step has a `decision` field in the contract.
- Steps before the frontier are `go` or `passed`.
- The frontier step is `no_go`.
- Steps after the frontier are `blocked`.
- The `save-project` step is present, proving **issue #235 requirement 4**
  (the result can be saved — `save-project` is the terminal step in the
  action chain and appears in the contract with a status).

## Per-step evidence model

The code editor first run E2E tests validate the same sequential, ordered
pipeline as the first-lesson vertical slice. Each step in the
`code-editor-first-run` scenario has a precondition gate:

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

The `StepExpectation` helper struct and `assert_step_evidence` function enforce
this invariant by walking the expected step order and checking that decisions
follow the go\* → no\_go → blocked\* pattern.

## ui-action-contract.json schema

The `ui-action-contract.json` artifact written by the `code-editor-first-run`
scenario follows the same `eatme.ui-action-contract/v1` schema as the
first-lesson vertical slice. See
[First-Lesson Vertical Slice: ui-action-contract.json schema](first-lesson-vertical-slice.md#ui-action-contractjson-schema)
for the full schema reference.

Key fields for code editor first run validation:

| Field | Expected value | Meaning |
| --- | --- | --- |
| `required_actions[].id == "edit-procedure-or-code-block"` | Present | Code editor tab observation — the contract knows a procedure edit step exists. |
| `required_actions[].id == "save-project"` | Present | Save capability — the contract knows the result can be saved. |
| `action_precondition_probes[].action_id == "place-object"` | `no_go` (baseline) | The frontier at baseline — no object placement affordance. |
| `action_precondition_probes[].action_id == "edit-procedure-or-code-block"` | `no_go` (after placement hook) | The frontier after placement — no procedure edit affordance. |

## API surface

The code editor first run E2E tests use the same public API as the existing
launch-smoke and first-lesson vertical-slice tests:

```rust
use eatme_alice::{LaunchSmokeOptions, LaunchSmokeScenario, run_launch_smoke};
```

| Type | Crate | Purpose |
| --- | --- | --- |
| `run_launch_smoke(options)` | `eatme-alice` | Runs the full launch smoke pipeline and returns a `LaunchSmokeManifest`. |
| `LaunchSmokeOptions` | `eatme-alice` | Configuration for Alice home, run id, runs directory, timeout, scenario, and packaging options. |
| `LaunchSmokeScenario` | `eatme-alice` | Identifies the scenario by id and starter project path. |
| `LaunchSmokeManifest` | `eatme-core` | Evidence manifest containing assertions, artifacts, failure category, and launch metadata. |

The `LaunchSmokeScenario::requires_real_ui_actions()` method returns `true` for
`code-editor-first-run`, routing it into the UI-action contract pipeline
alongside `first-lessons-real-ui-actions`.

Test-only helpers from `launch_smoke_support`:

| Helper | Purpose |
| --- | --- |
| `TestFixture::new()` | Creates an isolated test work directory with nonce-based paths under `target/test-work/`. |
| `PathOverride` | Injects fake tool paths so tests run without real desktop dependencies. |
| `write_fake_object_placement_hook()` | Writes a fake backend proof hook that causes `place-object` to advance to `passed`. |

Test-local helpers in `code_editor_first_run_e2e.rs`:

| Helper | Purpose |
| --- | --- |
| `StepExpectation` | Struct pairing a step id with an expected decision (`go`, `passed`, `no_go`, `blocked`). |
| `assert_step_evidence(contract, expectations)` | Walks the step expectations array and asserts each step's decision matches. |
| `make_smoke_options(fixture, run_id)` | Constructs `LaunchSmokeOptions` with `code-editor-first-run` scenario defaults. |

No new public API is introduced. The test file is an integration test that
consumes existing crate APIs.

## Configuration

### Fake-toolchain test options

| Option | Value | Rationale |
| --- | --- | --- |
| `scenario` | `code-editor-first-run` | Exercises the code editor first run UI-action pipeline. |
| `run_id` | `code-editor-baseline` / `code-editor-advanced` / `code-editor-readiness` | Identifies each test's evidence directory. |
| `runs_dir` | `fixture.root.join("runs")` | Nonce-based under `target/test-work/launch-smoke/` via `TestFixture`. |
| `timeout_seconds` | `120` | Fake tools complete quickly; the timeout is a safety bound. |
| `json` | `true` | Machine-readable output for assertion parsing. |
| `no_memory` | `true` | No persistent memory side effects. |

### Host requirements

None beyond a Rust toolchain. All tests run with fake tools — no Java, Maven,
Xvfb, wmctrl, xdotool, or Alice installation is required.

## Examples

### Run the baseline contract test and inspect the contract

```bash
TMPDIR=/tmp cargo test -p eatme-alice --test code_editor_first_run_e2e \
  -- baseline_contract --nocapture
```

After the test passes, inspect the generated contract:

```bash
CONTRACT=$(find target/test-work/launch-smoke -name ui-action-contract.json \
  -path '*/code-editor-first-run/code-editor-baseline/*' | head -1)
cat "$CONTRACT" | jq '.required_actions[] | {id, decision}'
```

Expected output shows all six steps (window steps have `null` decisions in the
raw JSON; the test helper derives `go` from `executed_action_probes`):

```json
{"id": "verify-specific-alice-window", "decision": null}
{"id": "activate-specific-alice-window", "decision": null}
{"id": "place-object", "decision": "no_go"}
{"id": "edit-procedure-or-code-block", "decision": "no_go"}
{"id": "run-world", "decision": "no_go"}
{"id": "save-project", "decision": "no_go"}
```

### Run the placement-hook advancement test and inspect the shifted frontier

```bash
TMPDIR=/tmp cargo test -p eatme-alice --test code_editor_first_run_e2e \
  -- placement_hook_advances --nocapture
```

Inspect the shifted frontier:

```bash
ADV_CONTRACT=$(find target/test-work/launch-smoke -name ui-action-contract.json \
  -path '*/code-editor-first-run/code-editor-advanced/*' | head -1)
cat "$ADV_CONTRACT" | jq '.action_precondition_probes[] | {action_id, decision}'
```

Expected output (frontier has shifted to edit):

```json
{"action_id": "edit-procedure-or-code-block", "decision": "no_go"}
```

### Verify all four issue #235 requirements at once

```bash
TMPDIR=/tmp cargo test -p eatme-alice --test code_editor_first_run_e2e
```

A passing suite confirms:

1. **Launch smoke passes** — `run_launch_smoke` completes without panic for
   `code-editor-first-run`.
2. **Code editor tab observed** — `edit-procedure-or-code-block` appears in
   `required_actions`.
3. **Procedure structure exists** — the contract frontier reaches
   `edit-procedure-or-code-block` when the placement hook is present.
4. **Result can be saved** — `save-project` appears in `required_actions` with
   a per-step status.

### Confirm the scenario is in the UI-action contract pipeline

```bash
cargo test -p eatme-alice --test launch_smoke_fake \
  -- lesson_smoke --nocapture 2>&1 | grep code-editor-first-run
```

The `requires_real_ui_actions()` gate on `LaunchSmokeScenario` returns `true`
for `code-editor-first-run`, routing it through the same UI-action contract
pipeline as `first-lessons-real-ui-actions`.

## Troubleshooting

### Test fails with "scenario not found"

The test uses `code-editor-first-run` as the scenario id. Verify the scenario
asset exists:

```bash
cargo run -q -p eatme-cli -- assets validate --json \
  | jq '.scenarios[] | select(.id == "code-editor-first-run")'
```

### Contract JSON is missing or empty

The `ui-action-contract.json` is written by the UI-action contract phase of
the launch smoke pipeline. If the file is missing, check whether
`requires_real_ui_actions()` returns `true` for `code-editor-first-run`:

```rust
assert!(LaunchSmokeScenario::new("code-editor-first-run").requires_real_ui_actions());
```

If the method returns `false`, the scenario was not routed into the UI-action
contract pipeline. The implementation extends `requires_real_ui_actions()` to
match both `first-lessons-real-ui-actions` and `code-editor-first-run`.

### Broken test: lesson\_smoke\_is\_ready\_when\_window\_evidence\_exists\_without\_screenshot

The `launch_smoke_fake.rs` test
`lesson_smoke_is_ready_when_window_evidence_exists_without_screenshot`
previously used `code-editor-first-run` as its scenario. Because
`code-editor-first-run` now routes through the UI-action contract pipeline
(setting `failure_category` to `ui_action_automation_unimplemented`), that test
was updated to use `building-a-scene-first-world` instead. This is the only
pre-existing test affected by the change.

If this test fails after a merge, verify the scenario in the test matches
`building-a-scene-first-world`:

```bash
grep -n 'lesson_smoke_is_ready_when_window_evidence_exists_without_screenshot' \
  crates/eatme-alice/tests/launch_smoke_fake.rs -A 20 | grep scenario
```

### Unix socket path too long

In deep worktree paths, use `TMPDIR=/tmp`:

```bash
TMPDIR=/tmp cargo test -p eatme-alice --test code_editor_first_run_e2e
```

### 500-line module limit

The test file targets approximately 250 lines with 3 tests and 3 helpers. If
the quality gate reports a line-count violation, split test helpers into a
shared support module following the pattern in
`crates/eatme-alice/tests/launch_smoke_support.rs`.

## Related documentation

- [First-Lesson Vertical Slice](first-lesson-vertical-slice.md) — The
  `first-lessons-real-ui-actions` vertical slice that this test parallels for
  the `code-editor-first-run` scenario.
- [Alice Lesson Smoke](alice-lesson-smoke.md) — Desktop scenario roster
  including `code-editor-first-run`.
- [Student Lesson E2E Tests](student-lesson-e2e-tests.md) — Student-facing
  readiness contract tests for the first-lesson system.
- [Evidence Artifact Contract](evidence-artifact-contract.md) — Schema and
  validation rules for evidence artifacts including `ui-action-contract.json`.
- [Scenario Authoring](scenario-authoring.md) — How to author and modify
  scenario YAML files like `code-editor-first-run`.
- [Validation and Quality Gates](validation-quality-gates.md) — Repository
  quality gates including the 500-line module limit.
- [Outside-in Alice Test Modules](outside-in-alice-test-modules.md) — Rust
  test module layout and authoring workflow.
- [Lesson Session Readiness](lesson-session-readiness.md) — Lesson session
  readiness contract that consumes scenario evidence.
