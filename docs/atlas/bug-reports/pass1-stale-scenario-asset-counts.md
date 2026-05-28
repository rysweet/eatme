# PASS 1: docs still teach the old scenario-asset count

- **Checklist:** stale documentation (docs spot-check)
- **Verdict:** FAIL

## Finding
Multiple docs pages still hardcode `93` scenario assets even though the current validator and generated adapters now use `105`.

## Evidence
- PASS 1 validation run: `cargo run -q -p eatme-cli -- assets validate --json` returned `"scenario_asset_count": 105` with empty `errors` and `warnings`.
- `assets/scenarios/gadugi/building-a-scene-first-world.yaml:36-39` now expects `"scenario_asset_count": 105`.
- `docs/generated-asset-consistency.md:52-69` still says the committed inventory has 93 scenario YAML files and shows adapters expecting `"scenario_asset_count": 93`.
- `docs/first-lesson-grading-report.md:69-70` and `:123-124` still show `All 93 scenario assets passed validation`.

## Why this is a bug
The examples are no longer aligned with the repository's validated asset inventory. Readers following the docs will compare against the wrong expected number.

## Impact
This can trigger false stale-doc conclusions, bad manual checks, and confusion when `assets validate --json` reports a larger count than the docs promise.

## Suggested fix
Refresh all examples and inventory tables that still mention 93 so they match the current validated count of 105.
