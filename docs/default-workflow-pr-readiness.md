# Default-workflow PR readiness

## PR #175 evidence contract

This page is the self-contained recovery artifact for PR #175. It records
bounded evidence for the checked-out repository HEAD and read-only GitHub PR
metadata observed during this recovery step.

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
| Checked-out local HEAD | `7232beddb1ef9b3acf3bcd1fa8f87b0b951555ad` |
| Checked-out local HEAD short SHA | `7232bed` |
| Observed GitHub PR head | `fb92a08c034f8f43dd8d1b7edc32d084d5596b3d` |
| Observed base branch | `master` |
| Observed base SHA | `17521c40bb72dd22669b596179327fc5cf307305` |
| Primary evidence capture | `2026-05-09T09:47:47Z` |
| Local/PR head comparison capture | `2026-05-09T09:48:46Z` |

Within this page, `local HEAD` and `observed PR head` refer to the full SHAs in
this table unless a command output shows the SHA verbatim.

At `2026-05-09T09:48:46Z`, `git rev-parse @{u}` returned the same SHA as the
observed GitHub PR head, `fb92a08c034f8f43dd8d1b7edc32d084d5596b3d`.
`git rev-list --left-right --count fb92a08c034f8f43dd8d1b7edc32d084d5596b3d...HEAD`
returned `0 1`, and `git merge-base --is-ancestor` confirmed the observed PR
head is an ancestor of the checked-out local HEAD.

Therefore, this artifact is scoped to PR #175 and the checked-out local HEAD,
but it does not claim that local HEAD `7232bed...` has been pushed to PR #175.

## Readiness evidence

### Local Git observations

Captured with:

```bash
date -u +%Y-%m-%dT%H:%M:%SZ
git branch --show-current
git rev-parse HEAD
git rev-parse --short HEAD
git rev-parse --abbrev-ref --symbolic-full-name @{u}
git log -1 --date=iso-strict --format='%H%x09%an%x09%ae%x09%ad%x09%s'
git status --short
```

Observed result at `2026-05-09T09:47:47Z`:

```text
branch=wave6-evidence-artifact-contract-1778302300
head_sha=7232beddb1ef9b3acf3bcd1fa8f87b0b951555ad
head_short=7232bed
upstream=origin/wave6-evidence-artifact-contract-1778302300
latest_commit=7232beddb1ef9b3acf3bcd1fa8f87b0b951555ad    Copilot    223556219+Copilot@users.noreply.github.com    2026-05-09T09:46:59Z    wip: checkpoint after implementation (steps 7-8)
status_short=<empty>
```

The clean `git status --short` observation applies only to the repository state
at that capture time, before this Step 9 artifact edit.

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
| `updatedAt` | `2026-05-09T08:36:48Z` |
| `headRefName` | `wave6-evidence-artifact-contract-1778302300` |
| `headRefOid` | `fb92a08c034f8f43dd8d1b7edc32d084d5596b3d` |
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
| Documentation Site | Build MkDocs site | `COMPLETED` | `SUCCESS` | `2026-05-09T08:37:05Z` |
| Quality Gates | detect changed files | `COMPLETED` | `SUCCESS` | `2026-05-09T08:36:59Z` |
| Documentation Site | Deploy to GitHub Pages | `COMPLETED` | `SKIPPED` | `2026-05-09T08:37:05Z` |
| Quality Gates | fmt, clippy, module size | `COMPLETED` | `SUCCESS` | `2026-05-09T08:37:33Z` |
| Quality Gates | tests | `COMPLETED` | `SUCCESS` | `2026-05-09T08:39:46Z` |
| Quality Gates | coverage | `COMPLETED` | `SUCCESS` | `2026-05-09T08:39:45Z` |
| Quality Gates | fmt, clippy, tests, module size, coverage | `COMPLETED` | `SUCCESS` | `2026-05-09T08:39:53Z` |
| Quality Gates | manual real Alice launch smoke | `COMPLETED` | `SKIPPED` | `2026-05-09T08:39:54Z` |
| none returned | GitGuardian Security Checks | `COMPLETED` | `SUCCESS` | `2026-05-09T08:36:50Z` |

These entries are per-check observations for the observed PR head. They are not
evidence that local HEAD has CI success.

### Local validation command evidence

