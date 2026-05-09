# Default-workflow PR readiness

## PR #175 current-head evidence artifact

This is the recovery handoff artifact for PR #175 only. It records
command-backed observations from the checked-out repository HEAD and read-only
GitHub PR metadata observed during this recovery step.

Use this page as the self-contained evidence contract for continuing PR #175
recovery after the prior rate-limited session. Do not treat it as merge
approval, CI approval, coverage proof, or validation completion.

## Scope

| Field | Observed value |
| --- | --- |
| Evidence timestamp | `2026-05-09T09:40:15Z` |
| Repository | `rysweet/eatme` |
| PR | [#175 Document evidence artifact contract](https://github.com/rysweet/eatme/pull/175) |
| Local branch | `wave6-evidence-artifact-contract-1778302300` |
| Upstream branch | `origin/wave6-evidence-artifact-contract-1778302300` |
| Checked-out HEAD | `fb92a08c034f8f43dd8d1b7edc32d084d5596b3d` |
| Checked-out HEAD short SHA | `fb92a08` |

The evidence applies only to the observed repository checkout, local working
tree state, and GitHub PR metadata recorded on this page. Future PR head
changes require a new evidence artifact update.

## Readiness evidence

### Local Git observations

The local repository state was captured with this fixed read-only command set:

```bash
date -u +%Y-%m-%dT%H:%M:%SZ
git branch --show-current
git rev-parse HEAD
git rev-parse --short HEAD
git rev-parse --abbrev-ref --symbolic-full-name @{u}
git log -1 --date=iso-strict --format='%H%x09%an%x09%ae%x09%ad%x09%s'
git status --short
```

Observed result:

```text
timestamp_utc=2026-05-09T09:40:15Z
branch=wave6-evidence-artifact-contract-1778302300
head_sha=fb92a08c034f8f43dd8d1b7edc32d084d5596b3d
head_short=fb92a08
upstream=origin/wave6-evidence-artifact-contract-1778302300
latest_commit=fb92a08c034f8f43dd8d1b7edc32d084d5596b3d	Copilot	223556219+Copilot@users.noreply.github.com	2026-05-09T08:31:54Z	wip: checkpoint after review feedback (steps 10-11)
status_short_begin
 M docs/default-workflow-pr-readiness.md
status_short_end
```

The working tree was dirty for this evidence artifact at capture time. The only
observed modified path was this page, `docs/default-workflow-pr-readiness.md`.
No clean-working-tree readiness claim is made.

### GitHub PR #175 observations

PR metadata was captured with this read-only command:

```bash
gh pr view 175 --json number,title,state,url,headRefName,headRefOid,baseRefName,baseRefOid,isDraft,mergeStateStatus,reviewDecision,statusCheckRollup,latestReviews,updatedAt,createdAt
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
| `updatedAt` | `2026-05-09T08:36:48Z` |
| `headRefName` | `wave6-evidence-artifact-contract-1778302300` |
| `headRefOid` | `fb92a08c034f8f43dd8d1b7edc32d084d5596b3d` |
| `baseRefName` | `master` |
| `baseRefOid` | `17521c40bb72dd22669b596179327fc5cf307305` |
| `mergeStateStatus` | `CLEAN` |
| `mergeable` | `MERGEABLE` |
| `reviewDecision` | Empty value returned by `gh`; no approval is claimed. |
| `latestReviews` | Empty list returned by `gh`; no review approval is claimed. |

Local `HEAD` and PR `headRefOid` were both observed as
`fb92a08c034f8f43dd8d1b7edc32d084d5596b3d` at the evidence timestamp. This is
an identity observation only. It is not a merge-readiness claim and it does not
apply to future PR head changes.

### GitHub status-check rollup observation

The following table records `statusCheckRollup` entries returned by
`gh pr view`. It is per-check metadata only, not a blanket CI-success,
required-check sufficiency, or merge-readiness claim.

| Workflow | Check | Status | Conclusion | Completed |
| --- | --- | --- | --- | --- |
| Documentation Site | Build MkDocs site | `COMPLETED` | `SUCCESS` | `2026-05-09T08:37:05Z` |
| Quality Gates | detect changed files | `COMPLETED` | `SUCCESS` | `2026-05-09T08:36:59Z` |
| Documentation Site | Deploy to GitHub Pages | `COMPLETED` | `SKIPPED` | `2026-05-09T08:37:05Z` |
| Quality Gates | fmt, clippy, module size | `COMPLETED` | `SUCCESS` | `2026-05-09T08:37:33Z` |
| Quality Gates | tests | `COMPLETED` | `SUCCESS` | `2026-05-09T08:39:46Z` |
| Quality Gates | coverage | `COMPLETED` | `SUCCESS` | `2026-05-09T08:39:45Z` |
| Quality Gates | fmt, clippy, tests, module size, coverage | `COMPLETED` | `SUCCESS` | `2026-05-09T08:39:53Z` |
| Quality Gates | manual real Alice launch smoke | `COMPLETED` | `SKIPPED` | `2026-05-09T08:39:54Z` |
| none returned | GitGuardian Security Checks | `COMPLETED` | `SUCCESS` | `2026-05-09T08:36:50Z` |

The rollup includes skipped checks. Branch-protection requirements were not
separately queried. This artifact therefore does not claim CI success,
test-coverage sufficiency, required-check sufficiency, or merge readiness.

### Validation command evidence

| Command or check | Execution status in this recovery step | Evidence claim |
| --- | --- | --- |
| `NODE_OPTIONS=--max-old-space-size=32768 mkdocs build --strict` | Executed with exit status `0` during this implementation pass; started `2026-05-09T09:41:07Z`, completed `2026-05-09T09:41:08Z`; MkDocs cleaned `site` and built documentation in `0.59` seconds. This supersedes the earlier `2026-05-09T09:40:00Z`, `2026-05-09T09:37:32Z`, and pre-finalization `2026-05-09T09:35:19Z` runs. | Local documentation rendering succeeded for this recovery step. This is not a PR readiness, CI success, or test coverage claim. |
| `cargo test -q -p eatme-assets default_workflow_attempt_contract_tests` | Executed with exit status `0` during this implementation pass; started `2026-05-09T09:41:07Z`, completed `2026-05-09T09:42:42Z`; result: `6 passed; 0 failed; 0 ignored; 0 measured; 67 filtered out`. | Focused readiness/evidence contract tests passed for this checkout. This is not a full local test-suite, CI-success, coverage, or merge-readiness claim. |
| `cargo run -q -p eatme-cli -- assets validate --json` | Not executed. | No local asset-validation success is claimed. |
| `cargo run -q -p eatme-cli -- assets generate-gadugi --check --json` | Not executed. | No generated-adapter freshness success is claimed. |
| `TMPDIR=/tmp ./scripts/quality-gates.sh` | Not executed. | No local full quality-gate success is claimed. |
| `gh pr checks 175 --watch --interval 10` | Not executed. | No live check-watch completion or required-check sufficiency is claimed. |

The `NODE_OPTIONS=--max-old-space-size=32768` setting is the documented
large-heap configuration used for the MkDocs command in this recovery path.
The MkDocs and focused contract-test evidence rows record command results from
this implementation pass so the artifact does not rely on earlier
pre-finalization builds or prior rate-limited session context.

## Review evidence

### Artifact location review

`docs/default-workflow-pr-readiness.md` is the expected project location for
this PR readiness artifact because `mkdocs.yml` already includes:

```yaml
- Default-workflow PR Readiness: default-workflow-pr-readiness.md
```

No stronger PR-specific evidence artifact convention was observed during this
step, so this existing MkDocs page remains the artifact location.

### Content review

This page was reviewed and kept as a PR #175 recovery handoff rather than a
generic default-workflow procedure. The artifact is scoped to PR #175, the
checked-out branch, and the checked-out HEAD SHA recorded above.

The review confirmed that the artifact:

1. Records command names and observed Git/GitHub values directly.
2. Keeps local validation evidence separate from GitHub metadata.
3. Records the dirty working tree state instead of converting it into a clean
   readiness claim.
4. Records empty `reviewDecision` and empty `latestReviews` values instead of
   claiming approval.
5. Records skipped status checks as skipped and avoids blanket CI-success or
   merge-readiness language.
6. Includes explicit nonclaims so recovery can continue without prior
   rate-limited session context.

### Repository observation used to avoid unsupported claims

The observed PR metadata reports `mergeStateStatus: CLEAN` and
`mergeable: MERGEABLE`, but this artifact does not treat those fields as merge
approval. The observed status-check rollup includes individual `SUCCESS` values,
but this artifact does not treat those values as blanket CI success, coverage
proof, or required-check sufficiency. The observed review fields are empty, so
this artifact does not claim approval.

## Unavailable or not executed checks

| Check | Reason/status | Result claim |
| --- | --- | --- |
| Local asset validation | `cargo run -q -p eatme-cli -- assets validate --json` was not executed during this recovery step. | No local asset-validation success is claimed. |
| Generated Gadugi adapter freshness | `cargo run -q -p eatme-cli -- assets generate-gadugi --check --json` was not executed during this recovery step. | No generated-adapter freshness success is claimed. |
| Full local quality gate | `TMPDIR=/tmp ./scripts/quality-gates.sh` was not executed during this recovery step. | No local full quality-gate success is claimed. |
| PR approval review | `gh pr view` returned an empty `reviewDecision` and empty `latestReviews`. | No approval is claimed. |
| Required-check sufficiency | Branch-protection requirements were not separately queried; only `statusCheckRollup` values were recorded. | No merge-readiness or required-check sufficiency is claimed. |
| Future PR state | PR metadata was captured at the evidence timestamp only. | No claim is made about later PR heads, reviews, checks, or mergeability. |

## Nonclaims

- No merge readiness is claimed.
- No PR approval is claimed.
- No blanket CI success is claimed.
- No full local test-suite success is claimed.
- No test coverage sufficiency is claimed.
- No local asset-validation success is claimed.
- No generated-adapter freshness success is claimed.
- No local full quality-gate success is claimed.
- No clean-working-tree readiness is claimed.
- No claim is made that skipped checks are acceptable for merge readiness.
- No claim is made that branch-protection requirements were satisfied.
- No claim is made that PR #175 remains unchanged after
  `2026-05-09T09:40:15Z`.
- No claim is made that future PR #175 heads equal the checked-out HEAD
  recorded here.
- No claim is made that prior rate-limited/default-workflow session context is
  required to continue recovery.
