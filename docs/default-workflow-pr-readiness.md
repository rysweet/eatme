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
| Checked-out local HEAD | `d8ddab5bbc443623f0bd49d3e134b37d842b0872` |
| Checked-out local HEAD short SHA | `d8ddab5` |
| Observed GitHub PR head | `d8ddab5bbc443623f0bd49d3e134b37d842b0872` |
| Observed base branch | `master` |
| Observed base SHA | `17521c40bb72dd22669b596179327fc5cf307305` |
| Current-head executable evidence capture | `2026-05-09T18:52:56Z` |
| GitHub PR metadata capture | `2026-05-09T18:52:56Z` |

Within this page, `local HEAD` and `observed PR head` refer to the same full
SHA in this table unless a command output shows a SHA verbatim.

At current capture time, the checked-out local HEAD and GitHub PR `headRefOid`
both resolved to `d8ddab5bbc443623f0bd49d3e134b37d842b0872`. Therefore, the
GitHub check rollup, mergeability metadata, and review metadata below are
current-head evidence for the same commit. This page does not claim that future
PR #175 heads, checks, reviews, or mergeability match these observations.

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

Observed result at `2026-05-09T18:52:56Z`:

```text
branch=wave6-evidence-artifact-contract-1778302300
head_sha=d8ddab5bbc443623f0bd49d3e134b37d842b0872
head_short=d8ddab5
upstream=origin/wave6-evidence-artifact-contract-1778302300
upstream_sha=d8ddab5bbc443623f0bd49d3e134b37d842b0872
status_short_begin
status_short_end
```

Current recovery capture starts from a clean local branch that matches the
observed GitHub PR head. No local source or contract-test change is required to
refresh the evidence artifact contract scope.

Historical Step 8 evidence capture reported a clean baseline before this
readiness artifact/test update in an earlier handoff. That historical note also
reported the local branch one commit ahead of upstream at that earlier capture
time:

```text
## wave6-evidence-artifact-contract-1778302300...origin/wave6-evidence-artifact-contract-1778302300 [ahead 1]
```

That historical clean baseline is not a claim about the current handoff
worktree. The archived wording, "current handoff intentionally contains only
these two pending readiness files," applied only to that earlier Step 8 handoff:

```text
docs/default-workflow-pr-readiness.md
crates/eatme-assets/src/default_workflow_pr_readiness_contract_tests.rs
```

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
| `updatedAt` | `2026-05-09T18:41:45Z` |
| `headRefName` | `wave6-evidence-artifact-contract-1778302300` |
| `headRefOid` | `d8ddab5bbc443623f0bd49d3e134b37d842b0872` |
| `baseRefName` | `master` |
| `baseRefOid` | `17521c40bb72dd22669b596179327fc5cf307305` |
| `mergeStateStatus` | `CLEAN` |
| `mergeable` | `MERGEABLE` |
| `reviewDecision` | Empty value returned; owner-free finalization does not require approval evidence. |
| `latestReviews` | Empty list returned; no human approval is claimed. |

The `mergeStateStatus` and `mergeable` values are recorded as GitHub metadata
only. They are treated as merge-readiness evidence only in combination with the
current-head green check rollup below and the focused evidence-artifact scope.

### GitHub status-check rollup observation

`gh pr view` returned these `statusCheckRollup` entries for the observed PR head:

| Workflow | Check | Status | Conclusion | Completed |
| --- | --- | --- | --- | --- |
| Documentation Site | Build MkDocs site | `COMPLETED` | `SUCCESS` | `2026-05-09T11:52:04Z` |
| Quality Gates | detect changed files | `COMPLETED` | `SUCCESS` | `2026-05-09T11:51:57Z` |
| Documentation Site | Deploy to GitHub Pages | `COMPLETED` | `SKIPPED` | `2026-05-09T11:52:04Z` |
| Quality Gates | fmt, clippy, module size | `COMPLETED` | `SUCCESS` | `2026-05-09T11:52:31Z` |
| Quality Gates | tests | `COMPLETED` | `SUCCESS` | `2026-05-09T11:54:38Z` |
| Quality Gates | coverage | `COMPLETED` | `SUCCESS` | `2026-05-09T11:54:41Z` |
| Quality Gates | fmt, clippy, tests, module size, coverage | `COMPLETED` | `SUCCESS` | `2026-05-09T11:54:48Z` |
| Quality Gates | manual real Alice launch smoke | `COMPLETED` | `SKIPPED` | `2026-05-09T11:54:48Z` |
| none returned | GitGuardian Security Checks | `COMPLETED` | `SUCCESS` | `2026-05-09T11:51:46Z` |

