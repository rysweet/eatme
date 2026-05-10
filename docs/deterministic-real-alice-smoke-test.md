# Deterministic real-Alice launch smoke integration test

The `launch_smoke_real` integration test validates that a real Alice desktop
session can be launched, observed, and evidenced through the existing
`run_launch_smoke` harness. It is gated behind the `EATME_REAL_ALICE=1`
environment variable so CI and developer machines without Alice desktop
dependencies skip the test automatically.

## Contents

- [Usage](#usage)
- [Environment gate](#environment-gate)
- [What the test proves](#what-the-test-proves)
- [Manifest assertions](#manifest-assertions)
- [Screenshot validation](#screenshot-validation)
- [Manifest schema round-trip test](#manifest-schema-round-trip-test)
- [API surface](#api-surface)
- [Configuration](#configuration)
- [Examples](#examples)
- [Troubleshooting](#troubleshooting)
- [Related documentation](#related-documentation)

## Usage

Run the real-Alice integration test:

```bash
EATME_REAL_ALICE=1 cargo test -p eatme-alice --test launch_smoke_real
```

The test is a standard Rust integration test in
`crates/eatme-alice/tests/launch_smoke_real.rs`. When `EATME_REAL_ALICE` is
unset or not `1`, the test returns early with a skip message and passes. No
`#[ignore]` attribute is used — the runtime check matches the CI workflow
pattern used by real Alice launch-smoke jobs.

Run all `eatme-alice` tests (the real-Alice test skips automatically when the
environment variable is absent):

```bash
cargo test -p eatme-alice
```

Run only the CI-safe schema round-trip test without Alice dependencies:

```bash
cargo test -p eatme-alice manifest_schema_round_trip
```

## Environment gate

| Variable | Required value | Effect |
| --- | --- | --- |
| `EATME_REAL_ALICE` | `1` | Enables the real-Alice integration test. Any other value or absence causes the test to skip. |
| `ALICE_HOME` | Path to Alice checkout | The Alice checkout directory. Defaults to `../alice3-modernization` when not set. |

The gate is a runtime `std::env::var` check, not a compile-time `cfg`
attribute. This means:

- `cargo test -p eatme-alice` always compiles the test.
- The test binary always includes `launch_smoke_real`.
- The test body returns early when the gate is not satisfied.
- CI workflows that set `EATME_REAL_ALICE=1` on self-hosted runners with Alice
  desktop dependencies get the full integration validation.

## What the test proves

The real-Alice integration test exercises the complete `run_launch_smoke` path
with a real Alice installation:

1. **Dependency check** — all required desktop tools are available (Xvfb,
   xdpyinfo, wmctrl, xwininfo, xdotool, scrot/import, Java, Maven).
2. **Alice discovery** — the configured `ALICE_HOME` has the expected checkout
   shape.
3. **Alice packaging** — the Maven build succeeds (uses `--offline-package`).
4. **Virtual display** — Xvfb starts and responds to xdpyinfo.
5. **Alice launch** — the Java process starts and stays alive through the
   startup wait.
6. **Visual evidence** — a non-empty startup screenshot is captured as a PNG
   file.
7. **Window list capture** — the window list is captured for diagnostics (window
   identity does not contribute to the `startup_screenshot` assertion for this
   scenario because `accepts_window_evidence()` returns `false`).
8. **Log evidence** — the Alice log file exists and contains no fatal lines.
9. **Manifest fidelity** — all 6 manifest assertions pass and the manifest is
   written to disk as valid JSON.

## Manifest assertions

The test validates that all 6 core manifest assertions pass:

| Assertion key | Meaning |
| --- | --- |
| `dependencies_available` | All required desktop tools were detected. |
| `display_responsive` | The virtual X display responded to xdpyinfo. |
| `process_started` | The Alice Java process stayed alive through the startup wait. |
| `startup_screenshot` | A non-empty startup screenshot was captured. (The `real-alice-launch-smoke` scenario requires a screenshot; window-identity evidence is only accepted for lesson-level scenarios.) |
| `no_fatal_logs` | The Alice log contains no fatal DISPLAY, OpenGL, or Java exception lines. |
| `real_alice_execution_evidence` | The combination of process, display, visual evidence, and log proves real Alice execution. |

The test asserts:

```rust
assert!(manifest.failure_category.is_none());
assert!(manifest.assertions.values().all(|a| a.passed));
```

## Screenshot validation

The test reads the screenshot path from `manifest.screenshot.path` rather than
hard-coding the internal layout. It validates:

- The screenshot file exists on disk.
- The file is a valid PNG (the first 8 bytes match the PNG magic signature
  `\x89PNG\r\n\x1a\n`).
- The file size is greater than zero.

```rust
let screenshot_path = &manifest.screenshot.as_ref().unwrap().path;
let header = std::fs::read(screenshot_path).unwrap();
assert!(header.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]));
```

## Manifest schema round-trip test

A CI-safe unit test in `crates/eatme-alice/src/launch/tests.rs` validates
that the `LaunchSmokeManifest` type round-trips through JSON
serialize→deserialize without data loss. This test runs on every
`cargo test -p eatme-alice` invocation and does not require real Alice or
desktop dependencies.

The round-trip test:

1. Constructs a `LaunchSmokeManifest` with representative field values,
   including assertions, artifacts, fatal log lines, and optional fields.
2. Serializes the manifest to JSON with `serde_json::to_string`.
3. Deserializes the JSON back to a `LaunchSmokeManifest` with
   `serde_json::from_str`.
4. Asserts field-by-field equality between the original and deserialized
   manifest.

This test required adding `serde::Deserialize` to three types in
`crates/eatme-core/src/manifest.rs`:

| Type | Derives |
| --- | --- |
| `AssertionResult` | `Clone`, `Debug`, `Serialize`, `Deserialize` |
| `ArtifactInfo` | `Clone`, `Debug`, `Serialize`, `Deserialize` |
| `LaunchSmokeManifest` | `Clone`, `Debug`, `Serialize`, `Deserialize` |

The `Deserialize` derive is additive and backward-compatible. No production
code path deserializes manifest types from external input; the derive exists
solely for test round-trip validation.

## API surface

The integration test uses the existing public API from `eatme-alice`:

```rust
use eatme_alice::{LaunchSmokeOptions, LaunchSmokeScenario, run_launch_smoke};
```

The manifest types come from `eatme-core` and are accessed through the
`run_launch_smoke` return type:

| Type | Crate | Purpose |
| --- | --- | --- |
| `run_launch_smoke(options)` | `eatme-alice` | Runs the full launch smoke pipeline and returns a `LaunchSmokeManifest`. |
| `LaunchSmokeOptions` | `eatme-alice` | Configuration for Alice home, run id, runs directory, timeout, scenario, and packaging options. |
| `LaunchSmokeScenario` | `eatme-alice` | Identifies the scenario by id and starter project path. |
| `LaunchSmokeManifest` | `eatme-core` | The evidence manifest containing assertions, artifacts, failure category, and all launch metadata. |
| `AssertionResult` | `eatme-core` | Individual assertion with `passed: bool` and `detail: String`. |
| `ArtifactInfo` | `eatme-core` | Artifact metadata with `path`, `size_bytes`, and `sha256`. |

No new public API is introduced. The integration test consumes the same
`run_launch_smoke` function used by the fake-toolchain tests and the CLI.

## Configuration

### Integration test options

The real-Alice test uses these `LaunchSmokeOptions`:

| Option | Value | Rationale |
| --- | --- | --- |
| `alice_home` | `ALICE_HOME` env var or `../alice3-modernization` | Standard Alice checkout location. |
| `scenario` | `LaunchSmokeScenario::default()` | Uses the `real-alice-launch-smoke` baseline scenario. |
| `run_id` | `real-alice-smoke` | Kebab-case identifier for the evidence directory. |
| `runs_dir` | `target/test-work/launch-smoke-real/runs` | Isolated under `target/` to avoid polluting project root. |
| `timeout_seconds` | `900` | 15-minute timeout for cold Maven builds and slow Java startup. |
| `json` | `true` | Machine-readable output. |
| `no_memory` | `true` | No persistent memory side effects from test runs. |
| `offline_package` | `true` | Uses cached Maven dependencies, no network access. |

### Host requirements

The real-Alice integration test requires a Linux host with:

| Dependency | Minimum | Purpose |
| --- | --- | --- |
| Java | 21 | Alice runtime |
| Maven | 3.9+ | Alice packaging |
| Xvfb | Any | Virtual X display |
| xdpyinfo | Any | Display readiness probe |
| wmctrl | Any | Window list capture |
| xwininfo | Any | Fallback window tree capture |
| xdotool | Any | Window activation |
| scrot or ImageMagick `import` | Any | Screenshot capture |
| Mesa/llvmpipe | Any | Software OpenGL rendering |

Install all dependencies on Ubuntu/Debian:

```bash
sudo apt-get install -y \
  openjdk-21-jdk maven \
  xvfb x11-utils wmctrl x11-xserver-utils xdotool \
  scrot imagemagick mesa-utils
```

## Examples

### Run the real-Alice smoke test on a self-hosted runner

```bash
export ALICE_HOME=/opt/alice3-modernization
EATME_REAL_ALICE=1 cargo test -p eatme-alice --test launch_smoke_real -- --nocapture
```

### Run all eatme-alice tests (real test auto-skips)

```bash
cargo test -p eatme-alice
```

Output includes:

```text
test launch_smoke_real::real_alice_launch_smoke_writes_passing_manifest_and_valid_screenshot ... ok
```

When `EATME_REAL_ALICE` is not set, the test prints a skip message and passes
without exercising Alice.

### Inspect the evidence after a real run

```bash
cat target/test-work/launch-smoke-real/runs/real-alice-launch-smoke/real-alice-smoke/manifest.json \
  | jq '.assertions | to_entries[] | {key, passed: .value.passed}'
```

Expected output when all assertions pass:

```json
{"key": "dependencies_available", "passed": true}
{"key": "display_responsive", "passed": true}
{"key": "no_fatal_logs", "passed": true}
{"key": "process_started", "passed": true}
{"key": "real_alice_execution_evidence", "passed": true}
{"key": "startup_screenshot", "passed": true}
```

### Run only the CI-safe schema test

```bash
cargo test -p eatme-alice manifest_schema_round_trip
```

This test always runs, requires no Alice installation, and validates that
manifest types serialize and deserialize correctly.

## Troubleshooting

### Test skips unexpectedly

Verify the environment variable is set to exactly `1`:

```bash
echo $EATME_REAL_ALICE   # should print: 1
```

The check is `std::env::var("EATME_REAL_ALICE") == Ok("1".into())`. Values
like `true`, `yes`, or empty string do not activate the test.

### Missing dependencies

Run the dependency check first:

```bash
cargo run -q -p eatme-cli -- deps check --json
```

The manifest `failure_category` field will report `missing_dependency` and the
`dependencies_available` assertion will fail if desktop tools are missing.

### Screenshot is not a valid PNG

If the test fails on PNG header validation, check that `scrot` or
ImageMagick `import` is installed and can capture from an Xvfb display:

```bash
Xvfb :99 -screen 0 1024x768x24 &
DISPLAY=:99 scrot /tmp/test.png
file /tmp/test.png   # should report: PNG image data
```

### Alice process exits immediately

Check that the Alice Maven build succeeded and that the Alice IDE jar exists:

```bash
ls ${ALICE_HOME}/alice-ide/target/alice-ide-*-SNAPSHOT.jar
```

If the jar is missing, run packaging first:

```bash
cargo run -q -p eatme-cli -- alice package \
  --alice-home "${ALICE_HOME}" --offline --json
```

### Unix socket path too long

In deep worktree paths, the X display socket path may exceed the 108-character
Unix socket limit. Use `TMPDIR=/tmp` to shorten the socket path:

```bash
TMPDIR=/tmp EATME_REAL_ALICE=1 cargo test -p eatme-alice --test launch_smoke_real
```

## Related documentation

- [Alice Integration](alice-integration.md) — CLI commands for discovery,
  packaging, and launch smoke.
- [Alice Lesson Smoke](alice-lesson-smoke.md) — Desktop scenario roster and
  evidence contracts.
- [Validation and Quality Gates](validation-quality-gates.md) — Repository
  quality gates including the real Alice launch gate.
- [Outside-in Alice Test Modules](outside-in-alice-test-modules.md) — Rust
  test module layout and authoring workflow.
