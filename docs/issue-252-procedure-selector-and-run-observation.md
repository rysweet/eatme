# Issue #252: Procedure selector and Run observation fixes

Two bugs caused five scenario assertion failures in the
`first-lessons-real-ui-actions` scenario. Both are fixed.

## Bug 1 — Procedure selector mismatch

### Problem

`DEFAULT_PROCEDURE_SELECTOR` in `launch_edit_procedure.rs` was set to
`scene.eatmeFirstLessonStep`. The Alice-side `EatmeEditProcedure` hook
requires exactly `scene.eatmeFirstLesson` (no `Step` suffix). Every
`edit-procedure-or-code-block` action probe failed because Alice could not
resolve the selector.

### Fix

The constant is now `scene.eatmeFirstLesson`:

```rust
pub(crate) const DEFAULT_PROCEDURE_SELECTOR: &str = "scene.eatmeFirstLesson";
```

All test fixtures that hardcoded `scene.eatmeFirstLessonStep` in
`procedure_selector` fields are updated to match.

### Scope

Only the **procedure selector** changed. The save/reopen selectors documented in
[Save/Reopen Readiness](save-reopen-readiness.md) remain
`scene.eatmeFirstLessonStep` — that is the correct selector for save and
reopen operations and is unaffected by this fix.

| Selector constant | Value | Used by |
| --- | --- | --- |
| `DEFAULT_PROCEDURE_SELECTOR` | `scene.eatmeFirstLesson` | `edit-procedure-or-code-block` action probe |
| Save selector | `scene.eatmeFirstLessonStep` | Save Project proof artifact |
| Reopen selector | `scene.eatmeFirstLessonStep` | Select Project proof artifact |

### Configuration

No configuration changes are required. The selector is a compile-time constant
resolved by `launch_edit_procedure.rs`. Scenario assets, Gadugi adapters, and
CLI commands are unaffected.

### Verification

```bash
cargo run -q -p eatme-cli -- assets validate --json
```

The `edit-procedure-or-code-block` action probe now resolves the procedure
selector correctly. Previously failing test fixtures pass with the corrected
value.

---

## Bug 2 — Run observation uses screenshot comparison

### Problem

After the Run toolbar click, `probe_run_window_after_toolbar_button` fell back
to polling `wmctrl -lx` / `xwininfo -root -tree` for a separate Run window.
Alice does not open a separate Run window — it renders the Run animation in the
existing scene panel. The wmctrl fallback always failed because no new window
appears, causing the `run_world_desktop_toolbar_window_observed` assertion to
fail.

### Fix

The wmctrl/xwininfo fallback is replaced with a **pre/post screenshot
comparison**:

1. **Before the toolbar click** — `probe_run_toolbar_sequence` captures
   `screenshots/scene-before-run-click.png` using `capture_run_window_screenshot`.
2. **After the toolbar click** — `probe_run_window_after_toolbar_button`
   captures `screenshots/scene-after-run-click.png` after a 2-second delay.
3. **Comparison** — `screenshots_differ` performs a byte-level comparison of
   the two files using `std::fs::read`. If the files differ, the Run animation
   started and the assertion passes.

The sentinel fast-path (checking `run-window-created.json`) remains the
preferred detection method. The screenshot comparison replaces only the
wmctrl/xwininfo fallback branch.

### API

#### `capture_run_window_screenshot`

The existing `capture_run_window_screenshot` gains an optional `filename`
parameter (the current default `"run-window-after-dispatch.png"` is preserved
for callers that omit it):

```rust
fn capture_run_window_screenshot(
    runner: &impl CommandRunner,
    display: &str,
    run_dir: &Path,
    filename: &str,
) -> Result<String, String>
```

Captures a screenshot to `{run_dir}/screenshots/{filename}` using `scrot` with
`import` fallback (same as before). Returns the command string on success.

> **Note:** A separate `capture_screenshot_to` function exists in
> `launch/evidence.rs` for post-focus launch evidence. That function returns
> `Result<ArtifactInfo>` and serves a different purpose. The
> `capture_run_window_screenshot` function in `launch_run_window.rs` is
> intentionally kept distinct.

#### `screenshots_differ`

```rust
fn screenshots_differ(before: &Path, after: &Path) -> bool
```

Returns `true` when both files exist, are non-empty, and have different byte
contents. Returns `false` when either file is missing, empty, or contents are
identical. Uses `std::fs::read` — no external commands.

### Evidence artifacts

The screenshot directory under each run now contains:

