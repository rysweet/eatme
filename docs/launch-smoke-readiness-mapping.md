# Launch-smoke readiness mapping

Launch-smoke readiness mapping is the feature that converts raw
`real-alice-launch-smoke` comparison manifests into structured readiness reports.
It is the bounded reporting layer between deterministic desktop evidence and
human/CI readiness decisions.

## Contents

- [Overview](#overview)
- [Architecture](#architecture)
- [Usage](#usage)
- [Configuration](#configuration)
- [API reference](#api-reference)
- [Evidence progress tracking](#evidence-progress-tracking)
- [Non-claim boundary contract](#non-claim-boundary-contract)
- [Scenario YAML reference](#scenario-yaml-reference)
- [Examples](#examples)
- [Troubleshooting](#troubleshooting)

## Overview

When a `real-alice-launch-smoke` comparison manifest exists, the readiness
mapper inspects its `targets` object for baseline and modernized entries. Each
target is checked for:

| Evidence item | What it proves |
| --- | --- |
| Target entry exists | Both sides of the comparison were attempted |
| Embedded `launch_manifest` present | The target ran through the full launch pipeline |
| Target status and failure category | The launch completed without a fatal failure category |
| Required assertions pass | `display_responsive`, `process_started`, `startup_screenshot`, `no_fatal_logs`, `real_alice_execution_evidence` |
| Artifacts recorded | Window-list, screenshot, and log artifact metadata paths are present and non-empty |

When all five items are present for both targets, the report status is `ready`.
Any missing, failed, malformed, or incomplete item sets the status to
`not_ready` with a machine-readable `issues` list.

## Architecture

The mapping lives in `crates/eatme-alice/src/compare/lesson_readiness/` and is
composed of four modules:

```text
lesson_readiness/
├── launch_smoke.rs              # Entry point: check_launch_smoke_readiness()
├── launch_smoke/
│   └── evidence_progress.rs     # Progress tracker for 5 required evidence items
└── output/
    ├── launch_smoke.rs          # Human summary and role readiness builder
    └── claims.rs                # Unproven claims and limitation constants
```

The `check_launch_smoke_readiness` function is dispatched when the comparison
manifest's `scenario_id` matches the exact string `real-alice-launch-smoke`.
All other scenario ids route to the first-lesson readiness path.

### Data flow

```text
comparison-manifest.json
  → check_lesson_session_readiness()
    → scenario_id == "real-alice-launch-smoke"?
      → check_launch_smoke_readiness()
        → inspect_required_launch_smoke_targets()
        → build_launch_smoke_readiness_output()
        → launch_smoke_evidence_progress()
        → LessonSessionReadinessReport { status, issues, evidence_progress, ... }
```

## Usage

### Check an existing manifest

```bash
cargo run -q -p eatme-cli -- alice check-lesson-readiness \
  --manifest runs/comparisons/real-alice-launch-smoke/local/comparison-manifest.json
```

### Run comparison and check readiness in one step

```bash
export NODE_OPTIONS=--max-old-space-size=32768
export ALICE_BASELINE_HOME=/path/to/alice-reference
export ALICE_MODERNIZED_HOME=/path/to/alice-candidate

cargo run -q -p eatme-cli -- alice compare-launch-smoke \
  --run-id local-real-alice-launch-smoke \
  --json \
  --no-memory \
  --offline-package
```

### JSON output

Add `--json` to any readiness command. The output includes:

```json
{
  "passed": true,
  "status": "ready",
  "readiness_status": "ready",
  "scenario_id": "real-alice-launch-smoke",
  "human_summary": "real-alice-launch-smoke launch-smoke readiness is ready from existing target launch-smoke manifest evidence only.",
  "issues": [],
  "required_evidence": [
    "comparison manifest with baseline and modernized targets for real-alice-launch-smoke",
    "embedded launch-smoke manifest for each target",
    "each target status is passed with no launch failure category",
    "required launch-smoke assertions passed for each target",
    "launch-smoke artifact metadata for window list, screenshot, and log"
  ],
  "evidence_progress": {
    "total_required": 5,
    "present": 5,
    "missing": 0,
    "invalid": 0,
    "not_observed": 0,
    "blocked": 0,
    "summary": "5 of 5 required launch-smoke evidence items are present; 0 missing, 0 invalid, 0 not observed, 0 blocked."
  },
  "unproven_claims": [
    "First-lesson completion is not proven.",
    "Full world execution is not proven.",
    "Grading is not proven.",
    "Creative assessment is not proven.",
    "Full Alice UI automation is not proven.",
    "Visible rendering correctness is not proven.",
    "Save completion is not proven.",
    "Deployed sharing/platform success is not proven."
  ],
  "limitations": [
    "First-lesson completion is not proven.",
    "Full world execution is not proven.",
    "Grading is not proven.",
    "Creative assessment is not proven.",
    "Full Alice UI automation is not proven.",
    "Visible rendering correctness is not proven.",
    "Save completion is not proven.",
    "Deployed sharing/platform success is not proven.",
    "bounded to existing launch-smoke manifest metadata",
    "does not add lesson-action detection",
    "does not grade student worlds",
    "does not perform creative assessment",
    "does not prove full UI automation",
    "does not prove visible correctness"
  ]
}
```

### Plain text output

Without `--json`, the CLI prints a bounded human-readable report:

```text
real-alice-launch-smoke launch-smoke readiness is ready from existing target
launch-smoke manifest evidence only.

Required evidence:
  ✓ comparison manifest with baseline and modernized targets for real-alice-launch-smoke
  ✓ embedded launch-smoke manifest for each target
  ✓ each target status is passed with no launch failure category
  ✓ required launch-smoke assertions passed for each target
  ✓ launch-smoke artifact metadata for window list, screenshot, and log

Unproven:
  First-lesson completion is not proven.
  Full world execution is not proven.
  Grading is not proven.
  Creative assessment is not proven.
  Full Alice UI automation is not proven.
  Visible rendering correctness is not proven.
  Save completion is not proven.
  Deployed sharing/platform success is not proven.
```

## Configuration

### Environment variables

| Variable | Required | Purpose |
| --- | --- | --- |
| `NODE_OPTIONS` | Recommended | Set `--max-old-space-size=32768` for large manifests |
| `ALICE_BASELINE_HOME` | For execution | Path to reference Alice checkout |
| `ALICE_MODERNIZED_HOME` | For execution | Path to candidate Alice checkout |
| `EATME_REAL_ALICE` | No | Not required for `real-alice-launch-smoke`; only required for non-baseline scenario ids |
| `TMPDIR` | For deep worktrees | Set to `/tmp` to avoid Unix socket path length failures |

### Required host dependencies

The scenario declares these capabilities in its YAML:

| Dependency | Why |
| --- | --- |
| `rust-cli` | Build and run eatme commands |
| `java-21` | Alice requires Java 21 |
| `maven` | Package Alice through Maven |
| `xvfb` | Virtual display for headless launch |
| `xdpyinfo` | Verify display responsiveness |
| `wmctrl` | Window list evidence |
| `screenshot-tool` | Capture startup screenshot |

Check host readiness:

```bash
cargo run -q -p eatme-cli -- deps check --json
```

## API reference

### Rust API

The public entry point is `check_lesson_session_readiness`:

```rust
use eatme_alice::check_lesson_session_readiness;
use std::path::Path;

let manifest_path = Path::new("runs/comparisons/real-alice-launch-smoke/local/comparison-manifest.json");
let report = check_lesson_session_readiness(manifest_path)?;

assert!(report.passed);
assert_eq!(report.status, "ready");
assert_eq!(report.scenario_id.as_deref(), Some("real-alice-launch-smoke"));
```

### Report struct fields

The full `LessonSessionReadinessReport` struct serializes these fields:

| Field | Type | Description |
| --- | --- | --- |
| `schema_version` | `String` | Always `"eatme.alice-lesson-session-readiness/v1"` |
| `manifest_path` | `String` | Display path of the input comparison manifest |
| `scenario_id` | `Option<String>` | Always `Some("real-alice-launch-smoke")` for this path |
| `passed` | `bool` | `true` when all required evidence is present |
| `status` | `String` | `"ready"` or `"not_ready"` (normalized) |
| `readiness_status` | `String` | `"ready"` or `"incomplete"` (raw) |
| `blocked_reason` | `Option<String>` | Always `None` for launch-smoke path |
| `human_summary` | `String` | One-sentence summary for display |
| `desktop_proof_contract` | `DesktopProofContract` | Structured proof status (`verified`, `unsupported_environment`, or `launched_but_unverified`) |
| `shown_evidence` | `Vec<ReadinessEvidenceItem>` | Evidence items with `present` state |
| `not_yet_shown` | `Vec<ReadinessEvidenceItem>` | Evidence items not yet present |
| `desktop_next_action` | `Option<DesktopNextActionSummary>` | Always `None` for launch-smoke path |
| `original_alice_action_evidence` | `OriginalAliceActionEvidenceReport` | Original Alice action evidence availability |
| `unproven_claims` | `Vec<String>` | 8 claims explicitly not made by this report |
| `evidence_progress` | `LessonReadinessEvidenceProgress` | Item-level progress tracking |
| `evidence_boundaries` | `Vec<FirstLessonEvidenceBoundary>` | Always empty for launch-smoke path |
| `required_evidence` | `Vec<String>` | The 5 required evidence labels |
| `no_go_contracts` | `Vec<LessonSessionNoGoContract>` | Always empty for launch-smoke path |
| `lesson_session_readiness` | `LessonSessionReadinessEnvelope` | Session-level readiness envelope |
| `role_readiness` | `Vec<LessonSessionReadinessEnvelope>` | Per-role (baseline, modernized) readiness |
| `contract_check` | `LessonSessionContractCheck` | Contract validation metadata |
| `execute_requested` | `Option<bool>` | Whether `--execute` was used in comparison |
| `target_evidence` | `Vec<LessonTargetEvidence>` | Per-target evidence detail |
| `issues` | `Vec<String>` | Empty when ready; describes each gap otherwise |
| `limitations` | `Vec<String>` | 8 unproven claim sentences + 6 operational limitations |

### Evidence progress struct

| Field | Type | Description |
| --- | --- | --- |
| `total_required` | `usize` | Always 5 for launch-smoke |
| `present` | `usize` | Items with evidence found |
| `missing` | `usize` | Items with no evidence |
| `invalid` | `usize` | Items with malformed evidence |
| `not_observed` | `usize` | Items not yet checked |
| `blocked` | `usize` | Items blocked by prerequisite failure |
| `summary` | `String` | Human-readable progress sentence |
| `next_actionable_blocker` | `Option<String>` | First issue to resolve, if any (omitted from JSON when `None`) |
| `next_missing_real_desktop_proof` | `Option<String>` | Next missing desktop proof item (omitted from JSON when `None`; always `None` for launch-smoke) |
| `items` | `Vec<LessonReadinessEvidenceProgressItem>` | Per-item detail |

### Required assertions

The launch-smoke path requires these five assertions to be present and passed in
each target's `launch_manifest.assertions`:

1. `display_responsive` — the virtual display accepted connections
2. `process_started` — the Alice Java process launched
3. `startup_screenshot` — a screenshot was captured
4. `no_fatal_logs` — no fatal-level log entries were found
5. `real_alice_execution_evidence` — deterministic execution evidence was recorded

## Evidence progress tracking

The evidence progress tracker maps the five required evidence labels to
observable states:

| State | Meaning |
| --- | --- |
| `present` | Evidence found and valid |
| `missing` | Evidence not found in manifest |
| `invalid` | Evidence found but malformed or failed |
| `not_observed` | Not yet inspected (initial state) |
| `blocked` | Cannot be checked because a prerequisite failed |

Progress is computed per-item against both baseline and modernized targets. The
`next_actionable_blocker` field points to the first issue a human should
investigate when status is `not_ready`.

## Non-claim boundary contract

The readiness mapper explicitly declares what it does **not** prove. These
non-claims are unconditional and always present in the report regardless of
readiness status:

```text
First-lesson completion is not proven.
Full world execution is not proven.
Grading is not proven.
Creative assessment is not proven.
Full Alice UI automation is not proven.
Visible rendering correctness is not proven.
Save completion is not proven.
Deployed sharing/platform success is not proven.
```

These boundaries are enforced by:

- Constants in `claims.rs` (`LAUNCH_SMOKE_UNPROVEN_CLAIMS`)
- Contract tests in `launch_smoke_readiness.rs`
- Documentation tests in `launch_smoke_docs.rs`
- Wording audits in PR recovery tests

The mapping never promotes launch-smoke manifest evidence into a lesson
completion, grading, creative assessment, or UI automation claim.

## Scenario YAML reference

The canonical scenario asset is
`assets/scenarios/eatme/real-alice-launch-smoke.yaml`:

```yaml
schema_version: eatme.scenario/v1
id: real-alice-launch-smoke
title: Real Alice launch smoke
resource_basis:
  - name: Alice 3 setup/download guidance
    url: https://www.alice.org/get-alice/alice-3/
  - name: Alice 3 source
    url: https://github.com/TheAliceProject/alice3
purpose: >-
  Record bounded readiness that the scenario-labeled launch path packaged and
  launched the real Alice desktop application under Xvfb, then captured
  manifest/log/window/screenshot evidence before any agentic classroom behavior
  is trusted. The baseline launch smoke is manifest-level evidence only; it is
  not full UI automation, not creative assessment, and not learner-world grading.
capabilities:
  required:
    - rust-cli
    - java-21
    - maven
    - xvfb
    - xdpyinfo
    - wmctrl
    - screenshot-tool
  optional:
    - glxinfo
adapter:
  targets:
    - eatme-cli
    - gadugi-cli
steps:
  - id: validate-assets
    command: cargo run -q -p eatme-cli -- assets validate --json
    evidence:
      - stdout JSON has passed=true
  - id: check-dependencies
    command: cargo run -q -p eatme-cli -- deps check --json
    evidence:
      - stdout JSON has all_required_available=true
  - id: discover-alice
    command: >-
      cargo run -q -p eatme-cli -- alice discover
      --alice-home ${ALICE_HOME} --json
    evidence:
      - stdout JSON has alice_ide_jar_exists=true
      - stdout JSON has target_lib_exists=true
      - stdout JSON has starter_project_exists=true
  - id: launch-smoke
    command: >-
      cargo run -q -p eatme-cli -- alice launch-smoke
      --alice-home ${ALICE_HOME} --run-id ${RUN_ID}
      --runs-dir runs --timeout 900 --json
      --no-memory --offline-package
      --scenario real-alice-launch-smoke
    evidence:
      - manifest scenario_id equals real-alice-launch-smoke
      - manifest failure_category is null
      - manifest assertions all pass
      - manifest assertions include real_alice_execution_evidence passed=true
      - window-list evidence exists when the window manager can report it
      - screenshot exists and is non-empty
      - log exists and is non-empty
      - manifest/log/window/screenshot evidence is not full UI automation, not creative assessment, and not learner-world grading
timeouts:
  scenario_seconds: 1800
  launch_seconds: 900
artifacts:
  manifest: runs/real-alice-launch-smoke/${RUN_ID}/manifest.json
  screenshot: runs/real-alice-launch-smoke/${RUN_ID}/screenshots/startup.png
  log: runs/real-alice-launch-smoke/${RUN_ID}/alice.log
unsupported_policy: >-
  If host graphics, DISPLAY, Java 21, or Maven prerequisites are missing, fail
  loudly with a missing_dependency category. Do not silently skip.
```

The scenario is validated by `cargo run -q -p eatme-cli -- assets validate --json`
and generates a matching Gadugi adapter checked by
`cargo run -q -p eatme-cli -- assets generate-gadugi --check --json`.

## Examples

### Tutorial: verify readiness from a local comparison run

1. Set up environment:

    ```bash
    export NODE_OPTIONS=--max-old-space-size=32768
    export ALICE_BASELINE_HOME=../alice3-reference
    export ALICE_MODERNIZED_HOME=../alice3-candidate
    ```

2. Run the comparison:

    ```bash
    cargo run -q -p eatme-cli -- alice compare-launch-smoke \
      --run-id my-local-check \
      --json \
      --no-memory \
      --offline-package
    ```

3. Check readiness from the resulting manifest:

    ```bash
    cargo run -q -p eatme-cli -- alice check-lesson-readiness \
      --manifest runs/comparisons/real-alice-launch-smoke/my-local-check/comparison-manifest.json
    ```

4. Inspect evidence progress:

    ```bash
    cargo run -q -p eatme-cli -- alice check-lesson-readiness \
      --manifest runs/comparisons/real-alice-launch-smoke/my-local-check/comparison-manifest.json \
      --json | jq '.evidence_progress'
    ```

### Tutorial: CI integration

Add to your CI workflow:

```yaml
- name: Check launch-smoke readiness
  run: |
    cargo run -q -p eatme-cli -- alice check-lesson-readiness \
      --manifest ${{ env.MANIFEST_PATH }} \
      --json
  env:
    NODE_OPTIONS: --max-old-space-size=32768
```

The command exits non-zero when readiness is `not_ready`, so CI will fail on
missing or broken evidence.

### Tutorial: PR recovery evidence

For PR recovery branches, run the full validation sequence against the current
`HEAD` and confirm the no-op guard:

```bash
export NODE_OPTIONS=--max-old-space-size=32768

# Rust quality gate
TMPDIR=/tmp ./scripts/quality-gates.sh

# Asset and adapter gates
cargo run -q -p eatme-cli -- assets validate --json
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json

# Documentation gate
mkdocs build --strict

# No-op guard (confirms no uncommitted changes)
./scripts/default-workflow-noop-guard.sh
```

Record the final `HEAD` SHA in the PR description as exact-head evidence.

## Troubleshooting

### `status: "not_ready"` with empty issues

The manifest may be missing the `targets` key entirely. Confirm the comparison
ran to completion and wrote both baseline and modernized target entries.

### `failure_category` is set on a target

A non-null `failure_category` means the launch failed at a known stage
(e.g., `screenshot_missing`, `process_crash`). Fix the underlying host issue and
re-run the launch smoke.

### Unix socket path too long

In deep worktrees, set `TMPDIR=/tmp` before running quality gates:

```bash
TMPDIR=/tmp ./scripts/quality-gates.sh
```

### Missing assertions in manifest

Each target must embed all five required assertions. If an assertion key is
missing, the target was likely created by an older CLI version. Re-run the launch
smoke with the current CLI.

### Gadugi adapter out of sync

After modifying the scenario YAML, regenerate adapters:

```bash
cargo run -q -p eatme-cli -- assets generate-gadugi
```

Then verify:

```bash
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
```
