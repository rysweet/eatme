# Default-workflow PR readiness

## PR #175 evidence contract

This page is the self-contained recovery artifact for PR #175. It records
bounded evidence for a validated PR evidence head, GitHub PR metadata observed
at evidence-capture time, and the publication-head boundary for this artifact.

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
| Validated evidence head | `a951f34a0a187adfa24cfe0555ca00da6a04197d` |
| Validated evidence head short SHA | `a951f34` |
| Observed GitHub PR head at evidence capture | `a951f34a0a187adfa24cfe0555ca00da6a04197d` |
| Observed base branch | `master` |
| Observed base SHA | `17521c40bb72dd22669b596179327fc5cf307305` |
| Evidence-head executable evidence capture | `2026-05-09T19:02:06Z` |
| GitHub PR metadata capture | `2026-05-09T19:02:06Z` |
| Artifact publication head | not embedded in this committed artifact; committing a documentation refinement changes the PR head. The exact post-push publication head/check rollup belongs in the PR finalization record outside this file. |

Within this page, `validated evidence head` means the PR head whose metadata and
check rollup were captured above. `Artifact publication head` means the later
commit that publishes this page after refinement.

At evidence-capture time, the checked-out local HEAD and GitHub PR `headRefOid`
both resolved to `a951f34a0a187adfa24cfe0555ca00da6a04197d`. Therefore, the
GitHub check rollup, mergeability metadata, and review metadata below are
validated evidence-head observations for the same commit. This page deliberately
does not claim that its own eventual publication commit has checked itself.

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

Observed result at `2026-05-09T19:02:06Z`, before this refinement changed the
artifact/test files:

```text
branch=wave6-evidence-artifact-contract-1778302300
head_sha=a951f34a0a187adfa24cfe0555ca00da6a04197d
head_short=a951f34
upstream=origin/wave6-evidence-artifact-contract-1778302300
upstream_sha=a951f34a0a187adfa24cfe0555ca00da6a04197d
status_short_begin
status_short_end
```

This clean status is a pre-refinement observation for the validated evidence
head. It is not a claim about the post-edit worktree or the eventual
publication head. This refinement intentionally changes only the readiness
artifact and the contract tests that guard it:

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
| `updatedAt` | `2026-05-09T18:55:26Z` |
| `headRefName` | `wave6-evidence-artifact-contract-1778302300` |
| `headRefOid` | `a951f34a0a187adfa24cfe0555ca00da6a04197d` |
| `baseRefName` | `master` |
| `baseRefOid` | `17521c40bb72dd22669b596179327fc5cf307305` |
| `mergeStateStatus` | `CLEAN` |
| `mergeable` | `MERGEABLE` |
| `reviewDecision` | Empty value returned; owner-free finalization does not require approval evidence. |
| `latestReviews` | Empty list returned; no human approval is claimed. |

The `mergeStateStatus` and `mergeable` values are recorded as GitHub metadata
only. They are treated as merge-readiness evidence only in combination with the
validated evidence-head green check rollup below, the publication-head boundary,
and the focused evidence-artifact scope.

### GitHub status-check rollup observation

`gh pr view` returned these `statusCheckRollup` entries for the validated
evidence head:

| Workflow | Check | Status | Conclusion | Completed |
| --- | --- | --- | --- | --- |
| Documentation Site | Build MkDocs site | `COMPLETED` | `SUCCESS` | `2026-05-09T18:55:44Z` |
| Quality Gates | detect changed files | `COMPLETED` | `SUCCESS` | `2026-05-09T18:55:37Z` |
| Documentation Site | Deploy to GitHub Pages | `COMPLETED` | `SKIPPED` | `2026-05-09T18:55:45Z` |
| Quality Gates | fmt, clippy, module size | `COMPLETED` | `SUCCESS` | `2026-05-09T18:56:19Z` |
| Quality Gates | tests | `COMPLETED` | `SUCCESS` | `2026-05-09T18:58:29Z` |
| Quality Gates | coverage | `COMPLETED` | `SUCCESS` | `2026-05-09T18:58:30Z` |
| Quality Gates | fmt, clippy, tests, module size, coverage | `COMPLETED` | `SUCCESS` | `2026-05-09T18:58:38Z` |
| Quality Gates | manual real Alice launch smoke | `COMPLETED` | `SKIPPED` | `2026-05-09T18:58:38Z` |
| none returned | GitGuardian Security Checks | `COMPLETED` | `SUCCESS` | `2026-05-09T18:55:30Z` |

These entries are per-check observations for the validated evidence head: 7
successful checks, 2 skipped checks, 0 failing checks, and 0 pending checks.
Skipped rows are explicitly not counted as successful checks, approval,
branch-protection sufficiency, or manual real Alice launch evidence.

### Validated evidence-head executable evidence

Validated evidence-head executable evidence uses the GitHub status-check rollup
for PR head `a951f34a0a187adfa24cfe0555ca00da6a04197d` as the source of truth.
The rollup is complete for that head, contains no failing or pending checks, and
is sufficient for the evidence baseline before this refinement. Backup local
validation commands remain documented with the required
`NODE_OPTIONS=--max-old-space-size=32768` setting and no timeout wrapper, but
they are not rerun unless GitHub evidence is stale, missing, ambiguous, or local
files change.

