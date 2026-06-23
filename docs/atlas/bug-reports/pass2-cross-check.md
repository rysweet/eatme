# Pass 2 Cross-Check

This file is a historical snapshot of an older audit. Counts in it were true
for that audit only. The current inventory is 57 canonical EatMe scenarios, 58
Gadugi scenarios, and 115 scenario YAML files in total.

## Summary

| Pass 1 finding | Pass 2 verdict | Notes |
| --- | --- | --- |
| `pass1-api-contracts-cli-gaps.md` | **CONFIRMED** | Layer 5 still omits live CLI commands wired in `eatme-cli`. |
| `pass1-cli-usage-three-step-drift.md` | **CONFIRMED** | Quick-reference docs still describe a three-step report while code returns six steps. |
| `pass1-dead-core-root-reexports.md` | **NEEDS_ATTENTION** | The root re-exports are unused by first-party consumers, but crate tests explicitly preserve them as supported API. |
| `pass1-invented-a3p-grading-flow.md` | **NEEDS_ATTENTION** | The `.a3p` parser/grading path exists in test/integration code, but not in the shipped CLI contract described by the atlas. |
| `pass1-stale-scenario-asset-counts.md` | **CONFIRMED** | Historical snapshot only; current docs use 57 canonical, 58 Gadugi, and 115 total. |
| `pass1-web-load-is-synthetic.md` | **CONFIRMED** | Web-platform `Load` is still local bookkeeping, not a REST call. |

## Cross-checks

### `pass1-api-contracts-cli-gaps.md`

**Pass 1 verdict:** FAIL — API contracts omit live CLI surfaces  
**Pass 2 verdict:** **CONFIRMED**

**Rationale:** `crates/eatme-cli/src/main.rs:44-64` and `crates/eatme-cli/src/main.rs:198-300` define and dispatch `assets grading-report`, `alice compare-launch-smoke`, `alice check-lesson-session`, `alice check-lesson-readiness`, and `alice run-first-lesson-readiness`. `docs/atlas/runtime-topology/README.md:13-17` also documents these handoffs, but `docs/atlas/api-contracts/README.md:5-15` still lists only the older six-command subset. Layer 5 is therefore incomplete relative to live CLI code.

### `pass1-cli-usage-three-step-drift.md`

**Pass 1 verdict:** FAIL — `cli-usage.md` contradicts the grading command's six-step contract  
**Pass 2 verdict:** **CONFIRMED**

**Rationale:** `docs/cli-usage.md:96-102` still says `assets grading-report` evaluates only `validate-assets`, `check-dependencies`, and `launch-smoke`. The implementation in `crates/eatme-assets/src/grading_report.rs:68-107` adds `place-object`, `edit-code`, and `run-world`, and `crates/eatme-cli/src/grading.rs:56-63` always returns that six-step report. `docs/first-lesson-grading-report.md:35-54` matches the code, so the drift is real and limited to the quick-reference page.

### `pass1-dead-core-root-reexports.md`

**Pass 1 verdict:** FAIL — dead `eatme-core` root re-exports  
**Pass 2 verdict:** **NEEDS_ATTENTION**

**Rationale:** The public root re-exports are real: `crates/eatme-core/src/lib.rs:8-12` exposes `Program`, `Procedure`, `Statement`, `CodeComment`, `CollaborativeProject`, `EditSession`, and `NavigationTarget`. A workspace search found no non-`eatme-core` imports of those names from the crate root, so Pass 1 was right about first-party non-use. But `crates/eatme-core/src/lib.rs:24-83` contains explicit tests that preserve the root re-export surface, which makes this look like an intentional public API commitment rather than obviously removable dead code. This should stay open for human review, not be treated as an automatic cleanup.

### `pass1-invented-a3p-grading-flow.md`

**Pass 1 verdict:** FAIL — atlas invents an A3P grading pipeline that the CLI does not run  
**Pass 2 verdict:** **NEEDS_ATTENTION**

**Rationale:** The shipped CLI still does not ingest a saved student project: `crates/eatme-cli/src/grading.rs:31-63` builds the report from `validate_assets(...)` plus `check_dependencies(...)` only. But the broader codebase does contain a real `.a3p` parser and grading path in test/integration code: `crates/eatme-alice/tests/real_a3p_pipeline_integration.rs:6-12` imports `parse_a3p_program`, `crates/eatme-alice/tests/real_a3p_pipeline_integration.rs:105-178` parses a ZIP-backed `program.xml`, and `crates/eatme-alice/tests/a3p_parser_support.rs:209-220` documents the AST extraction path. So the atlas is overstating a user-facing contract, but it is not describing a wholly fictional implementation.

### `pass1-stale-scenario-asset-counts.md`

**Pass 1 verdict:** FAIL — docs still teach the old scenario-asset count  
**Pass 2 verdict:** **CONFIRMED**

**Rationale:** This is a historical snapshot of an older audit. Its counts were true for that audit only. The current inventory is 57 canonical EatMe scenarios, 58 Gadugi scenarios, and 115 scenario YAML files in total.

### `pass1-web-load-is-synthetic.md`

**Pass 1 verdict:** FAIL — web-platform reload coverage is synthetic, not REST-backed  
**Pass 2 verdict:** **CONFIRMED**

**Rationale:** `docs/web-platform-testing.md:86-97` still promises save/reload and instructor-review coverage. In actual code, `crates/eatme-alice/tests/web_platform_curriculum_e2e.rs:196-221` sends `POST /api/project/save` for `Step::Save`, but `Step::Load` only compares the remembered `saved_path` and `saved_count` in memory and returns `load({path})` without any HTTP request. `docs/atlas/api-contracts/README.md:24-34` likewise documents save but no load endpoint.

## New findings Pass 1 missed

### New bug: layer 7 shows a live desktop reopen edge that `run_launch_smoke` never executes

**Verdict:** **NEW BUG**

**Evidence:**
- `docs/atlas/service-components/eatme-alice.mmd:41-44` shows `run_window --> save` and `save -. optional .-> reopen`.
- `crates/eatme-alice/src/launch.rs:426-451` calls `probe_project_save_hook(...)` and writes the UI-action contract, but never calls `probe_project_reopen_hook(...)`.
- `crates/eatme-alice/src/launch_reopen_project.rs:62-68` defines the reopen probe, and `crates/eatme-alice/src/launch_save_reopen_contract_tests.rs:1-30` exercises it only in tests.

**Why this matters:** The atlas currently presents reopen as part of the live `eatme-alice` launch pipeline, but actual production wiring stops at save. That makes the service-component diagram stronger than the runtime path the code executes.
