# Post-focus screenshot evidence

The launch smoke pipeline captures a **post-focus screenshot** after window
activation succeeds. This second screenshot proves that the Alice window was
visually present and focused — not just that `xdotool`/`wmctrl` reported
success. It is captured only for scenarios that require real UI actions
(`first-lessons-real-ui-actions`).

## Contents

- [Usage](#usage)
- [Pipeline position](#pipeline-position)
- [Blocked cascade](#blocked-cascade)
- [Manifest fields](#manifest-fields)
- [Assertion key](#assertion-key)
- [API surface](#api-surface)
- [Evidence module](#evidence-module)
- [Configuration](#configuration)
- [Examples](#examples)
- [Troubleshooting](#troubleshooting)
- [Related documentation](#related-documentation)

## Usage

The post-focus screenshot is captured automatically when running any launch
smoke scenario that enables real UI actions. No additional flags are needed.

Run the first-lesson vertical slice (fake tools, CI-safe):

```bash
cargo test -p eatme-alice --test first_lesson_vertical_slice \
  -- fake_toolchain_vertical_slice_reports --nocapture
```

Run the real-Alice smoke test with post-focus capture:

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

After the run, the post-focus screenshot is at:

```text
<run-dir>/screenshots/post_focus.png
```

## Pipeline position

The post-focus screenshot is captured **after** window activation and **before**
the UI-action contract probes (place-object, edit-procedure, run-world,
save-project). This positions it as the final piece of "the window is real and
focused" evidence before the harness attempts deeper interaction.

```text
1. Dependencies check
2. Alice discovery + packaging
3. Xvfb display start
4. Alice launch + process wait
5. Window list capture (wmctrl -lx / xwininfo)
6. Startup screenshot (scrot / import → screenshots/startup.png)
7. Alice window search (targeting)
8. Window activation (wmctrl -ia / xdotool windowfocus)
9. ► Post-focus screenshot (scrot / import → screenshots/post_focus.png)   ← NEW
10. UI-action contract probes (place-object, edit, run, save)
11. Manifest build + write
```

The startup screenshot (step 6) captures whatever is on the virtual display
immediately after launch. The post-focus screenshot (step 9) captures the
display state after the harness has identified and focused a specific Alice
window. Comparing the two provides visual evidence that focus changed.

## Blocked cascade

The post-focus screenshot follows a blocked cascade from window detection
through activation:

| Step | Depends on | Blocked when |
| --- | --- | --- |
| `specific_alice_window_detected` | Window list capture | wmctrl/xwininfo finds no Alice main window. |
| `activate_alice_window_ui_action` | Window detection | Detection failed — no window id to activate. |
| `post_focus_screenshot_captured` | Window activation | Activation failed — screenshot would not reflect a focused Alice window. |

When activation fails or is blocked, the post-focus screenshot step is skipped
and the manifest records:

- `post_focus_screenshot`: `null`
- `post_focus_screenshot_error`: `"blocked: window activation did not succeed"`

The assertion `post_focus_screenshot_captured` records `passed: false` with a
detail explaining why the capture was blocked. This preserves the invariant
that every interaction step either succeeds, fails with an error, or is
explicitly blocked by a prior step.

## Manifest fields

Two new fields are added to `LaunchSmokeManifest`:

| Field | Type | Default | Meaning |
| --- | --- | --- | --- |
| `post_focus_screenshot` | `Option<ArtifactInfo>` | `null` | Artifact metadata for `screenshots/post_focus.png`. Present when the screenshot was captured successfully. |
| `post_focus_screenshot_error` | `Option<String>` | `null` | Error message when post-focus capture failed or was blocked. |

Both fields use `#[serde(default)]` to ensure backward compatibility. Older
manifests that lack these fields deserialize cleanly with `None` values.

Example manifest excerpt with a successful post-focus capture:

```json
{
  "screenshot": {
    "path": "runs/first-lessons-real-ui-actions/run-1/screenshots/startup.png",
    "size_bytes": 45321,
    "sha256": "abc123..."
  },
  "post_focus_screenshot": {
    "path": "runs/first-lessons-real-ui-actions/run-1/screenshots/post_focus.png",
    "size_bytes": 52104,
    "sha256": "def456..."
  },
  "post_focus_screenshot_error": null
}
```

Example manifest excerpt when activation was blocked:

```json
{
  "screenshot": {
    "path": "runs/first-lessons-real-ui-actions/run-1/screenshots/startup.png",
    "size_bytes": 45321,
    "sha256": "abc123..."
  },
  "post_focus_screenshot": null,
  "post_focus_screenshot_error": "blocked: window activation did not succeed"
}
```

## Assertion key

| Assertion key | Meaning | Passes when |
| --- | --- | --- |
| `post_focus_screenshot_captured` | Post-focus screenshot evidence was captured. | `post_focus_screenshot` is `Some` with `size_bytes > 0`. |

This assertion is only inserted for scenarios that require real UI actions.
It does not affect the `startup_screenshot` or `real_alice_execution_evidence`
assertions — those use the startup screenshot and are independent.

The assertion result includes a detail string:

| Outcome | Detail |
| --- | --- |
| Captured successfully | `"post-focus screenshot captured (N bytes)"` where N is `size_bytes` |
| Capture failed | Raw error from `capture_post_focus_screenshot` (e.g. `"capturing post-focus screenshot failed: scrot=..., import=..."`) |
| Blocked by activation | `"blocked: window activation did not succeed"` |

## API surface

No new public API types are introduced. The post-focus screenshot uses the
existing `ArtifactInfo` type from `eatme-core` and is threaded through the
existing `build_manifest` function.

### Evidence module

The capture logic lives in `crates/eatme-alice/src/launch/evidence.rs`:

| Function | Visibility | Purpose |
| --- | --- | --- |
| `capture_screenshot_to(runner, display, path)` | `pub(super)` | Shared helper that runs scrot→import fallback chain to an explicit output path. Used by both `capture_screenshot` and `capture_post_focus_screenshot`. |
| `capture_post_focus_screenshot(runner, display, run_dir)` | `pub(super)` | Captures to `screenshots/post_focus.png`. Returns `Result<ArtifactInfo>`. |

The existing `capture_screenshot` function is refactored to call
`capture_screenshot_to` with `screenshots/startup.png`. The capture chain
(scrot with 10s timeout and 2 retries, then ImageMagick import fallback)
is identical for both paths.

### Orchestration in launch.rs

After the existing window activation block:

```rust
// Existing: activation probe + assertion recording
let alice_window_activation_probe = if options.scenario.requires_real_ui_actions() {
    let probe = probe_alice_window_activation(&runner, display.name(), &window_text);
    record_alice_window_activation(&mut assertions, &probe);
    // ... failure_category handling ...
    Some(probe)
} else {
    None
};

// NEW: post-focus screenshot capture
let (post_focus_screenshot, post_focus_screenshot_error) =
    if options.scenario.requires_real_ui_actions() {
        match &alice_window_activation_probe {
            Some(probe) if probe.status == "passed" => {
                capture_artifact_or_error(
                    capture_post_focus_screenshot(&runner, display.name(), &run_dir),
                )
            }
            _ => (None, Some("blocked: window activation did not succeed".into())),
        }
    } else {
        (None, None)
    };
if options.scenario.requires_real_ui_actions() {
    let ok = post_focus_screenshot
        .as_ref()
        .map(|a| a.size_bytes > 0)
        .unwrap_or(false);
    let detail = match (&post_focus_screenshot, &post_focus_screenshot_error) {
        (Some(a), _) if a.size_bytes > 0 => {
            format!("post-focus screenshot captured ({} bytes)", a.size_bytes)
        }
        (_, Some(err)) => err.clone(),
        _ => "post-focus screenshot capture failed".into(),
    };
    assertions.insert(
        "post_focus_screenshot_captured".into(),
        bool_assert(ok, detail),
    );
}
```

The `post_focus_screenshot` and `post_focus_screenshot_error` are passed to
`build_manifest` as two additional `Option` parameters.

### Manifest builder

`build_manifest` and `write_blocked_manifest` accept two new parameters:

```rust
pub(super) fn build_manifest(
    // ... existing parameters ...
    post_focus_screenshot: Option<ArtifactInfo>,       // NEW
    post_focus_screenshot_error: Option<String>,        // NEW
    // ... remaining parameters ...
) -> LaunchSmokeManifest { ... }
```

`write_blocked_manifest` passes `None` for both fields when writing a
manifest for an early-exit failure.

## Configuration

No new configuration options are required. The post-focus screenshot uses
the same capture tools (scrot, import) with the same timeout (10s) and
retry count (2) as the startup screenshot.

### Output path

The post-focus screenshot is always written to:

```text
<run-dir>/screenshots/post_focus.png
```

This path is hardcoded — no user-controlled path components are involved.
The `screenshots/` directory is created by the existing startup screenshot
capture, so it exists before the post-focus capture runs.

## Examples

### Inspect both screenshots after a real-Alice run

```bash
RUN_DIR=target/test-work/launch-smoke-real/runs/first-lessons-real-ui-actions/real-alice-smoke

# Startup screenshot (before focus)
file "$RUN_DIR/screenshots/startup.png"

# Post-focus screenshot (after activation)
file "$RUN_DIR/screenshots/post_focus.png"

# Compare file sizes to see visual difference
ls -la "$RUN_DIR/screenshots/"
```

### Check the post-focus assertion in the manifest

```bash
cat "$RUN_DIR/manifest.json" \
  | jq '{
      post_focus_screenshot: .post_focus_screenshot,
      post_focus_error: .post_focus_screenshot_error,
      assertion: .assertions.post_focus_screenshot_captured
    }'
```

Expected output when activation succeeded:

```json
{
  "post_focus_screenshot": {
    "path": ".../screenshots/post_focus.png",
    "size_bytes": 52104,
    "sha256": "def456..."
  },
  "post_focus_error": null,
  "assertion": {
    "passed": true,
    "detail": "post-focus screenshot captured (52104 bytes)"
  }
}
```

### Check all interaction-related assertions together

```bash
cat "$RUN_DIR/manifest.json" \
  | jq '.assertions | to_entries[]
        | select(.key | test("specific_alice_window|activate_alice|post_focus"))
        | {key, passed: .value.passed, detail: .value.detail}'
```

Expected output for a successful interaction chain:

```json
{"key": "activate_alice_window_ui_action", "passed": true, "detail": "wmctrl activated Alice window 0x600007"}
{"key": "post_focus_screenshot_captured", "passed": true, "detail": "post-focus screenshot captured (52104 bytes)"}
{"key": "specific_alice_window_detected", "passed": true, "detail": "wmctrl or xwininfo identified Alice main window 0x600007"}
```

### Fake-toolchain test coverage

Fake-toolchain tests automatically exercise the post-focus screenshot path
because the fake `scrot` and `import` tools respond to any output path. No
additional test setup is needed — the existing `PathOverride` fake tools
handle both `screenshots/startup.png` and `screenshots/post_focus.png`.

```bash
cargo test -p eatme-alice --test first_lesson_vertical_slice \
  -- fake_toolchain --nocapture
```

### Validate manifest schema round-trip

The `manifest_schema_round_trip` test in `crates/eatme-alice/src/launch/tests.rs`
validates that `post_focus_screenshot` and `post_focus_screenshot_error` survive
JSON serialize→deserialize. This test runs on every `cargo test -p eatme-alice`
invocation:

```bash
cargo test -p eatme-alice manifest_schema_round_trip
```

## Troubleshooting

### Post-focus screenshot is null but activation passed

Check the manifest for `post_focus_screenshot_error`:

```bash
cat "$RUN_DIR/manifest.json" | jq '.post_focus_screenshot_error'
```

If the error mentions scrot or import, verify the screenshot tools work
after focus:

```bash
Xvfb :99 -screen 0 1024x768x24 &
DISPLAY=:99 scrot /tmp/post_focus_test.png
file /tmp/post_focus_test.png
```

### Post-focus screenshot is blocked but Alice window is visible

The blocked cascade requires all three steps to succeed in order:
detection → activation → post-focus capture. Check which step failed:

```bash
cat "$RUN_DIR/manifest.json" | jq '{
  detection: .assertions.specific_alice_window_detected,
  activation: .assertions.activate_alice_window_ui_action,
  post_focus: .assertions.post_focus_screenshot_captured
}'
```

If detection passed but activation failed, the window may be present but
wmctrl/xdotool cannot focus it (common when the window manager does not
support `_NET_ACTIVE_WINDOW`). The
`alice_window_activation_unsupported` failure category indicates this.

### Post-focus screenshot is identical to startup screenshot

This is expected in Xvfb environments where there is only one window and
no compositor. The display buffer does not change visually when focus moves
to the only mapped window. The assertion still passes because the capture
succeeded — visual difference is not validated.

### Older manifests fail to deserialize

Both new fields use `#[serde(default)]`, so they deserialize as `None` from
older JSON that lacks the keys. If deserialization fails, check that the
`LaunchSmokeManifest` struct has `#[serde(default)]` on the new fields:

```rust
#[serde(default)]
pub post_focus_screenshot: Option<ArtifactInfo>,
#[serde(default)]
pub post_focus_screenshot_error: Option<String>,
```

### 500-line module limit

The `evidence.rs` module adds approximately 15 lines for `capture_screenshot_to`
and `capture_post_focus_screenshot`. The `launch.rs` module adds approximately
8 lines for orchestration. Both modules remain well under the 500-line quality
gate. If the gate triggers, check the current line counts:

```bash
wc -l crates/eatme-alice/src/launch/evidence.rs crates/eatme-alice/src/launch.rs
```

## Related documentation

- [Deterministic Real-Alice Smoke Test](deterministic-real-alice-smoke-test.md) —
  The baseline real-Alice integration test that produces the startup screenshot.
- [First-Lesson Vertical Slice](first-lesson-vertical-slice.md) —
  Per-step evidence model including window detection and activation probes.
- [Alice Integration](alice-integration.md) — CLI commands for discovery,
  packaging, and launch smoke.
- [Alice Lesson Smoke](alice-lesson-smoke.md) — Desktop scenario roster and
  evidence contracts.
- [Evidence Artifact Contract](evidence-artifact-contract.md) — Schema and
  validation rules for evidence artifacts.
- [Validation and Quality Gates](validation-quality-gates.md) — Repository
  quality gates including the 500-line module limit.
