# PASS 1: docs still teach the old scenario-asset count

This file is a historical snapshot of an older audit. Its counts were true for that audit only.
The current inventory is 57 canonical EatMe scenarios, 58 Gadugi scenarios, and
115 scenario YAML files in total.

- **Checklist:** stale documentation (docs spot-check)
- **Verdict:** RESOLVED (historical)

## Finding
Multiple docs pages previously hardcoded `93` scenario assets even though the validator and generated adapters later used `105`. Current validation uses `115`.

## Evidence
- Historical PASS 1 validation returned `"scenario_asset_count": 107`.
- Current validation uses `"scenario_asset_count": 115`.
- Current generated adapters expect `"scenario_asset_count": 115`.

## Why this is a bug
This report is retained as historical audit evidence. It is not an active failure because current examples and adapters use the validated `115` inventory.

## Impact
This can trigger false stale-doc conclusions, bad manual checks, and confusion when `assets validate --json` reports a larger count than the docs promise.

## Suggested fix
Keep future examples and inventory tables aligned with the current validated count of 115.
