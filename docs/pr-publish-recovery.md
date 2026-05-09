# PR publish-failure recovery

This guide defines the recovery workflow for a pull request whose publish step
completed validation work but failed while committing or pushing. The workflow
keeps recovery bounded to the current PR head, preserved publish artifacts, and
repository-owned validation gates.

Use this workflow for PR 174 when the failed publish attempt stopped because
`pre-commit` was installed locally but the repository did not contain
`.pre-commit-config.yaml`.

## Recovery contract

The recovery workflow has three valid outcomes:

| Outcome | When to use it |
| --- | --- |
| Focused recovery commit | Preserved artifacts prove that an allowed version or package metadata change is missing from the PR branch. |
| No-op justification | The required PR head is verified, artifacts are accounted for, no allowed metadata change is missing, and the worktree is clean. Any merge-readiness blockers are still reported separately. |
| Blocked report | Required artifacts are missing, the PR head cannot be verified against the captured live `headRefOid`, recovery intent is ambiguous, validation fails, or a focused recovery commit cannot be made safely. |

Do not manually merge the PR. Do not force-push, rebase, reset, or rewrite
history. Do not add `.pre-commit-config.yaml` for this recovery path.

## Required inputs

| Input | Requirement |
| --- | --- |
| PR number | `174` |
| Required recovery head | Capture PR 174's live `headRefOid` with `gh pr view` at recovery start; do not hardcode a commit SHA in this repository-owned contract. |
| PR branch | `wave6-persona-gap-fill-1778302300` |
| Preserved artifacts | The PR 174 `wave7-pr174-*` path recorded in [Local Hook Artifacts](local-hook-artifacts.md) by default |
| Allowed recovery files | `pyproject.toml`, `mkdocs.yml`, and package-facing metadata documentation only when artifacts prove the intent |
| Repository gates | Cargo asset validation, Gadugi adapter freshness, MkDocs strict build when docs or metadata affect documentation |

Treat preserved artifacts as evidence only. Inspect them as data; do not execute
artifact contents.

## Configuration

Keep the Node heap preference available for tooling that needs it:

```bash
export NODE_OPTIONS=--max-old-space-size=32768
```

This environment variable is a runner preference, not a project requirement.
Do not add it to repository configuration.

Use the PR 174 failed-publish artifact directory recorded in
[Local Hook Artifacts](local-hook-artifacts.md) (`docs/local-hook-artifacts.md`)
by default:

```bash
export PR_RECOVERY_ARTIFACT_DIR="<PR-174-failed-publish-files-directory>"
```

