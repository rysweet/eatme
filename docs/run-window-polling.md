# Run window polling after toolbar dispatch

After the eatme harness dispatches a Run toolbar button click via `xdotool`,
a **post-click window detection step** polls `wmctrl` for up to 10 seconds
looking for a new Alice window whose window ID differs from the main window.
This replaces the previous fixed 2-second sleep followed by a single-shot
`wmctrl` check, which always failed because the Run window had not yet
appeared within that narrow timing window.

The polling logic lives in a dedicated module
`crates/eatme-alice/src/launch_run_window_poll.rs`, keeping
`launch_run_window.rs` (464 lines) under the 500-line quality gate.

## Contents

- [Usage](#usage)
- [Pipeline position](#pipeline-position)
- [Polling behavior](#polling-behavior)
- [Blocked cascade](#blocked-cascade)
- [Manifest fields](#manifest-fields)
- [Assertion key](#assertion-key)
- [API surface](#api-surface)
- [Configuration](#configuration)
- [Examples](#examples)
- [Troubleshooting](#troubleshooting)
- [Related documentation](#related-documentation)

## Usage

The run window polling step runs automatically when the toolbar dispatch
probe passes. No additional flags are needed.

Run the first-lesson vertical slice (fake tools, CI-safe):

```bash
cargo test -p eatme-alice --test first_lesson_vertical_slice \
  -- fake_toolchain_vertical_slice_reports --nocapture
```

Run the real-Alice smoke test with polling:

```bash
TMPDIR=/tmp EATME_REAL_ALICE=1 cargo test -p eatme-alice --test launch_smoke_real -- --nocapture
```

Run via CLI:

```bash
EATME_REAL_ALICE=1 cargo run -q -p eatme-cli -- alice launch-smoke \
  --alice-home "${ALICE_HOME}" \
  --scenario first-lessons-real-ui-actions \
  --run-id local-first-lesson \
  --runs-dir runs \
  --json \
  --no-memory \
  --offline-package
```

After the run, check the assertion in the manifest:

```bash
cat runs/first-lessons-real-ui-actions/local-first-lesson/manifest.json \
  | jq '.assertions.run_world_desktop_toolbar_window_observed'
```

## Pipeline position

The polling step replaces the previous 2-second sleep inside
`probe_run_window_after_toolbar_button`. It sits between the toolbar click
dispatch and the desktop run execution sentinel wait:

```text
 1. Dependencies check
 2. Alice discovery + packaging
 3. Xvfb display start
 4. Alice launch + process wait
 5. Window list capture + Alice window search
 6. Startup screenshot
 7. Window activation (wmctrl -ia / xdotool windowfocus)
 8. Post-focus screenshot
 9. UI-action contract probes:
    a. Place-object probe
    b. Edit-procedure probe
    c. Run-world shortcut dispatch (Ctrl+F5) + window observation
    d. Run toolbar button dispatch (xdotool click)
    e. ► Run window poll (wmctrl -lx, up to 10s, 500ms interval)   ← THIS
    f. Desktop run execution sentinel wait (up to 20s)
10. Save-project probe
11. Manifest build + write
```

The shortcut path (`probe_run_window_after_shortcut`) is not modified.
Only the toolbar fallback path uses the new polling logic.

## Polling behavior

The poller executes `wmctrl -lx` every 500 ms for up to 10 seconds
(wall-clock deadline). On each iteration it:

1. Runs `wmctrl -lx` with a 2-second per-command timeout.
2. Scans each output line for Alice Run window evidence using the
   `line_is_alice_run_window()` heuristic (same logic as the existing
   `has_run_window_evidence()`, extracted to per-line granularity).
3. Extracts the `0x`-prefixed hex window ID from any matching line
   (first whitespace-delimited token starting with `0x`, same approach
   as `launch_window_targeting::window_id()`).
4. Compares the extracted window ID against the **main** Alice window ID
   (from `toolbar_probe.window_id`). A match is excluded — the poller
   is looking for a *new* window, not the main one.
5. If a new Run window is found, returns immediately with the new window's
   ID. The probe records `run_window_observed=true`.
6. If `toolbar_probe.window_id` is `None`, any matching Run window line is
   accepted (no exclusion possible).

### Sentinel fast-path

Before entering the polling loop, `probe_run_window_after_toolbar_button`
(in `launch_run_window.rs`) checks once for the
`run-window-evidence/run-window-created.json` sentinel file. If the
sentinel is present and valid, the probe passes immediately without
calling `poll_for_run_window()`. This preserves the existing fast-path
for scenarios where Alice's RabbitHole integration writes the sentinel
before any `wmctrl` output becomes visible. The sentinel check stays in
`launch_run_window.rs`, not in the new polling module.

A screenshot is captured on the sentinel fast-path, consistent with the
existing behavior.

### Timing

| Parameter | Value | Rationale |
| --- | --- | --- |
| Total polling duration | 10 seconds | Generous enough for Alice to create the Run window in Xvfb. |
| Poll interval | 500 ms | Balances responsiveness against CPU overhead. |
| Per-command timeout | 2 seconds | Prevents a hung `wmctrl` from consuming the entire deadline. |
| Maximum poll iterations | ~20 | 10 000 ms ÷ 500 ms. Actual count depends on `wmctrl` latency. |

The poller uses `std::time::Instant` for wall-clock deadline enforcement.
If a `wmctrl` invocation takes longer than 500 ms, the next poll starts
immediately (no negative sleep). If the deadline expires mid-poll, the
result of the in-flight `wmctrl` is still evaluated before returning.

### Result types

The polling function returns a `RunWindowPollResult` enum:

| Variant | Meaning |
| --- | --- |
| `Found { window_id, poll_count, elapsed }` | A new Alice Run window was detected. `window_id` is the `0x`-prefixed hex ID of the *new* window (not the main window). |
| `NotFound { poll_count, elapsed, excluded_main_id }` | No new Run window appeared within the deadline. `excluded_main_id` records which window ID was excluded from matching. |

## Blocked cascade

The run window observation follows the same blocked cascade as before:

| Step | Depends on | Blocked when |
| --- | --- | --- |
| `dispatch-run-toolbar-button` | Window activation + geometry validation | Activation failed, geometry mismatch, or Run window already opened via shortcut. |
| `observe-run-window-after-toolbar-button` | Toolbar dispatch | Toolbar dispatch did not pass. |
| `observe-desktop-run-execution-after-toolbar-button` | Run window observation | Run window was not observed. |

When the toolbar dispatch is blocked or fails, the polling step is skipped
and the probe records `status: "blocked"` with a detail explaining the
dependency.

## Manifest fields

No new manifest fields are added. The existing
`run_world_desktop_toolbar_window_observed` assertion now records richer
detail strings that include polling statistics:

### Passed probe detail

```text
observed a new Alice Run window 0x600042 (distinct from main window 0x600007) after Run toolbar click via wmctrl polling (found on poll 3 of 20, elapsed 1.5s); this indicates desktop Run window opening, not world completion
```

### Failed probe detail

```text
Run toolbar click succeeded, but no new Alice Run window was observed after 10s of wmctrl polling (20 polls, excluded main window 0x600007)
```

### Blocked probe detail

```text
blocked: desktop Run toolbar dispatch must pass before Run window observation
```

## Assertion key

| Assertion key | Meaning | Passes when |
| --- | --- | --- |
| `run_world_desktop_toolbar_window_observed` | A new Run window was detected after toolbar click. | The poller found a new window ID distinct from the main window, **or** the sentinel fast-path succeeded. |

The assertion is only inserted when the toolbar dispatch probe passes.
When the toolbar probe is blocked (e.g. the shortcut path already
opened the Run window), neither the toolbar dispatch assertion nor the
window observation assertion is inserted.

## API surface

### New module: `launch_run_window_poll`

Located at `crates/eatme-alice/src/launch_run_window_poll.rs`.
Registered in `lib.rs` as `mod launch_run_window_poll`.

| Item | Visibility | Purpose |
| --- | --- | --- |
| `RunWindowPollResult` | `pub(crate)` | Enum with `Found` and `NotFound` variants. |
| `poll_for_run_window(runner: &impl CommandRunner, display: &str, main_window_id: Option<&str>) -> RunWindowPollResult` | `pub(crate)` | Entry point. Runs the 10s polling loop. `main_window_id` is `toolbar_probe.window_id.as_deref()` passed by the caller. |
| `find_new_run_window(wmctrl_output: &str, main_window_id: Option<&str>) -> Option<String>` | private | Scans `wmctrl -lx` output for a new Run window, excluding `main_window_id` when `Some`. When `None`, accepts any matching Run window. |
| `line_is_alice_run_window(line: &str) -> bool` | `pub(crate)` | Per-line heuristic extracted from `has_run_window_evidence()`. Returns `true` when the line contains (case-insensitive) (`" run"` or `"\"run"`) AND `"org.alice"` AND NOT `"firefox"`. Used by `launch_run_window::has_run_window_evidence()`. |

### Modified module: `launch_run_window`

| Change | Description |
| --- | --- |
| Removed `std::thread::sleep(Duration::from_secs(2))` | The 2-second sleep in `probe_run_window_after_toolbar_button` is removed. The first poll is immediate. |
| Added `use crate::launch_run_window_poll` | Imports the polling module. |
| Delegation to poller | After the sentinel fast-path check, `probe_run_window_after_toolbar_button` calls `poll_for_run_window()` and maps the result to a `UiActionProbe`. This replaces the `capture_window_text()` + `has_run_window_evidence()` call chain on the toolbar path. Those functions remain in `launch_run_window.rs` for the shortcut path. |
| Passed probe gets new window ID (polling path) | When polling succeeds, the probe's `window_id` is set to the **new** Run window ID (from the poller), not the main window ID. This is a behavior change from the previous code which always echoed `toolbar_probe.window_id`. On the sentinel fast-path, the probe still uses `toolbar_probe.window_id` (unchanged). |
| Failed probe includes statistics | The detail string includes poll count, elapsed time, and the excluded main window ID. |

### Unchanged: shortcut path

`probe_run_window_after_shortcut` retains its existing 2-second sleep and
single-shot `wmctrl` check. Only the toolbar fallback path is modified.

## Configuration

No new configuration options are required. The polling parameters
(10-second deadline, 500 ms interval, 2-second per-command timeout) are
compile-time constants in `launch_run_window_poll.rs`:

```rust
const POLL_DEADLINE: Duration = Duration::from_secs(10);
const POLL_INTERVAL: Duration = Duration::from_millis(500);
const WMCTRL_TIMEOUT: Duration = Duration::from_secs(2);
```

These values are not user-configurable. They match the existing
`DESKTOP_RUN_EXECUTION_WAIT` (20s) pattern in `launch_desktop_execution.rs`
— fixed polling windows enforced by `Instant::now()`.

## Examples

### Check the run window observation in a manifest

```bash
RUN_DIR=target/test-work/launch-smoke-real/runs/first-lessons-real-ui-actions/real-alice-smoke

cat "$RUN_DIR/manifest.json" \
  | jq '{
      toolbar_dispatch: .assertions.run_world_desktop_toolbar_dispatch,
      window_observed: .assertions.run_world_desktop_toolbar_window_observed,
      execution_observed: .assertions.run_world_desktop_execution_observed
    }'
```

Expected output when polling succeeds:

```json
{
  "toolbar_dispatch": {
    "passed": true,
    "detail": "xdotool clicked the configured Run toolbar coordinate (344,47) ..."
  },
  "window_observed": {
    "passed": true,
    "detail": "observed a new Alice Run window 0x600042 (distinct from main window 0x600007) after Run toolbar click via wmctrl polling (found on poll 3 of 20, elapsed 1.5s); this indicates desktop Run window opening, not world completion"
  },
  "execution_observed": {
    "passed": true,
    "detail": "observed RabbitHole desktop Run execution artifact with VM statement events ..."
  }
}
```

### Trace the full toolbar→window→execution chain

```bash
cat "$RUN_DIR/manifest.json" \
  | jq '.assertions | to_entries[]
        | select(.key | test("run_world_desktop"))
        | {key, passed: .value.passed, detail: .value.detail}'
```

### Run the unit tests for the polling module

```bash
cargo test -p eatme-alice -- launch_run_window_poll --nocapture
```

The polling module includes 10 inline unit tests using `FakeCommandRunner`:

| Test | Verifies |
| --- | --- |
| `finds_new_run_window_on_first_poll` | Immediate return when `wmctrl` output contains a new Run window. |
| `excludes_main_window_id` | The main window ID is not accepted as a new Run window. |
| `returns_not_found_when_no_run_window_appears` | `NotFound` after polling with no matching output. |
| `accepts_any_run_window_when_main_id_is_none` | No exclusion when the main window ID is unavailable. |
| `line_heuristic_matches_org_alice_run` | Per-line match on `org.alice` + `Run` patterns. |
| `line_heuristic_rejects_firefox` | Firefox windows are excluded even if they contain "alice". |
| `line_heuristic_rejects_main_window_title` | Main window titles without "Run" are excluded. |
| `extracts_window_id_from_wmctrl_line` | Correct `0x`-prefixed hex extraction from wmctrl output. |
| `find_new_run_window_returns_first_new_run_window_id` | Multi-line scan returns the first new Run window ID. |
| `find_new_run_window_returns_none_when_only_main_matches` | All Run window matches excluded when they share the main window ID. |

### Validate quality gates

```bash
# Module line count (must be ≤ 500)
wc -l crates/eatme-alice/src/launch_run_window_poll.rs
wc -l crates/eatme-alice/src/launch_run_window.rs

# Full quality gate
TMPDIR=/tmp ./scripts/quality-gates.sh
```

## Troubleshooting

### `run_world_desktop_toolbar_window_observed` still fails

Check the detail string for polling statistics:

```bash
cat "$RUN_DIR/manifest.json" \
  | jq '.assertions.run_world_desktop_toolbar_window_observed.detail'
```

If the detail says "20 polls, excluded main window 0x...":

1. The poller ran to completion but no new window appeared. Alice may not
   have created the Run window within 10 seconds.
2. Check the screenshots for visual state:
   ```bash
   file "$RUN_DIR/screenshots/run-window-after-dispatch.png"
   ```
3. Look at the `wmctrl -lx` output in the probe's `stdout` field for what
   windows were visible:
   ```bash
   cat "$RUN_DIR/manifest.json" \
     | jq '.assertions.run_world_desktop_toolbar_window_observed.stdout'
   ```

### Toolbar dispatch is blocked

The toolbar dispatch is skipped when:

- The Run window already opened via the Ctrl+F5 shortcut path (expected).
- Window activation failed (check `activate_alice_window_ui_action`).
- Window geometry does not match the expected 1000×740 launch size.

Check the full chain:

```bash
cat "$RUN_DIR/manifest.json" \
  | jq '{
      activation: .assertions.activate_alice_window_ui_action,
      shortcut_window: .assertions.run_world_after_shortcut_dispatch,
      toolbar_dispatch: .assertions.run_world_desktop_toolbar_dispatch
    }'
```

### Polling finds the main window instead of a new window

This cannot happen — the poller explicitly excludes the main window ID
from `toolbar_probe.window_id`. If `toolbar_probe.window_id` is `None`
(which should not occur when the toolbar dispatch passes), the poller
accepts any matching Run window without exclusion. The detail string will
not mention "distinct from main window" in this edge case.

### `wmctrl` times out during polling

Each `wmctrl -lx` invocation has a 2-second timeout. If `wmctrl`
consistently times out, the poller will exhaust fewer iterations (since
each failed invocation consumes 2 seconds of the 10-second deadline).
The `NotFound` result will show a lower poll count. Check that `wmctrl`
works outside the harness:

```bash
DISPLAY=:99 wmctrl -lx
```

### 500-line module limit triggers

The new `launch_run_window_poll.rs` module should be well under 500 lines
(approximately 150–200 lines including tests). The existing
`launch_run_window.rs` should decrease slightly (removal of the 2-second
sleep, addition of a single delegation call). Check current counts:

```bash
wc -l crates/eatme-alice/src/launch_run_window_poll.rs \
      crates/eatme-alice/src/launch_run_window.rs
```

## Related documentation

- [Post-Focus Screenshot Evidence](post-focus-screenshot-evidence.md) —
  Screenshot capture pipeline that precedes the Run window observation.
- [Deterministic Real-Alice Smoke Test](deterministic-real-alice-smoke-test.md) —
  The baseline real-Alice integration test.
- [First-Lesson Vertical Slice](first-lesson-vertical-slice.md) —
  Per-step evidence model including the Run toolbar dispatch probes.
- [Evidence Artifact Contract](evidence-artifact-contract.md) — Schema and
  validation rules for evidence artifacts.
- [Validation and Quality Gates](validation-quality-gates.md) — Repository
  quality gates including the 500-line module limit.
