# PASS 1: API contracts omit live CLI surfaces

- **Checklist:** stale documentation (`runtime-topology` × `api-contracts`)
- **Verdict:** FAIL

## Finding
The atlas runtime layer documents several active CLI surfaces that the atlas API-contract layer omits.

## Evidence
- `crates/eatme-cli/src/main.rs:43-64` defines `Assets::GradingReport`, `Alice::CompareLaunchSmoke`, `Alice::CheckLessonSession`, `Alice::CheckLessonReadiness`, and `Alice::RunFirstLessonReadiness` in addition to the older commands.
- `docs/atlas/runtime-topology/README.md:10-17` explicitly documents `Assets::GradingReport`, `Alice::CompareLaunchSmoke`, and `Alice::RunFirstLessonReadiness` as runtime handoffs.
- `docs/atlas/api-contracts/README.md:5-14` lists only `deps check`, `alice discover`, `alice package`, `alice launch-smoke`, `assets validate`, and `assets generate-gadugi`.

## Why this is a bug
Layers 4 and 5 disagree about the public CLI surface. A reader following the API-contract layer would miss real commands that already exist and are wired into the dispatcher.

## Impact
The atlas under-documents the command contract, which makes the behavioral layers incomplete and weakens user-journey traceability for lesson readiness and comparison flows.

## Suggested fix
Update `docs/atlas/api-contracts/README.md` and `api-contracts.mmd/.dot` to include the missing CLI commands and their primary request/response types.