These entries are per-check observations for the observed PR head. Skipped rows
are explicitly not counted as successful checks, approval, branch-protection
sufficiency, or manual real Alice launch evidence.

### Current-head executable evidence

Current-head executable evidence uses the GitHub status-check rollup for PR
head `d8ddab5bbc443623f0bd49d3e134b37d842b0872` as the source of truth. The
rollup is complete for that head, contains no failing or pending checks, and is
sufficient for this focused evidence-artifact finalization. Backup local
validation commands remain documented with the required
`NODE_OPTIONS=--max-old-space-size=32768` setting and no timeout wrapper, but
they are not rerun unless GitHub evidence is stale, missing, ambiguous, or local
files change.

| Backup command | Current evidence source | Bounded claim |
| --- | --- | --- |
| `cargo run -q -p eatme-cli -- assets validate --json` | Covered by the current-head Quality Gates `tests` and aggregate successful rollup; rerun locally only if asset evidence becomes ambiguous. | Persona and scenario asset validation is within current-head validation scope. This is asset-contract evidence, not lesson-completion or grading evidence. |
| `cargo run -q -p eatme-cli -- assets generate-gadugi --check --json` | Covered by the current-head Quality Gates `tests` and aggregate successful rollup; rerun locally only if generated-adapter evidence becomes ambiguous. | Generated Gadugi adapter freshness is within current-head validation scope. This is adapter freshness evidence, not UI rendering or grading evidence. |
| `mkdocs build --strict` | Current-head Documentation Site `Build MkDocs site` completed with `SUCCESS`. | The documentation site renders under strict MkDocs rules for the current PR head. |
| `TMPDIR=/tmp ./scripts/quality-gates.sh` | Current-head Quality Gates aggregate `fmt, clippy, tests, module size, coverage` completed with `SUCCESS`. | The repository quality gate passes for the current PR head. This does not prove manual real Alice desktop launch, full UI automation, visual rendering correctness, grading, creative assessment, or lesson completion. |

### Historical same-head outside-in testing evidence

The Step 16b user-path commands below were previously run from this branch at
the recorded head `5b1c9f18b474ee61e64f2298c9e0b6d0af4ad301`. That recorded
head was same-head evidence for an earlier PR capture. It is now historical
silver-thread/e2e context only, not current-head proof and not a substitute for
the current GitHub check rollup above.

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

1. Separately scopes current local executable evidence and GitHub PR metadata.
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
head ref is `wave6-evidence-artifact-contract-1778302300`, and the observed
GitHub PR head SHA is `d8ddab5bbc443623f0bd49d3e134b37d842b0872`. The
checked-out local branch head is the same SHA, and the branch is not ahead of
the observed PR head at capture time.

No manual merge was performed. This recovery only updates workflow
readiness/review/finalization evidence and the executable readiness-contract
tests that guard it.

Finalization status: `merge-ready` for PR #175 evidence-contract recovery. The
previous `limited-ready` state no longer applies because local HEAD and GitHub
PR `headRefOid` now match, GitHub reports `mergeStateStatus: CLEAN`,
`mergeable: MERGEABLE`, and the current-head status-check rollup has completed
with successful required evidence checks and only explicitly skipped optional
deployment/manual-smoke rows. This means the artifact records executable
evidence, review boundaries, and explicit nonclaims sufficient for owner-free
workflow merge action. It does not mean the PR is already approved, merged, or
validated for UI automation, rendering correctness, grading, creative
assessment, or lesson completion.

## Nonclaims

- No PR approval is claimed.
- No blanket CI success is claimed beyond the listed current-head GitHub
  status-check rollup.
- No test coverage sufficiency is claimed beyond the reported current-head
  coverage summary.
- No local quality-gate rerun is claimed beyond the current-head GitHub Quality
  Gates rollup.
- No real Alice desktop execution is claimed.
- No full Alice UI automation is claimed.
- No full first-lesson readiness is claimed.
- No first-lesson completion is claimed.
- No Save completion is claimed.
- No visible rendering correctness is claimed.
- No grading or creative assessment is claimed.
- No claim is made that skipped checks are successful checks.
- No claim is made that GitHub has observed local commits beyond the recorded PR
  `headRefOid`.
- No claim is made that future PR #175 heads, checks, reviews, or mergeability
  match the observations recorded here.
- No prior rate-limited/default-workflow session context is required to continue
  recovery from this artifact.
