# Sharing and platform behavior report

The `sharing_platform` module evaluates which sharing and deployment features are
available for the Building a Scene first-lesson scenario. It checks asset
validity, host dependency availability, and four sharing/deployment features,
then outputs a structured JSON readiness report with per-feature status and
explicit dependency tracking.

The sharing platform report is a **feature readiness report**, not a deployment
gate. It answers "which sharing features work on this host?" — not "has the
student shared their world?" Two features (export-a3w, file-sharing) are
evaluable from precondition status. Two features (web-sharing, classroom-deploy)
are always platform-blocked in the current Alice desktop environment.

## Contents

- [Usage](#usage)
- [Output schema](#output-schema)
- [Feature entries](#feature-entries)
- [Feature dependency graph](#feature-dependency-graph)
- [Status semantics](#status-semantics)
- [Pass logic](#pass-logic)
- [API reference](#api-reference)
- [Configuration](#configuration)
- [Examples](#examples)
- [Troubleshooting](#troubleshooting)
- [Related documentation](#related-documentation)

## Usage

The sharing platform report is produced by calling the Rust API directly from
the CLI or from test code. It is a pure function that takes pre-computed
validation and dependency results and returns a structured readiness report.

```rust
use eatme_assets::sharing_platform::{
    check_sharing_platform_readiness,
    SharingPlatformInput,
};

let input = SharingPlatformInput {
    assets_valid: true,
    asset_reason: "All 115 scenario assets passed validation".into(),
    deps_available: true,
    deps_reason: "All required dependencies available".into(),
};

let report = check_sharing_platform_readiness(input);
println!("{}", serde_json::to_string_pretty(&report).unwrap());
```

The function evaluates six entries in dependency order:

1. **validate-assets** — checks committed scenario and persona assets.
   No dependencies (root entry).
2. **check-dependencies** — checks host tools required by real Alice launch
   smokes. No dependencies (root entry).
3. **export-a3w** — evaluates whether the Alice environment can export `.a3w`
   world files. Depends on `validate-assets` and `check-dependencies`.
4. **file-sharing** — evaluates whether exported worlds can be shared via file
   system. Depends on `export-a3w` (cascading — can't share what you can't
   export).
5. **web-sharing** — reports whether worlds can be shared via web link.
   Always `platform-blocked` (Alice desktop has no web export). No dependencies.
6. **classroom-deploy** — reports whether worlds can be deployed to a classroom
   server. Always `platform-blocked` (no classroom deployment server exists).
   No dependencies.

The function does not perform I/O. It evaluates readiness from pre-computed
input and hardcoded platform constraints.

## Output schema

The report serializes to structured JSON:

```json
{
  "schema_version": "eatme.assets/sharing-platform/v1",
  "lesson": "building-a-scene-first-world",
  "passed": true,
  "entries": [
    {
      "feature": "validate-assets",
      "status": "ready",
      "depends_on": [],
      "reason": "All 115 scenario assets passed validation"
    },
    {
      "feature": "check-dependencies",
      "status": "ready",
      "depends_on": [],
      "reason": "All required dependencies available"
    },
    {
      "feature": "export-a3w",
      "status": "ready",
      "depends_on": ["validate-assets", "check-dependencies"],
      "reason": "All preconditions met for .a3w export"
    },
    {
      "feature": "file-sharing",
      "status": "ready",
      "depends_on": ["export-a3w"],
      "reason": "File sharing available — export-a3w is ready"
    },
    {
      "feature": "web-sharing",
      "status": "platform-blocked",
      "depends_on": [],
      "reason": "Alice desktop does not support web sharing"
    },
    {
      "feature": "classroom-deploy",
      "status": "platform-blocked",
      "depends_on": [],
      "reason": "No classroom deployment server available"
    }
  ]
}
```

| Field | Type | Description |
| --- | --- | --- |
| `schema_version` | string | Always `eatme.assets/sharing-platform/v1`. |
| `lesson` | string | The lesson scenario id. Always `building-a-scene-first-world`. |
| `passed` | bool | `true` when both `export-a3w` and `file-sharing` are `ready`. See [Pass logic](#pass-logic). |
| `entries` | array | Ordered list of `FeatureEntry` objects. |
| `entries[].feature` | string | Feature identifier. |
| `entries[].status` | string | One of `ready`, `blocked`, or `platform-blocked`. |
| `entries[].depends_on` | array of strings | Feature names this entry depends on. Empty array `[]` for root entries and platform-blocked entries. |
| `entries[].reason` | string | Human-readable explanation of the status. |

## Feature entries

The sharing platform report evaluates six entries. The first two are
**precondition entries** that can be fully evaluated from pre-computed results.
The next two are **evaluable sharing features** whose status cascades from
preconditions. The last two are **platform-blocked features** that are always
blocked regardless of precondition status.

### Precondition entries

| Feature | What it checks | Ready when |
| --- | --- | --- |
| `validate-assets` | Committed persona and scenario assets | `assets_valid` is `true` |
| `check-dependencies` | Host tools for real Alice smoke runs | `deps_available` is `true` |

Precondition entries are replicated locally — the sharing platform module does
not import from the grading report module. Both modules evaluate the same
logical conditions independently to avoid coupling.

### Evaluable sharing features

| Feature | What it evaluates | Ready when |
| --- | --- | --- |
| `export-a3w` | Alice .a3w world export capability | Both `validate-assets` and `check-dependencies` are `ready` |
| `file-sharing` | File system sharing of exported worlds | `export-a3w` is `ready` |

These features cascade from preconditions. `file-sharing` depends on
`export-a3w` because you cannot share a world file that was not exported.

### Platform-blocked features

| Feature | Why it is blocked | Status |
| --- | --- | --- |
| `web-sharing` | Alice desktop has no web export capability | Always `platform-blocked` |
| `classroom-deploy` | No classroom deployment server exists | Always `platform-blocked` |

Platform-blocked features have no dependencies. Their status is determined by
the current Alice platform constraints, not by precondition results. They use
the distinct `platform-blocked` status to differentiate from precondition
failures (`blocked`).

## Feature dependency graph

Features form a partial dependency graph:

```text
validate-assets ─┐
                  ├─→ export-a3w → file-sharing
check-dependencies┘

web-sharing         (independent — always platform-blocked)
classroom-deploy    (independent — always platform-blocked)
```

The `depends_on` field on each entry makes this graph explicit in the JSON
output. Consumers can use the dependency graph to:

- Determine which features are available on the current host.
- Identify the root cause of a blocked feature (trace back through
  `depends_on`).
- Distinguish platform limitations (`platform-blocked`) from fixable
  precondition failures (`blocked`).

## Status semantics

Each entry receives one of three statuses:

| Status | Meaning |
| --- | --- |
| `ready` | Feature is available. Preconditions are met. |
| `blocked` | Feature is unavailable due to failed preconditions. The reason field explains what is missing. Fixable by resolving the upstream failure. |
| `platform-blocked` | Feature is unavailable due to platform limitations. Not fixable by changing preconditions — requires a different Alice platform or deployment target. |

The three-status design is intentional. `blocked` and `platform-blocked` convey
different remediation paths:

- **blocked**: Install missing host tools, fix invalid assets, or resolve
  upstream dependency failures.
- **platform-blocked**: No action available in the current environment. These
  features require web export or classroom server capabilities that Alice
  desktop does not provide.

## Pass logic

The top-level `passed` field considers only the two evaluable sharing features:

```text
passed = (export-a3w == ready) AND (file-sharing == ready)
```

Platform-blocked features (`web-sharing`, `classroom-deploy`) are excluded
from the pass calculation. They are informational — they report what the
platform cannot do, but do not prevent the report from passing. This mirrors
the grading report pattern where `not-yet-tested` steps do not prevent
readiness assessment.

Precondition entries (`validate-assets`, `check-dependencies`) are also
excluded from the direct pass check because their status is already captured
through dependency cascading into `export-a3w`.

## API reference

The sharing platform report is implemented in `eatme-assets` as a pure function
with no side effects.

### Types

```rust
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct SharingPlatformReport {
    pub schema_version: String,
    pub lesson: String,
    pub passed: bool,
    pub entries: Vec<FeatureEntry>,
}

#[derive(Clone, Debug, Serialize)]
pub struct FeatureEntry {
    pub feature: String,
    pub status: FeatureReadiness,
    pub depends_on: Vec<String>,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum FeatureReadiness {
    #[serde(rename = "ready")]
    Ready,
    #[serde(rename = "blocked")]
    Blocked,
    #[serde(rename = "platform-blocked")]
    PlatformBlocked,
}

pub struct SharingPlatformInput {
    pub assets_valid: bool,
    pub asset_reason: String,
    pub deps_available: bool,
    pub deps_reason: String,
}
```

The `FeatureReadiness` enum uses `platform-blocked` (not `PlatformBlocked` in
JSON) to make the distinction clear in serialized output. This is a different
enum from `StepStatus` in the grading report — the sharing platform module has
its own type hierarchy to maintain module independence.

### Function

```rust
pub fn check_sharing_platform_readiness(
    input: SharingPlatformInput,
) -> SharingPlatformReport
```

The function accepts a `SharingPlatformInput` with pre-computed results from
asset validation and dependency checking, then returns a
`SharingPlatformReport` with all six feature entries evaluated. The function is
deterministic and performs no I/O.

`SharingPlatformInput` mirrors the shape of `GradingInput` from the grading
report module but is a separate type. This keeps the sharing platform module
independent — it can be used without importing any grading report types.

### Dependency propagation logic

The function propagates status through the dependency graph:

1. Root entries (`validate-assets`, `check-dependencies`) are evaluated from
   `SharingPlatformInput` fields.
2. `export-a3w` checks its `depends_on` list. If any dependency is `Blocked`,
   `export-a3w` is `Blocked` with a reason listing the blockers.
3. `file-sharing` checks `export-a3w`. If `export-a3w` is `Blocked`,
   `file-sharing` is `Blocked`.
4. `web-sharing` and `classroom-deploy` are always `PlatformBlocked` regardless
   of precondition status.

### Crate boundary

The `eatme-assets` crate owns the sharing platform types and pure evaluation
function. The sharing platform module has no dependency on the grading report
module or on `eatme-alice`.

```text
eatme-assets/src/
  ├── sharing_platform.rs          ← types + check_sharing_platform_readiness()
  ├── sharing_platform_tests.rs    ← comprehensive test suite
  └── lib.rs                       ← module registration + re-exports
```

Both `check_sharing_platform_readiness` and the four public types
(`SharingPlatformReport`, `FeatureEntry`, `FeatureReadiness`,
`SharingPlatformInput`) are re-exported from `eatme_assets`.

## Configuration

The sharing platform report has no runtime configuration. All behavior is
determined by the `SharingPlatformInput` fields and hardcoded platform
constraints.

| Parameter | Source | Value |
| --- | --- | --- |
| `assets_valid` | Pre-computed from `validate_assets()` | `true` or `false` |
| `asset_reason` | Pre-computed from `validate_assets()` | Descriptive string |
| `deps_available` | Pre-computed from `check_dependencies()` | `true` or `false` |
| `deps_reason` | Pre-computed from `check_dependencies()` | Descriptive string |
| Schema version | Hardcoded | `eatme.assets/sharing-platform/v1` |
| Lesson scenario | Hardcoded | `building-a-scene-first-world` |
| Platform-blocked features | Hardcoded | `web-sharing`, `classroom-deploy` |

## Examples

### All preconditions ready — export and file sharing available

When assets are valid and all host dependencies are available:

```json
{
  "schema_version": "eatme.assets/sharing-platform/v1",
  "lesson": "building-a-scene-first-world",
  "passed": true,
  "entries": [
    {
      "feature": "validate-assets",
      "status": "ready",
      "depends_on": [],
      "reason": "All 115 scenario assets passed validation"
    },
    {
      "feature": "check-dependencies",
      "status": "ready",
      "depends_on": [],
      "reason": "All required dependencies available"
    },
    {
      "feature": "export-a3w",
      "status": "ready",
      "depends_on": ["validate-assets", "check-dependencies"],
      "reason": "All preconditions met for .a3w export"
    },
    {
      "feature": "file-sharing",
      "status": "ready",
      "depends_on": ["export-a3w"],
      "reason": "File sharing available — export-a3w is ready"
    },
    {
      "feature": "web-sharing",
      "status": "platform-blocked",
      "depends_on": [],
      "reason": "Alice desktop does not support web sharing"
    },
    {
      "feature": "classroom-deploy",
      "status": "platform-blocked",
      "depends_on": [],
      "reason": "No classroom deployment server available"
    }
  ]
}
```

The `passed` field is `true` because `export-a3w` and `file-sharing` are both
`ready`. The two platform-blocked features do not affect the pass result.

### Blocked by missing dependencies

When host dependencies are missing, the blockage cascades through the
dependency graph:

```json
{
  "schema_version": "eatme.assets/sharing-platform/v1",
  "lesson": "building-a-scene-first-world",
  "passed": false,
  "entries": [
    {
      "feature": "validate-assets",
      "status": "ready",
      "depends_on": [],
      "reason": "All 115 scenario assets passed validation"
    },
    {
      "feature": "check-dependencies",
      "status": "blocked",
      "depends_on": [],
      "reason": "Missing required tools: Xvfb, wmctrl"
    },
    {
      "feature": "export-a3w",
      "status": "blocked",
      "depends_on": ["validate-assets", "check-dependencies"],
      "reason": "Blocked by: check-dependencies"
    },
    {
      "feature": "file-sharing",
      "status": "blocked",
      "depends_on": ["export-a3w"],
      "reason": "Blocked by: export-a3w"
    },
    {
      "feature": "web-sharing",
      "status": "platform-blocked",
      "depends_on": [],
      "reason": "Alice desktop does not support web sharing"
    },
    {
      "feature": "classroom-deploy",
      "status": "platform-blocked",
      "depends_on": [],
      "reason": "No classroom deployment server available"
    }
  ]
}
```

Note that `web-sharing` and `classroom-deploy` remain `platform-blocked`, not
`blocked`. Their status is independent of precondition results.

### Blocked by invalid assets

When committed assets fail validation:

```json
{
  "schema_version": "eatme.assets/sharing-platform/v1",
  "lesson": "building-a-scene-first-world",
  "passed": false,
  "entries": [
    {
      "feature": "validate-assets",
      "status": "blocked",
      "depends_on": [],
      "reason": "Asset validation failed: 2 errors"
    },
    {
      "feature": "check-dependencies",
      "status": "ready",
      "depends_on": [],
      "reason": "All required dependencies available"
    },
    {
      "feature": "export-a3w",
      "status": "blocked",
      "depends_on": ["validate-assets", "check-dependencies"],
      "reason": "Blocked by: validate-assets"
    },
    {
      "feature": "file-sharing",
      "status": "blocked",
      "depends_on": ["export-a3w"],
      "reason": "Blocked by: export-a3w"
    },
    {
      "feature": "web-sharing",
      "status": "platform-blocked",
      "depends_on": [],
      "reason": "Alice desktop does not support web sharing"
    },
    {
      "feature": "classroom-deploy",
      "status": "platform-blocked",
      "depends_on": [],
      "reason": "No classroom deployment server available"
    }
  ]
}
```

### Both preconditions blocked

```json
{
  "schema_version": "eatme.assets/sharing-platform/v1",
  "lesson": "building-a-scene-first-world",
  "passed": false,
  "entries": [
    {
      "feature": "validate-assets",
      "status": "blocked",
      "depends_on": [],
      "reason": "Asset validation failed: 2 errors"
    },
    {
      "feature": "check-dependencies",
      "status": "blocked",
      "depends_on": [],
      "reason": "Missing required tools: Xvfb, wmctrl"
    },
    {
      "feature": "export-a3w",
      "status": "blocked",
      "depends_on": ["validate-assets", "check-dependencies"],
      "reason": "Blocked by: validate-assets, check-dependencies"
    },
    {
      "feature": "file-sharing",
      "status": "blocked",
      "depends_on": ["export-a3w"],
      "reason": "Blocked by: export-a3w"
    },
    {
      "feature": "web-sharing",
      "status": "platform-blocked",
      "depends_on": [],
      "reason": "Alice desktop does not support web sharing"
    },
    {
      "feature": "classroom-deploy",
      "status": "platform-blocked",
      "depends_on": [],
      "reason": "No classroom deployment server available"
    }
  ]
}
```

When both preconditions are blocked, `export-a3w` lists both blockers in its
reason field, matching the grading report convention.

## Troubleshooting

### `export-a3w` is blocked

Trace back through the dependency graph:

1. Check `validate-assets` — if blocked, run `cargo run -q -p eatme-cli --
   assets validate --json` to identify which assets failed.
2. Check `check-dependencies` — if blocked, install the missing host tools
   listed in the reason field.

### `file-sharing` is blocked but `check-dependencies` is ready

This means `validate-assets` is blocked. Fix the asset validation errors and
the blockage will cascade forward through `export-a3w` to `file-sharing`.

### `web-sharing` or `classroom-deploy` show `platform-blocked`

This is expected. These features are not available in the Alice desktop
environment. The `platform-blocked` status is informational and does not affect
the `passed` field. No remediation is available in the current platform.

### `passed` is `false` but platform-blocked features are the only concern

If `export-a3w` and `file-sharing` are both `ready`, `passed` is `true`
regardless of platform-blocked features. If `passed` is `false`, at least one
of `export-a3w` or `file-sharing` is `blocked` — check the dependency chain.

## Related documentation

- [First-Lesson Grading Report](first-lesson-grading-report.md) — the
  precondition readiness report for the Building a Scene first-lesson scenario.
  The sharing platform report evaluates the same preconditions independently.
- [Validation and Quality Gates](validation-quality-gates.md) — the asset
  validation and dependency checking commands used to produce
  `SharingPlatformInput`.
- [Creative Assessment Boundary](creative-assessment-boundary.md) — the
  boundary between machine-assessable and human-review-needed aspects. The
  sharing platform report stays within the machine-assessable boundary.
- [Lesson Readiness Module Boundary](lesson-readiness-module-boundary.md) —
  module boundary conventions for readiness helpers in the `eatme-assets` crate.
