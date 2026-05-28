# Pass 3 Journey Verdicts

## Summary

| Journey | Verdict | Reason |
| --- | --- | --- |
| `student-lesson-e2e` | **FAIL** | The atlas ends in parser-driven grading, but the shipped CLI does not wire that path. |
| `scenario-validation` | **PASS** | CLI surface, asset-validation flow, Gadugi generation, and crate wiring all match the code. |
| `web-platform-test` | **PASS** | The documented REST sequence matches the live test harness and the atlas contract layer. |
| `instructor-grading` | **FAIL** | Parser/grading code exists, but only in test/integration paths, not in a documented user-facing contract. |
| `developer-workflow` | **NEEDS_ATTENTION** | Local quality-gate steps are backed by code/config, but the final remote push step was intentionally not exercised. |

## Journey: `student-lesson-e2e`

### Verdict: **FAIL**

| Criterion | Status | Evidence |
| --- | --- | --- |
| Command surface exists | fail | `docs/atlas/user-journeys/student-lesson-e2e.mmd:19-22` claims `CLI -> Parser -> Grade -> CLI`, but `crates/eatme-cli/src/grading.rs:31-63` only calls `validate_assets(...)` and `check_dependencies(...)`. |
| Data-flow matches implementation | fail | `docs/atlas/data-flow/data-flow.mmd:6-10` shows `.a3p -> program.xml -> AST -> grade_* -> GradingReport`, but `crates/eatme-cli/src/grading.rs:35-63` never opens a project file. |
| Service-components are reachable | fail | `docs/atlas/service-components/eatme-cli.mmd:21-31` routes grading through `grading.rs`, but the parser path only appears in test modules such as `crates/eatme-alice/tests/real_a3p_pipeline_integration.rs:105-178` and `crates/eatme-alice/tests/a3p_parser_support.rs:209-220`. |
| Compile-time dependency story holds | warn | `crates/eatme-alice/Cargo.toml:15-21` keeps `regex`, `roxmltree`, `ureq`, and `zip` in `dev-dependencies`, which fits test-only parsing but not a shipped CLI grading path. |
| No dead/test-only critical-path gap | warn | The parser/grading path is real code, but it is not on the public CLI path described by the journey. |

**Verdict Rationale:** The launch-smoke half of the journey is real: `crates/eatme-cli/src/main.rs:218-237` calls `run_launch_smoke(...)`, and `crates/eatme-alice/src/launch.rs:42-496` drives dependency checks, packaging, desktop launch, UI action probes, and save evidence. The failure is the post-save grading half: the atlas says the CLI loads a saved `.a3p` and grades it, but the shipped CLI grading contract in `crates/eatme-cli/src/grading.rs:31-63` is readiness-only.

## Journey: `scenario-validation`

### Verdict: **PASS**

| Criterion | Status | Evidence |
| --- | --- | --- |
| Command surface exists | pass | `docs/atlas/user-journeys/scenario-validation.mmd:9-17` matches `crates/eatme-cli/src/main.rs:173-196`, which dispatches `assets validate` and `assets generate-gadugi`. |
| Data-flow matches implementation | pass | `docs/atlas/data-flow/data-flow.mmd:2-4` matches `crates/eatme-assets/src/validation/scenario.rs:20-63`, `crates/eatme-assets/src/validation/crew.rs:9-19`, and `crates/eatme-assets/src/lib.rs:115-177`. |
| Service-components are reachable | pass | `docs/atlas/service-components/eatme-assets.mmd:21-39` maps discovery, validation, report, and Gadugi generation; `crates/eatme-assets/src/gadugi.rs:20-95` is the concrete generation/check path. |
| Compile-time dependency story holds | pass | `docs/atlas/compile-deps/README.md:10-14` matches `crates/eatme-assets/Cargo.toml:8-13` and `crates/eatme-cli/Cargo.toml:8-15`. |
| No dead code on critical path | pass | `crates/eatme-assets/src/lib.rs:115-177` and `crates/eatme-assets/src/gadugi.rs:20-95` are directly exercised by the CLI route. |

**Verdict Rationale:** This journey is one of the atlas's strongest paths. The command surface, validation flow, recursive scenario discovery (`crates/eatme-assets/src/discovery.rs:5-29`), and Gadugi freshness check all align with the documented sequence.

## Journey: `web-platform-test`

### Verdict: **PASS**

