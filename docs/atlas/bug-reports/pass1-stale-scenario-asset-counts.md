# PASS 1: docs still teach the old scenario-asset count

- **Checklist:** stale documentation (docs spot-check)
- **Verdict:** FAIL

## Finding
Historical note: this older report found docs with stale scenario asset counts. Current docs should use `115`.

## Evidence
- Current validation uses `"scenario_asset_count": 115`.
- Current generated adapters expect `"scenario_asset_count": 115` for full asset validation.
- Current grading docs show `All 115 scenario assets passed validation`.

## Why this is a bug
The examples are no longer aligned with the repository's validated asset inventory. Readers following the docs will compare against the wrong expected number.

## Impact
This can trigger false stale-doc conclusions, bad manual checks, and confusion when `assets validate --json` reports a larger count than the docs promise.

## Suggested fix
Keep examples and inventory tables aligned with the current validated count of 115.