```text
runs/first-lessons-real-ui-actions/student-first-lessons-real-ui-actions/
`-- screenshots/
    |-- startup.png
    |-- scene-before-run-click.png
    |-- scene-after-run-click.png
    `-- run-window-after-dispatch.png
```

| Artifact | Purpose | Produced by |
| --- | --- | --- |
| `startup.png` | Post-launch screenshot (unchanged) | Launch sequence |
| `scene-before-run-click.png` | Scene panel before Run toolbar click | `probe_run_toolbar_sequence` |
| `scene-after-run-click.png` | Scene panel after Run toolbar click | `probe_run_window_after_toolbar_button` |
| `run-window-after-dispatch.png` | Post-dispatch screenshot (unchanged) | `with_run_window_screenshot` via sentinel fast-path |

### Assertion detail strings

The `run_world_desktop_toolbar_window_observed` assertion detail reflects the
detection method used:

| Detection method | `detail` value |
| --- | --- |
| Sentinel file | `"observed RabbitHole Run-window-created sentinel after Run toolbar click; this records Alice preparing the desktop Run frame, not world completion"` |
| Screenshot diff | `"pre/post screenshot comparison detected scene panel change after Run toolbar click; this indicates the Run animation started in the existing scene panel, not world completion"` |

Both methods produce a `"passed"` status. The assertion id remains
`run_world_desktop_toolbar_window_observed` — no changes to readiness report
consumers.

### What did NOT change

- **Sentinel fast-path** — `run-window-created.json` is still checked first.
- **Shortcut path** — `probe_run_window_after_shortcut` still uses
  `capture_window_text` / wmctrl because the Ctrl+F5 shortcut path is a
  different code branch.
- **`run-window-after-dispatch.png`** — the screenshot produced by the sentinel
  fast-path still uses this filename.
- **`desktop-first-lesson-next-action.json`** — unchanged.
- **Probe signatures** — `probe_run_window_after_toolbar_button` and
  `probe_run_toolbar_sequence` keep their existing public signatures.

### Configuration

No configuration changes. The 2-second post-click delay matches the existing
shortcut-path delay. Screenshot capture uses the same `scrot` / `import`
toolchain.

### Verification

```bash
# Run the unit tests
cargo test -p eatme-alice

# Confirm the 5 previously-failing scenario assertions now pass
cargo run -q -p eatme-cli -- assets validate --json

# Full quality gate
TMPDIR=/tmp ./scripts/quality-gates.sh
```

The `run_world_desktop_toolbar_window_observed` assertion passes because the
screenshot comparison detects the scene panel change. No wmctrl or xwininfo
invocation is required for the toolbar fallback path.

### Troubleshooting

| Symptom | Cause | Resolution |
| --- | --- | --- |
| Screenshot comparison reports identical files | Alice did not start the Run animation within 2 seconds | Check Alice log for errors. Verify the toolbar click coordinates match Alice's window geometry. |
| `scene-before-run-click.png` missing | Screenshot capture failed before the click | Verify `scrot` or ImageMagick `import` is installed. Check `DISPLAY` environment variable. |
| `scene-after-run-click.png` missing | Screenshot capture failed after the click | Same as above. Also check that Alice process is still running. |
| Sentinel file still detected | `run-window-created.json` was left from a prior run | Clean the run directory before re-running. The sentinel fast-path takes priority over screenshot diff. |

### Security

The fix **reduces** external command surface. The toolbar fallback path no
longer invokes `wmctrl` or `xwininfo`. The screenshot comparison uses
`std::fs::read` only — no shell commands, no user-controlled paths. All
screenshot paths are hardcoded constants under `run_dir/screenshots/`.

---

## Cross-reference

| Document | Relevant section | Impact |
| --- | --- | --- |
| [Lesson Session Readiness](lesson-session-readiness.md) | Modernized Run-window evidence row | Assertion unchanged; detection method is internal |
| [Save/Reopen Readiness](save-reopen-readiness.md) | Save/reopen selectors | **No change** — those selectors remain `scene.eatmeFirstLessonStep` |
| [Evidence Artifact Contract](evidence-artifact-contract.md) | Screenshot artifacts | **No change** — JSON artifact contracts are unaffected; new screenshots documented in alice-lesson-smoke.md and lesson-session-readiness.md |
| [Post-focus Screenshot Evidence](post-focus-screenshot-evidence.md) | Screenshot capture | Same `scrot`/`import` toolchain, new filenames |
| [Alice Lesson Smoke](alice-lesson-smoke.md) | Evidence location tree | `scene-before-run-click.png` and `scene-after-run-click.png` added |
| [Installation](installation.md) | Desktop dependencies | wmctrl remains required for other probes; no change |