When committing is required and `pre-commit` blocks the commit only because the
repository has no `.pre-commit-config.yaml`, use the full commit command in
[step 8](#8-commit-and-push-only-when-recovery-changed-files) with the
`PRE_COMMIT_ALLOW_NO_CONFIG=1` prefix. Do not omit the co-author trailer.

Use `PRE_COMMIT_ALLOW_NO_CONFIG=1` only after all of these are true:

1. `pre-commit` is installed and the commit hook fails only because there is no
   repository pre-commit config.
2. `test ! -f .pre-commit-config.yaml` succeeds from the repository root.
3. Existing repository validation gates pass for the touched files.
4. The staged files are limited to artifact-proven recovery metadata.

Set `PR_RECOVERY_ARTIFACT_DIR` to another preserved `files` directory only when
recovering a different failed-publish session with equivalent `wave7-pr174-*`
artifacts.

## Recovery components

The recovery design is split into evidence, GitHub service access,
reconciliation, validation, commit, and reporting components. Each component
must either produce recorded evidence for the same PR head or stop with a
blocked report.

## Evidence Collector

Gather local git state, fetch PR #174, verify captured live `headRefOid`,
inspect the preserved artifacts at
the PR 174 path recorded in [Local Hook Artifacts](local-hook-artifacts.md),
confirm repo validation surfaces, and detect pre-commit config presence. The
repository-root config probe is:

```bash
test ! -f .pre-commit-config.yaml
```

If artifact access denied prevents listing or reading required evidence, report
that the required artifacts are inaccessible and stop.

## PR State Inspector

Record open/draft status, base branch, head SHA, mergeability, review decision,
status checks, changed files, and commits for the same PR head. The GitHub query
must include `headRefOid` and `mergeStateStatus`. Green checks are evidence, not
sufficient merge-ready proof.

## GitHub Service Adapter

Use the authenticated `gh` CLI as the only GitHub API client for this recovery
workflow. Confirm access with `gh auth status --hostname github.com` without
printing tokens, then use `gh pr view 174` for read-only PR metadata. Do not add
a second API client, create tokens, or persist credentials.

Classify external service failures explicitly:

| Failure | Recovery handling |
| --- | --- |
| Authentication failure | Stop with a blocked report that records the exact `gh` failure. |
| API errors | Stop with a blocked report unless the read-only metadata request is clearly transient. |
| Rate limiting | Stop or retry a read-only metadata request only after the server or CLI guidance says it is safe. |
| Network interruption | Retry only read-only PR metadata once, then block if the same call still fails. |

Retry logic is intentionally narrow: retry read-only GitHub metadata reads at
most once, never retry `git push` blindly, and never turn a failed publish into a
local fallback commit. If a publish or PR-comment operation fails, preserve the
intended PR text unchanged outside the repository, record the exact `gh` failure
beside it, and report the external service blocker.

## Artifact Reconciler

Compare preserved artifact intent against current PR contents and classify each
signal as focused metadata, already present, out of scope, ambiguous, or a
blocked evidence gap. Do not execute artifact contents. Allowed focused metadata
scope is limited to `pyproject.toml`, `mkdocs.yml`, and package-facing metadata
documentation when artifacts prove the intent.

## Validator

Run existing repository-appropriate Cargo/MkDocs/asset validation for the files
touched by recovery. Preserve `NODE_OPTIONS=--max-old-space-size=32768`, run
`cargo run -q -p eatme-cli -- assets validate --json`, run
`cargo run -q -p eatme-cli -- assets generate-gadugi --check --json`, and run
`mkdocs build --strict` when docs or package metadata are touched. Do not use
timeout wrappers.

## Commit/Push Handler

Only if required, commit and push artifact-proven focused metadata changes to
`wave6-persona-gap-fill-1778302300`. Do not add .pre-commit-config.yaml.
`PRE_COMMIT_ALLOW_NO_CONFIG=1` is allowed only when pre-commit is installed, no
repo pre-commit config exists, repository gates passed, and the only staged
files are artifact-proven focused metadata. Commit messages must keep the
co-author trailer:

```text
Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>
```

## Readiness Reporter

Emit a strict no-op, focused recovery, or blocked report for the exact final
head. A no-op report must include `No-op justification:`, the captured live
`headRefOid`, the artifact accounting, clean-scope evidence, PR state, check
state, and `Merge-ready blockers/evidence:`. Green checks alone were not treated
as merge-ready. Manual merge: not performed.

## Recovery workflow

Run commands from the repository root.

### 1. Verify the local scope

```bash
git status --short
git remote
git branch --show-current
gh auth status --hostname github.com
```

The worktree must be clean before a no-op result. If local changes already
exist, classify them before continuing and stage only recovery-approved files.
Do not print remote URLs or credentials in recovery notes.

### 2. Fetch and verify the required PR head

```bash
required_head="$(gh pr view 174 --json headRefOid --jq .headRefOid)"
git fetch origin pull/174/head:pr-174-publish-recovery
git switch pr-174-publish-recovery
git rev-parse HEAD
test "$(git rev-parse HEAD)" = "${required_head}"
```

The recovered local head must equal the captured live `headRefOid`. If the PR
head changes after capture, refresh the exact-head evidence before applying
recovery changes or stop with a blocked report.

### 3. Capture live PR state

```bash
gh pr view 174 \
  --json state,isDraft,baseRefName,headRefName,headRefOid,mergeStateStatus,mergeable,reviewDecision,statusCheckRollup,files,commits
```

Record the returned `headRefOid`, merge state, mergeability, review decision,
status checks, changed files, and commits. Green checks are evidence, not
sufficient merge-ready proof.

If `gh pr view` fails because of authentication, API errors, rate limiting, or
network interruption, follow the [GitHub Service Adapter](#github-service-adapter)
rules. Do not infer live PR state from local git state when the GitHub metadata
read fails.

### 4. Account for preserved artifacts

```bash
ls -1 "${PR_RECOVERY_ARTIFACT_DIR}"/wave7-pr174-*
```

For each artifact, record:

| Field | Meaning |
| --- | --- |
| Path | The preserved file inspected. |
| Artifact type | Failed log, status file, patch, intended PR text, or other preserved publish evidence. |
| Recovery signal | Missing focused metadata, already-present metadata, out-of-scope content, ambiguity, or failure evidence. |
| Action | Apply focused edit, no-op, or block. |

If required artifacts are inaccessible, the workflow is blocked. Artifact access
denied is a blocked evidence gap, not a reason to infer intent. Do not replace
the artifact accounting with assumptions from local files or live PR metadata.

### 5. Reconcile artifact intent with the PR head

Compare artifact-proven metadata intent against the checked-out PR head.

Allowed focused recovery changes are limited to:

```text
pyproject.toml
mkdocs.yml
docs/<artifact-proven-package-facing-metadata-doc>.md
```

Documentation changes are allowed only when the artifact explicitly proves
package-facing metadata documentation intent. Asset, scenario, generated adapter,
test, workflow, or broad formatting changes are outside this recovery scope and
out of scope for recovery commits.

Choose the no-op path when all artifact-proven intended metadata is already
present at the required head.

### 6. Apply a focused recovery change when required

Edit only the artifact-proven file. Stage by path:

```bash
git add pyproject.toml
git add mkdocs.yml
git add docs/<metadata-doc>.md
git diff --cached --name-only
```

The staged file list must contain only the approved recovery files.

### 7. Validate through repository gates

Run the gates that match the touched files:

```bash
cargo run -q -p eatme-cli -- assets validate --json
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
mkdocs build --strict
```

For a documentation-only recovery, `mkdocs build --strict` is required. Keep the
Cargo asset gates when the documentation references assets, scenarios, adapters,
or the PR readiness contract.

Do not use shell-level timeout wrapper commands. Do not use timeout wrappers.

### 8. Commit and push only when recovery changed files

Commit with the repository co-author trailer:

```bash
git commit -m "Finalize PR 174 publish metadata

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
git push origin HEAD:wave6-persona-gap-fill-1778302300
```

If `pre-commit` blocks only because there is no `.pre-commit-config.yaml`, repeat
the same commit command with `PRE_COMMIT_ALLOW_NO_CONFIG=1` after confirming the
conditions in [Configuration](#configuration):

```bash
PRE_COMMIT_ALLOW_NO_CONFIG=1 git commit -m "Finalize PR 174 publish metadata

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

### 9. Refresh PR state after any push

```bash
gh pr view 174 \
  --json state,isDraft,baseRefName,headRefName,headRefOid,mergeStateStatus,mergeable,reviewDecision,statusCheckRollup,files,commits
git status --short
git rev-parse HEAD
```

The final report must refer to the post-push `headRefOid`. If no push occurred,
it must refer to the verified no-op baseline.

## No-op report template

Use this literal structure only when no repository changes are required:

```text
No-op justification:
PR 174 remains at captured live head <headRefOid>.
Preserved wave7-pr174 artifacts were inspected from <artifact-directory> and
accounted for as: <artifact-summary>.
No artifact-proven focused metadata change is missing from pyproject.toml,
mkdocs.yml, or package-facing metadata documentation.
Worktree: clean.
PR state: <open-or-draft-state>; base <base>; head branch
wave6-persona-gap-fill-1778302300; headRefOid
<headRefOid>; merge state <state>;
mergeable <value>; review decision <decision>.
Checks: <same-head check summary>.
Merge-ready blockers/evidence: <review state, mergeability, clean scope, pending
or missing evidence>. Green checks alone were not treated as merge-ready.
```

The no-op recovery decision is invalid when artifacts are missing, the head
differs from the captured live `headRefOid`, the worktree is dirty, or allowed
metadata intent is ambiguous. Unresolved merge-readiness blockers do not
invalidate a no-op recovery decision, but they must be reported in the
merge-ready blockers/evidence line and must not be described as merge-ready.

Blocked edge cases include artifact access denied, required artifacts are
inaccessible, PR head changed after capture, artifact intent is
ambiguous, out-of-scope changes, dirty worktree state, pending checks, failed
checks, GitHub authentication failure, GitHub API errors, GitHub rate limiting,
network interruption after the single allowed read-only retry, and green checks
alone are not merge-ready.

## Focused recovery report template

Use this structure after a recovery commit is pushed:

```text
Focused recovery completed for PR 174.
Starting head: <captured live headRefOid>.
Final head: <post-push headRefOid>.
Preserved wave7-pr174 artifacts inspected: <artifact-summary>.
Recovered files: <staged-file-list>.
Validation: <commands run and result>.
Pre-commit handling: <not needed | PRE_COMMIT_ALLOW_NO_CONFIG=1 used because no
.pre-commit-config.yaml exists and repository gates passed>.
PR state: <state, draft status, merge state, mergeability, review decision,
same-head checks>.
Manual merge: not performed.
```

## Blocked report template

Use this structure when recovery cannot proceed safely:

```text
Blocked:
<specific blocker>.

Evidence:
- Captured live headRefOid: <headRefOid>.
- Observed PR head: <headRefOid>.
- Artifact accounting: <available, missing, inaccessible, or ambiguous files>.
- Worktree scope: <clean or dirty with paths>.
- Validation state: <commands run or skipped because blocked>.

No recovery commit was made and PR 174 was not manually merged.
```

## Readiness rules

PR 174 is not merge-ready merely because checks are green. A strict final
readiness statement must include:

| Evidence | Required state |
| --- | --- |
| Exact head | Local `HEAD` and PR `headRefOid` match the reported final head. |
| Scope | Changed files are empty for no-op or limited to artifact-proven metadata for recovery. |
| Artifacts | Every preserved `wave7-pr174-*` artifact is inspected or reported as a blocker. |
| Validation | Repository gates relevant to the touched files pass without timeout wrappers. |
| Merge state | GitHub reports a clean merge state and mergeability for the same head. |
| Review state | `reviewDecision`, reviews, and comments are recorded for the same head. |
| Manual merge | No manual merge or equivalent merge action was performed. |

## Related documentation

- [Default-workflow PR Readiness](default-workflow-pr-readiness.md)
- [Validation and Quality Gates](validation-quality-gates.md)
- [GitHub Pages](github-pages.md)