| Command | Step 9b execution status | Bounded claim |
| --- | --- | --- |
| `NODE_OPTIONS=--max-old-space-size=32768 mkdocs build --strict` | Executed in Step 9b with exit status `0`; started `2026-05-09T09:51:45Z`, completed `2026-05-09T09:51:46Z`; MkDocs cleaned `site` and built documentation in `0.63` seconds. | Post-optimization local documentation rendering succeeded for this checkout. This is not CI success or merge readiness. |
| `cargo test -q -p eatme-assets default_workflow_attempt_contract_tests` | Executed in Step 9b with exit status `0`; started `2026-05-09T09:51:46Z`, completed `2026-05-09T09:51:46Z`; result: `6 passed; 0 failed; 0 ignored; 0 measured; 67 filtered out`. | Post-optimization focused contract tests passed for this checkout. This is not full-suite, coverage, CI, or merge-readiness evidence. |
| `cargo run -q -p eatme-cli -- assets validate --json` | Not executed in Step 9b. | No local asset-validation success is claimed. |
| `cargo run -q -p eatme-cli -- assets generate-gadugi --check --json` | Not executed in Step 9b. | No generated-adapter freshness success is claimed. |
| `TMPDIR=/tmp ./scripts/quality-gates.sh` | Not executed in Step 9b. | No local full quality-gate success is claimed. |
| `gh pr checks 175 --watch --interval 10` | Not executed in Step 9b. | No live check-watch completion or required-check sufficiency is claimed. |

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

The Step 9 review simplified the prior implementation-pass narrative into this
evidence contract and checked that it:

1. Scopes observations to PR #175 and local HEAD.
2. Explicitly records that the GitHub PR head observed by `gh pr view` is not
   local HEAD.
3. Keeps local Git evidence, GitHub PR metadata, status-check metadata, and local
   validation evidence in separate sections.
4. Lists checks that were not executed instead of implying success.
5. Provides explicit nonclaims so recovery can continue without prior
   rate-limited session context.

### Performance review

This artifact is static MkDocs content, so no algorithm, cache, or runtime
resource path applies. The Step 9b optimization was to keep evidence in one
command ledger, avoid a second unavailable-check table, and refer back to the
scoped SHA fields instead of repeating full identifiers outside command output.

### Security review

The Step 10b security review treated local Git output, GitHub PR metadata, and
status-check names as untrusted evidence. No security issue requiring source,
workflow, or credential-handling changes was found in this artifact.

| Checklist item | Result | Evidence or mitigation |
| --- | --- | --- |
| Input validation | Pass | SHA-like values in scope are recorded as observed 40-character lowercase hexadecimal values, except the explicitly labeled 7-character short SHA. Timestamps are recorded in UTC ISO format, and ambiguous PR fields are kept as metadata rather than readiness claims. |
| Output encoding | Pass | Command text is fenced as `bash` or `text`; observed dynamic values are in Markdown tables/code spans; no raw HTML or script content is introduced. |
| Authentication/authorization | Pass | The artifact records fixed read-only `git` observations and `gh pr view` metadata only. It does not merge, approve, push, alter workflows, or modify PR state. |
| Sensitive data handling | Pass | The page includes only bounded recovery evidence: repository, PR, branch, SHA, status-check, command, and nonclaim data. It does not include tokens, environment dumps, auth configuration, private config, or unrelated local file contents. |
| No hardcoded secrets | Pass | No password, token, API key, credential, or secret literal was observed. `NODE_OPTIONS=--max-old-space-size=32768` is a resource setting for the documented MkDocs command, not a credential. |
| Proper error messages | Pass | Unavailable or not-executed checks are recorded as nonclaims rather than success-shaped fallbacks, stack traces, credential-bearing errors, or hidden failures. |

## Nonclaims

- No merge readiness is claimed.
- No PR approval is claimed.
- No blanket CI success is claimed.
- No CI success is claimed for local HEAD.
- No claim is made that local HEAD has been pushed to PR #175.
- No full local test-suite success is claimed.
- No test coverage sufficiency is claimed.
- No local asset-validation success is claimed.
- No generated-adapter freshness success is claimed.
- No local full quality-gate success is claimed.
- No claim is made that skipped checks are acceptable for merge readiness.
- No claim is made that branch-protection requirements were satisfied.
- No claim is made that future PR #175 heads, checks, reviews, or mergeability
  match the observations recorded here.
- No prior rate-limited/default-workflow session context is required to continue
  recovery from this artifact.
