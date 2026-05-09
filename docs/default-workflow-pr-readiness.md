# Default-workflow PR readiness

## PR #175 evidence contract

This page is the self-contained recovery artifact for PR #175. It records
bounded evidence for the checked-out repository HEAD, GitHub PR metadata, and
current-head executable checks observed during PR #175 default-workflow
recovery.

This is evidence-contract finalization, not validation completion. Treat every
claim below as limited to the command, timestamp, and observed value that
supports it.

## Scope

| Field | Observed value |
| --- | --- |
| Artifact path | `docs/default-workflow-pr-readiness.md` |
| Repository | `rysweet/eatme` |
| PR | [#175 Document evidence artifact contract](https://github.com/rysweet/eatme/pull/175) |
| Local branch | `wave6-evidence-artifact-contract-1778302300` |
| Local upstream | `origin/wave6-evidence-artifact-contract-1778302300` |
| Checked-out local HEAD | `5b1c9f18b474ee61e64f2298c9e0b6d0af4ad301` |
| Checked-out local HEAD short SHA | `5b1c9f1` |
| Observed GitHub PR head | `5b1c9f18b474ee61e64f2298c9e0b6d0af4ad301` |
| Observed base branch | `master` |
| Observed base SHA | `17521c40bb72dd22669b596179327fc5cf307305` |
| Current-head executable evidence capture | `2026-05-09T10:38:36Z` |
| GitHub PR metadata capture | `2026-05-09T10:35Z` |

Within this page, `local HEAD` and `observed PR head` refer to the full SHAs in
this table unless a command output shows the SHA verbatim.

At current capture time, `git rev-parse @{u}` returned the same SHA as the
observed GitHub PR head, `5b1c9f18b474ee61e64f2298c9e0b6d0af4ad301`.
Therefore, this artifact is scoped to PR #175 and the checked-out local HEAD.
It does not claim that future PR #175 heads, checks, reviews, or mergeability
match these observations.

## Readiness evidence

### Local Git observations

Captured with:

```bash
date -u +%Y-%m-%dT%H:%M:%SZ
git branch --show-current
git rev-parse HEAD
git rev-parse --short HEAD
git rev-parse --abbrev-ref --symbolic-full-name @{u}
git rev-parse @{u}
git status --short
```

Observed result at `2026-05-09T10:38:36Z`:

```text
branch=wave6-evidence-artifact-contract-1778302300
head_sha=5b1c9f18b474ee61e64f2298c9e0b6d0af4ad301
head_short=5b1c9f1
upstream=origin/wave6-evidence-artifact-contract-1778302300
upstream_sha=5b1c9f18b474ee61e64f2298c9e0b6d0af4ad301
status_short= M crates/eatme-assets/src/lib.rs
 M docs/default-workflow-pr-readiness.md
?? crates/eatme-assets/src/default_workflow_pr_readiness_contract_tests.rs
```

The dirty status is limited to the readiness artifact and the readiness-contract
test module wired into `eatme-assets`. No unrelated dirty paths were observed at
the capture time.

### GitHub PR #175 observations

Captured with:

```bash
gh pr view 175 --json number,title,state,url,headRefName,headRefOid,baseRefName,baseRefOid,isDraft,mergeStateStatus,mergeable,reviewDecision,statusCheckRollup,latestReviews,updatedAt,createdAt
```

Observed metadata:

| Field | Observed value |
| --- | --- |
| `number` | `175` |
| `title` | `Document evidence artifact contract` |
| `url` | `https://github.com/rysweet/eatme/pull/175` |
| `state` | `OPEN` |
| `isDraft` | `false` |
| `createdAt` | `2026-05-09T05:02:52Z` |
| `updatedAt` | `2026-05-09T10:10:07Z` |
| `headRefName` | `wave6-evidence-artifact-contract-1778302300` |
| `headRefOid` | `5b1c9f18b474ee61e64f2298c9e0b6d0af4ad301` |
| `baseRefName` | `master` |
| `baseRefOid` | `17521c40bb72dd22669b596179327fc5cf307305` |
| `mergeStateStatus` | `CLEAN` |
| `mergeable` | `MERGEABLE` |
| `reviewDecision` | Empty value returned; no approval is claimed. |
| `latestReviews` | Empty list returned; no approval is claimed. |

The `mergeStateStatus` and `mergeable` values are recorded as GitHub metadata
only. They are not treated as approval, required-check sufficiency, or merge
readiness.

### GitHub status-check rollup observation

`gh pr view` returned these `statusCheckRollup` entries for the observed PR head:

| Workflow | Check | Status | Conclusion | Completed |
| --- | --- | --- | --- | --- |
| Documentation Site | Build MkDocs site | `COMPLETED` | `SUCCESS` | `2026-05-09T10:10:24Z` |
| Quality Gates | detect changed files | `COMPLETED` | `SUCCESS` | `2026-05-09T10:10:20Z` |
| Documentation Site | Deploy to GitHub Pages | `COMPLETED` | `SKIPPED` | `2026-05-09T10:10:24Z` |
| Quality Gates | fmt, clippy, module size | `COMPLETED` | `SUCCESS` | `2026-05-09T10:10:56Z` |
| Quality Gates | tests | `COMPLETED` | `SUCCESS` | `2026-05-09T10:13:01Z` |
| Quality Gates | coverage | `COMPLETED` | `SUCCESS` | `2026-05-09T10:13:09Z` |
| Quality Gates | fmt, clippy, tests, module size, coverage | `COMPLETED` | `SUCCESS` | `2026-05-09T10:13:14Z` |
| Quality Gates | manual real Alice launch smoke | `COMPLETED` | `SKIPPED` | `2026-05-09T10:13:15Z` |
| none returned | GitGuardian Security Checks | `COMPLETED` | `SUCCESS` | `2026-05-09T10:10:10Z` |

These entries are per-check observations for the observed PR head. Skipped rows
are explicitly not counted as successful checks, approval, branch-protection
sufficiency, or manual real Alice launch evidence.

### Current-head executable evidence

All Step 8 evidence commands were run on branch
`wave6-evidence-artifact-contract-1778302300` at local HEAD
`5b1c9f18b474ee61e64f2298c9e0b6d0af4ad301` with
`NODE_OPTIONS=--max-old-space-size=32768` and no timeout wrapper.

| Command | Result | Bounded claim |
| --- | --- | --- |
| `cargo run -q -p eatme-cli -- assets validate --json` | Exit `0`; `"passed": true`, `instructor_count: 11`, `student_count: 13`, `core_scenario_count: 25`, `creative_scenario_count: 12`, `scenario_asset_count: 93`, `errors: []`, `warnings: []`. | Persona and scenario assets validate for this checkout. This is asset-contract evidence, not lesson-completion or grading evidence. |
| `cargo run -q -p eatme-cli -- assets generate-gadugi --check --json` | Exit `0`; `"passed": true`, `generated_count: 46`, `checked_count: 46`, `changed: []`, `errors: []`. | Generated Gadugi adapters are current for this checkout. This is adapter freshness evidence, not UI rendering or grading evidence. |
| `mkdocs build --strict` | Exit `0`; MkDocs cleaned `site` and built documentation in `0.36` seconds. | The documentation site renders under strict MkDocs rules for this checkout. |
| `TMPDIR=/tmp ./scripts/quality-gates.sh` | Exit `0`; ran `cargo fmt`, `cargo clippy`, `cargo test`, module-size check, and `cargo llvm-cov`; the new readiness-contract tests passed in both `cargo test` and coverage runs, and the coverage summary reported `TOTAL` line coverage `86.34%`. | The repository quality gate passes for this checkout. This does not prove manual real Alice desktop launch, full UI automation, visual rendering correctness, grading, creative assessment, or lesson completion. |

### Historical same-head outside-in testing evidence

The Step 16b user-path commands below were previously run from the same branch
and same HEAD. They are retained as same-head history for silver-thread/e2e
context, not as newly executed Step 8 proof. The current Step 8 executable
evidence is the command table above.

The `@wave6-evidence-artifact-contract-1778302300` install target in these
commands was a branch ref as resolved at execution time, not an immutable
SHA-pinned install reference. The same-head claim depends on the recorded
execution context, not on the install URL alone.

```bash
uvx --from git+https://github.com/rysweet/eatme.git@wave6-evidence-artifact-contract-1778302300 amplihack <command>
```

| Scenario | Command | Result | Key output | Fix count |
| --- | --- | --- | --- | --- |
| Simple asset validation | `uvx --from git+https://github.com/rysweet/eatme.git@wave6-evidence-artifact-contract-1778302300 amplihack assets validate --json` | Exit `0` | `"passed": true`, `instructor_count: 11`, `student_count: 13`, `scenario_asset_count: 93`, `errors: []`, `warnings: []` | 0 |
| Integration manifest contract | `uvx --from git+https://github.com/rysweet/eatme.git@wave6-evidence-artifact-contract-1778302300 amplihack alice compare-launch-smoke --scenario first-lessons-real-ui-actions --run-id step16b-manifest-contract --runs-dir /tmp/eatme-step16b-compare-runs --json` followed by `uvx --from git+https://github.com/rysweet/eatme.git@wave6-evidence-artifact-contract-1778302300 amplihack alice check-lesson-session --manifest /tmp/eatme-step16b-compare-runs/comparisons/first-lessons-real-ui-actions/step16b-manifest-contract/comparison-manifest.json --json` | Exit `0` for both commands | Comparison manifest used `execute_requested: false`, `functionality_result: not_measured`, and `timing_result: not_measured`; contract check returned `"passed": true`, `automation_status: action_contract_blocked_until_ui_automation`, `issues: []` | 0 |
| Readiness nonclaim probe | `uvx --from git+https://github.com/rysweet/eatme.git@wave6-evidence-artifact-contract-1778302300 amplihack alice run-first-lesson-readiness --run-id step16b-current-head --runs-dir /tmp/eatme-step16b-runs --json` | Exit `1` | `"passed": false`, `status: not_ready`, `readiness_status: incomplete`, desktop proof `reason_code: execute_not_requested`, `2 of 10 required evidence items are present; 8 missing`, and `Error: first-lesson readiness sequence incomplete` | 0 |

The readiness nonclaim probe is recorded because it documents fail-closed
behavior when real Alice desktop execution is not requested. It is not counted
as merge readiness, real-desktop proof, full first-lesson readiness, or a
successful outside-in scenario.

## Review evidence

### Location review

`mkdocs.yml` includes this artifact in the documentation navigation:

```yaml
- Default-workflow PR Readiness: default-workflow-pr-readiness.md
```

Repository search for `Default-workflow PR Readiness`,
`default-workflow-pr-readiness`, `evidence artifact contract`, and `PR
readiness` across `docs/` and `mkdocs.yml` found this page, its MkDocs nav
entry, and links from `docs/index.md`; no stronger PR-specific artifact location
was observed. The artifact therefore remains at
`docs/default-workflow-pr-readiness.md`.

### Content review

The Step 8 review keeps this page as an evidence contract and checks that it:

1. Scopes observations to PR #175 and the current local/PR head.
2. Keeps local Git evidence, GitHub PR metadata, status-check metadata, and
   executable evidence in separate sections.
3. Lists skipped, not-measured, no-execute, and historical states as nonclaims
   instead of implying success.
4. Records the no-execute readiness probe as historical fail-closed behavior,
   not as a product readiness success.
5. Provides explicit nonclaims so recovery can continue without prior
   rate-limited session context.

### Security review

The Step 8 security review treated local Git output, GitHub PR metadata, and
command output as untrusted evidence. No security issue requiring
source, workflow, or credential-handling changes was found in this artifact.

| Checklist item | Result | Evidence or mitigation |
| --- | --- | --- |
| Input validation | Pass | SHA-like values in scope are recorded as observed 40-character lowercase hexadecimal values, except the explicitly labeled 7-character short SHA. Timestamps are recorded in UTC ISO format, and ambiguous PR fields are kept as metadata rather than readiness claims. |
| Output encoding | Pass | Command text is fenced as `bash` or wrapped in Markdown tables/code spans; observed dynamic values are plain text, not raw HTML. |
| Authentication/authorization | Pass | The artifact records fixed read-only `git`, `gh pr view`, local validation, and historical branch-installed `uvx` observations. It does not merge, approve, alter workflows, or modify PR state. |
| Sensitive data handling | Pass | The page includes bounded recovery evidence: repository, PR, branch, SHA, status-check, command, and nonclaim data. It does not include tokens, environment dumps, auth configuration, private config, or unrelated local file contents. |
| No hardcoded secrets | Pass | No password, token, API key, credential, or secret literal was observed. |
| Proper error messages | Pass | Skipped, historical, not-measured, no-execute, and unavailable states are recorded as nonclaims rather than success-shaped fallbacks, stack traces, credential-bearing errors, or hidden failures. |

## Finalization evidence

PR #175 remains unmerged. The observed GitHub PR state is `OPEN`, the observed
head ref is `wave6-evidence-artifact-contract-1778302300`, and the observed PR
head SHA is `5b1c9f18b474ee61e64f2298c9e0b6d0af4ad301`.

No manual merge was performed. This recovery only updates workflow readiness/review/finalization evidence and the executable readiness-contract tests
that guard it.

Finalization status: `limited-ready` for PR #175 evidence-contract recovery at
the observed head. This means the artifact records current-head executable
evidence, review boundaries, and explicit nonclaims sufficient for workflow
handoff. It does not mean the PR is approved, merged, branch-protection
sufficient, or validated for UI automation, rendering correctness, grading,
creative assessment, or lesson completion.

## Nonclaims

- No merge readiness is claimed.
- No PR approval is claimed.
- No blanket CI success is claimed beyond the listed GitHub status-check rollup.
- No test coverage sufficiency is claimed beyond the reported current-head
  coverage summary.
- No local quality-gate claim is made beyond the observed
  `TMPDIR=/tmp ./scripts/quality-gates.sh` exit `0`.
- No real Alice desktop execution is claimed.
- No full Alice UI automation is claimed.
- No full first-lesson readiness is claimed.
- No first-lesson completion is claimed.
- No Save completion is claimed.
- No visible rendering correctness is claimed.
- No grading or creative assessment is claimed.
- No claim is made that skipped checks are acceptable for merge readiness.
- No claim is made that branch-protection requirements were satisfied.
- No claim is made that future PR #175 heads, checks, reviews, or mergeability
  match the observations recorded here.
- No prior rate-limited/default-workflow session context is required to continue
  recovery from this artifact.