| Backup command | Evidence source | Bounded claim |
| --- | --- | --- |
| `cargo run -q -p eatme-cli -- assets validate --json` | Covered by the validated evidence-head Quality Gates `tests` and aggregate successful rollup; rerun locally only if asset evidence becomes ambiguous. | Persona and scenario asset validation is within validated evidence-head scope. This is asset-contract evidence, not lesson-completion or grading evidence. |
| `cargo run -q -p eatme-cli -- assets generate-gadugi --check --json` | Covered by the validated evidence-head Quality Gates `tests` and aggregate successful rollup; rerun locally only if generated-adapter evidence becomes ambiguous. | Generated Gadugi adapter freshness is within validated evidence-head scope. This is adapter freshness evidence, not UI rendering or grading evidence. |
| `mkdocs build --strict` | Validated evidence-head Documentation Site `Build MkDocs site` completed with `SUCCESS`. | The documentation site renders under strict MkDocs rules for the validated evidence head. |
| `TMPDIR=/tmp ./scripts/quality-gates.sh` | Validated evidence-head Quality Gates aggregate `fmt, clippy, tests, module size, coverage` completed with `SUCCESS`. | The repository quality gate passes for the validated evidence head. This does not prove manual real Alice desktop launch, full UI automation, visual rendering correctness, grading, creative assessment, or lesson completion. |

Because this file and its contract tests are part of the refinement, the exact
post-push publication head must be checked after push and recorded outside this
committed artifact. This avoids a self-referential freshness claim where editing
the artifact invalidates the SHA it names as current.

### Historical same-head outside-in testing evidence

The Step 16b user-path commands below were previously run from this branch at
the recorded head `5b1c9f18b474ee61e64f2298c9e0b6d0af4ad301`. That recorded
head was same-head evidence for an earlier PR capture. It is now historical
silver-thread/e2e context only, not validated evidence-head proof and not a
substitute for the GitHub check rollup above.

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

1. Separately scopes validated evidence-head executable evidence and GitHub PR
   metadata.
2. Keeps local Git evidence, GitHub PR metadata, status-check metadata, and
   executable evidence in separate sections.
3. Lists skipped, not-measured, no-execute, and historical states as nonclaims
   instead of implying success.
4. Records the no-execute readiness probe as historical fail-closed behavior,
   not as a product readiness success.
5. Provides explicit nonclaims so recovery can continue without prior
   rate-limited session context.
6. Separates the validated evidence head from the later artifact publication
   head so the page cannot become stale solely by being committed.

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

PR #175 remains unmerged. At evidence-capture time, the observed GitHub PR state
was `OPEN`, the observed head ref was
`wave6-evidence-artifact-contract-1778302300`, and the validated evidence head
was `a951f34a0a187adfa24cfe0555ca00da6a04197d`. That SHA is the evidence head
for this page, not the immutable publication head for this committed
refinement.

No manual merge was performed. This recovery only updates workflow
readiness/review/finalization evidence and the executable readiness-contract
tests that guard it.

Finalization status: `merge-ready-after-publication-head-checks` for PR #175
evidence-contract recovery. The validated evidence head is `CLEAN`,
`MERGEABLE`, and has 7 successful checks, 2 skipped checks, 0 failing checks,
and 0 pending checks. Because this refinement changes the committed artifact,
final owner-free merge evidence must use the post-push publication head/check
rollup recorded outside this file. This artifact records executable evidence,
review boundaries, and explicit nonclaims; it does not mean the PR is already
approved, merged, or validated for UI automation, rendering correctness,
grading, creative assessment, or lesson completion.

### External publication-head evidence record

The exact publication-head evidence must be recorded outside this committed
artifact after push, because the act of committing this file creates a new PR
head that this file cannot already have observed. The external record must name
the publication head SHA and the GitHub check rollup for that exact SHA before
calling the PR merge-ready or giving a literal no-op justification tied to the
publication head, check rollup, and focused artifact-contract scope.

Required external record fields:

| Field | Required evidence |
| --- | --- |
| Publication head SHA | Full 40-character PR `headRefOid` observed from GitHub after push. |
| GitHub check rollup for that exact SHA | Successful, skipped, failing, and pending check counts for the publication head. |
| Merge state | GitHub `mergeStateStatus` and `mergeable` values for the publication head. |
| Review state | Current `reviewDecision` and latest review observations, including any empty owner-free state. |
| Owner-free decision | Explicit statement that owner-free finalization does not require owner intervention. |
| Scope decision | Confirmation that finalization remains limited to the focused artifact-contract scope. |
| Validation decision | Whether GitHub current-head evidence is sufficient or which focused local checks were rerun. |
| Finalization decision | Merge-ready conclusion or literal no-op justification tied to the publication head, checks, and scope. |
| PR evidence comment | URL or identifier for the external publication-head evidence record. |

## Nonclaims

- No PR approval is claimed.
- No blanket CI success is claimed beyond the listed validated evidence-head
  GitHub status-check rollup.
- No test coverage sufficiency is claimed beyond the reported validated
  evidence-head coverage summary.
- No local quality-gate rerun is claimed beyond the validated evidence-head
  GitHub Quality Gates rollup.
- No post-push publication-head check rollup is claimed inside this committed
  artifact.
- No real Alice desktop execution is claimed.
- No full Alice UI automation is claimed.
- No full first-lesson readiness is claimed.
- No first-lesson completion is claimed.
- No Save completion is claimed.
- No visible rendering correctness is claimed.
- No grading or creative assessment is claimed.
- No claim is made that skipped checks are successful checks.
- No claim is made inside this file that GitHub has observed the eventual
  publication commit beyond the recorded validated evidence-head `headRefOid`.
- No claim is made that future PR #175 heads, checks, reviews, or mergeability
  match the observations recorded here.
- No prior rate-limited/default-workflow session context is required to continue
  recovery from this artifact.