| Criterion | Status | Evidence |
| --- | --- | --- |
| API/command surface exists | pass | `docs/atlas/api-contracts/README.md:24-34` lists `GET /api/health`, `POST /api/launch`, `POST /api/scene/add-object`, `POST /api/code/edit-procedure`, `POST /api/world/run`, `POST /api/project/save`, and `POST /api/events/register`; `crates/eatme-alice/tests/web_platform_curriculum_e2e.rs:151-239` issues exactly those calls. |
| Data-flow matches implementation | pass | `docs/atlas/data-flow/data-flow.mmd:16-18` matches the request/response/assertion loop in `crates/eatme-alice/tests/web_platform_curriculum_e2e.rs:143-245`. |
| Service-components are reachable | pass | `docs/atlas/service-components/eatme-alice.mmd:20-22` and `:46-49` connect the test harness to the TypeScript server; the journey file `docs/atlas/user-journeys/web-platform-test.mmd:8-19` matches that split. |
| Compile-time dependency story holds | pass | `docs/atlas/compile-deps/README.md:11-12` matches `crates/eatme-alice/Cargo.toml:15-21`, where `ureq` is a dev-dependency for the live REST adapter tests. |
| No dead code on critical path | pass | The execute loop and scenario definitions in `crates/eatme-alice/tests/web_platform_curriculum_e2e.rs` are the critical path, and they are directly referenced by the journey. |

**Verdict Rationale:** The documented web-platform journey is accurate for the path it actually claims. The known synthetic-load gap remains real, but it is outside this specific sequence diagram, which stops at save plus event registration.

## Journey: `instructor-grading`

### Verdict: **FAIL**

| Criterion | Status | Evidence |
| --- | --- | --- |
| User-facing surface exists | fail | `docs/atlas/user-journeys/instructor-grading.mmd:10-18` presents an instructor-facing `.a3p -> parser -> AST -> grade -> score -> report` path, but `docs/atlas/api-contracts/README.md:16-34` documents no CLI or REST contract for that flow. |
| Data-flow matches implementation | fail | `docs/atlas/data-flow/data-flow.mmd:6-10` describes the parser/grading chain, but the public CLI grading path in `crates/eatme-cli/src/grading.rs:31-63` is still readiness-only. |
| Service-components are reachable | warn | `docs/atlas/service-components/eatme-assets.mmd:24-39` correctly shows grading and quality-scoring modules, and `crates/eatme-alice/tests/real_a3p_pipeline_integration.rs:105-178` proves the parser path exists, but that path is test/integration-only. |
| Compile-time dependency story holds | warn | The parser-support stack is backed by `crates/eatme-alice/Cargo.toml:15-21` dev-dependencies (`regex`, `roxmltree`, `zip`), reinforcing that this is not a shipped runtime contract. |
| No dead/test-only critical-path gap | fail | `crates/eatme-alice/tests/a3p_parser_support.rs:209-220` documents the `.a3p` parser, but it lives under `tests/`, not under a public CLI/API surface. |

**Verdict Rationale:** The codebase does have real `.a3p` parsing and AST-driven grading machinery, but the atlas journey overstates its delivery status. A human can run the test/integration path; a user cannot reach the same path through the documented CLI or REST contracts.

## Journey: `developer-workflow`

### Verdict: **NEEDS_ATTENTION**

| Criterion | Status | Evidence |
| --- | --- | --- |
| Local workflow surface exists | pass | `docs/atlas/user-journeys/developer-workflow.mmd:9-15` matches `scripts/quality-gates.sh:13-27` and `.pre-commit-config.yaml:4-15`. |
| Data-flow matches implementation | pass | `docs/index.md:37-43` and `docs/validation-quality-gates.md:43-59` describe the same `cargo test`, `cargo clippy`, and pre-commit flow. |
| Component/config reachability | pass | The repo-surface and automation layers back this path through `scripts/quality-gates.sh` and the local pre-commit config. |
| Test-count claim is current | pass | Pass 3 verification ran `cargo test --workspace --all-features -- --list | awk '/: test$/{count++} END{print count}'` and observed `1393`, matching `docs/atlas/user-journeys/developer-workflow.mmd:9-11`. |
| Remote step verified | warn | `docs/atlas/user-journeys/developer-workflow.mmd:16-17` ends with `git push feat/code-atlas`, but this pass intentionally did not push. |

**Verdict Rationale:** The local quality-gate portion of the journey is backed by real scripts and configs, and the hardcoded `1393` test count is currently accurate. The only unverified part is the final remote push step, which is operationally outside the codebase and was explicitly out of scope for this pass.
