# Default-workflow PR readiness

Default-workflow PR readiness is the exact-head gate used when a pull request
needs a clear final readiness decision. It is also the recovery path when an
outer wrapper fails without a useful structured result.

This page describes the intended behavior for the PR readiness recovery feature:
incorporate current `master`, validate the candidate head locally, push the
candidate, verify the exact pull request head, keep readiness wording bounded,
validate docs/assets/generated adapters/tests, and return a structured result
that always explains whether files changed.

## Contents

- [Readiness contract](#readiness-contract)
- [Configuration](#configuration)
- [Usage](#usage)
- [Readiness wording policy](#readiness-wording-policy)
- [Validation gates](#validation-gates)
- [GitHub metadata fields](#github-metadata-fields)
- [Structured workflow output](#structured-workflow-output)
- [Examples](#examples)
- [Blocker handling](#blocker-handling)

## Readiness contract

A PR is default-workflow ready only when every gate passes for the exact commit
being reviewed.

| Gate | Required result |
| --- | --- |
| Branch | The existing recovery branch is used; no replacement branch is created. |
| Current `master` | The PR branch contains current `origin/master`, preferably by rebase. Merge is used only when rebase is unsafe or conflict-heavy. |
| Exact head | After push, the local `HEAD` SHA equals the PR `headRefOid`. A mismatch blocks readiness. |
| GitHub checks | Required checks are green for that same SHA. |
| Merge state | `mergeStateStatus` is `CLEAN`. |
| Mergeability | `mergeable` is `MERGEABLE`. |
| Readiness wording | User-facing wording is plain, bounded, and scoped to evidence packaging/readiness. |
| Overclaim boundary | The PR does not claim full UI automation, full world execution, visible rendering correctness, grading, creative assessment, Save completion, deployed sharing/platform success, or first-lesson completion. |
| Gadugi adapters | Generated adapters are fresh whenever canonical scenario assets are affected. |
| Scope | Fixes are limited to conflicts, failed checks, stale generated adapters, or wording that violates the readiness wording policy. |
| Structured output | The final workflow output includes `Files modified` with actual paths, or an explicit no-op justification with exact-head readiness evidence. |

A previous wrapper failure is not a blocker when direct verification proves the
same exact head, current master incorporation, green checks, clean mergeability,
bounded wording, fresh generated adapters, and passing local validation.

## Configuration

Run commands from the repository root.

Set the repository's large-heap Node option before invoking workflow wrappers or
documentation commands that may call Node-based tooling:

```bash
export NODE_OPTIONS=--max-old-space-size=32768
```

Keep local preference-file paths out of committed documentation. They are host
specific and are not part of the repository contract.

The Rust asset validation and Gadugi generator commands do not require Node, but
the environment variable is safe to keep exported for repository-wide workflow
commands.

For GitHub checks, use authenticated `gh` access to the repository that owns the
PR. Do not place tokens, secrets, local credential paths, environment dumps, or
raw command output in readiness comments.

## Usage

Use this generic procedure for evidence-artifact readiness recovery on any PR
that needs the same exact-head readiness gate. Replace the placeholders before
running the commands.

```bash
PR_NUMBER=<pr-number>
PR_BRANCH=<existing-pr-branch>
BASE_BRANCH=master
```

1. Start on the existing PR branch.

   ```bash
   git switch "$PR_BRANCH"
   ```

2. Fetch the current base branch and PR metadata.

   ```bash
   git fetch origin "$BASE_BRANCH" "$PR_BRANCH" --prune
   gh pr view "$PR_NUMBER" --json headRefName,headRefOid,mergeStateStatus,mergeable,statusCheckRollup
   ```

3. Incorporate current `master`.

   ```bash
   git rebase "origin/$BASE_BRANCH"
   ```

   If rebase is unsafe or conflict-heavy, stop the rebase and use the minimal
   merge path instead:

   ```bash
   git rebase --abort
   git merge "origin/$BASE_BRANCH"
   ```

4. Inspect readiness wording and generated adapter freshness when the PR
   touches scenario assets, generated adapters, readiness docs, or artifact
   contract code.

5. Run the validation gates in [Validation gates](#validation-gates) on the
   candidate local `HEAD` after all fixes.

6. Push the candidate head to the existing PR branch.

   ```bash
   git push origin "$PR_BRANCH"
   ```

7. Confirm exact-head readiness only after the push: local `HEAD` must equal the
   PR `headRefOid`, and required GitHub checks must be green for that SHA.

   ```bash
   git rev-parse HEAD
   gh pr view "$PR_NUMBER" --json headRefOid,mergeStateStatus,mergeable,statusCheckRollup
   gh pr checks "$PR_NUMBER" --watch --interval 10
   ```

8. Return the structured workflow output described in
   [Structured workflow output](#structured-workflow-output).

## Readiness wording policy

Evidence-artifact readiness is intentionally narrow. It may claim that the PR
packages, validates, or reports bounded evidence artifacts for review. It must
not turn available artifacts, screenshots, manifests, or declarations into
broader product-success claims.

Allowed wording:

```text
Evidence artifacts are packaged for readiness review.
The report records bounded evidence and explicit not-yet-shown claims.
Save completion requires distinct finish-state evidence.
Full world execution, deployed sharing, and platform success are not claimed.
```

Unsupported wording:

```text
The full Alice UI flow is automated.
The learner completed the first lesson.
The saved world was graded successfully.
Rendering correctness is proven.
Save and deployed sharing completed successfully.
```

This readiness wording policy applies to user-facing scenario text, readiness
output, PR comments, and recovery summaries. It is not the same as the artifact
text validator in [Evidence Artifact Contract](evidence-artifact-contract.md),
which validates supplied artifact fields and status-required evidence text
rather than arbitrary PR prose. Generated Gadugi adapters consume the canonical
scenario wording; do not hand-edit generated Gadugi YAML to change mission
intent.

For artifact field-level rules, see
[Evidence Artifact Contract](evidence-artifact-contract.md).

## Validation gates

Run the gates from the repository root after master incorporation and after any
fix. These local gates validate the candidate `HEAD`; exact-head readiness is
established only after that candidate is pushed, the PR `headRefOid` matches the
same SHA, and GitHub checks pass for that SHA.

Build the documentation site:

```bash
NODE_OPTIONS=--max-old-space-size=32768 mkdocs build --strict
```

Validate persona and scenario assets:

```bash
cargo run -q -p eatme-cli -- assets validate --json
```

Check generated Gadugi adapter freshness:

```bash
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
```

Run the repository quality gate. In deep worktrees, set `TMPDIR=/tmp` so Unix
socket paths stay short enough for the test environment:

```bash
TMPDIR=/tmp ./scripts/quality-gates.sh
```

The local gate is a candidate result until the branch is pushed. The PR is ready
only when each command exits successfully for the candidate head, the pushed PR
head matches that SHA, and required GitHub checks pass for that exact head.

## GitHub metadata fields

The readiness gate consumes these `gh pr view` fields:

| Field | Required value |
| --- | --- |
| `headRefName` | Existing PR branch name |
| `headRefOid` | Exact local `HEAD` SHA |
| `mergeStateStatus` | `CLEAN` |
| `mergeable` | `MERGEABLE` |
| `statusCheckRollup` | Required checks green for `headRefOid` |

Fetch the PR head, merge state, mergeability, and check summary:

```bash
gh pr view "$PR_NUMBER" \
  --json headRefName,headRefOid,mergeStateStatus,mergeable,statusCheckRollup
```

`statusCheckRollup` is green only when every required check for `headRefOid` has
completed successfully. A required check blocks readiness when it is pending,
queued, in progress, requested, failing, errored, timed out, skipped when branch
protection requires it to run, cancelled, missing, or reported for a different
head.

If the head changes during review, stop and restart readiness verification for
the new SHA.

## Structured workflow output

The final workflow result must make the file-change state explicit. This avoids
the no-op guard failure mode where validation passed but the output omitted both
modified files and no-op justification.

When files changed, include `Files modified` with real repository paths:

```text
Files modified:
- docs/default-workflow-pr-readiness.md
- assets/scenarios/eatme/student-artifact-package-share-evidence.yaml
- assets/scenarios/gadugi/student-artifact-package-share-evidence.yaml
```

When no files changed, include an explicit no-op justification and exact-head
readiness evidence:

```text
No-op justification:
No source changes were needed because the branch already incorporated current
master, the local HEAD matched the PR headRefOid after push, readiness wording
stayed within the bounded policy, generated adapters were fresh, and all
required validation gates passed at that exact head.

Exact-head readiness evidence:
- Branch: <existing-pr-branch>
- Local HEAD: <sha>
- PR headRefOid: <same-sha>
- mergeStateStatus: CLEAN
- mergeable: MERGEABLE
- Required checks: green for <same-sha>
- Validation: NODE_OPTIONS=--max-old-space-size=32768 mkdocs build --strict; assets validate --json; assets generate-gadugi --check --json; TMPDIR=/tmp ./scripts/quality-gates.sh
```

The readiness summary should name only gates that actually ran and passed at the
exact head. Do not include full logs, secrets, credential paths, or environment
dumps.

## Examples

### PR 175 command values

For PR 175 evidence-artifact readiness recovery, the generic placeholders are:

```bash
PR_NUMBER=175
PR_BRANCH=wave6-evidence-artifact-contract-1778302300
BASE_BRANCH=master
```

### PR 175 with a documentation-only recovery fix

Use this shape when the recovery changes only readiness documentation:

```text
Default-workflow readiness recovery for PR 175 completed at exact head <sha>.

Files modified:
- docs/default-workflow-pr-readiness.md

Verified gates:
- current master incorporated
- local HEAD equals PR headRefOid
- mergeStateStatus=CLEAN and mergeable=MERGEABLE
- readiness wording remains bounded
- generated Gadugi adapters are fresh
- docs, assets, adapter freshness, and repository quality gates pass
```

### PR 175 with no source changes

Use this shape when the branch is already ready after fetching current master:

```text
Default-workflow readiness recovery for PR 175 completed at exact head <sha>.

No-op justification:
No files were changed because the existing branch already satisfied the
readiness wording policy after current master verification.

Exact-head readiness evidence:
- Branch: wave6-evidence-artifact-contract-1778302300
- Local HEAD: <sha>
- PR headRefOid: <same-sha>
- Required checks: green for <same-sha>
- Local validation: docs, assets, generated adapters, and quality gates passed
```

## Blocker handling

If any gate fails, do not publish readiness. Fix only the minimal issue that
caused the blocker, run the relevant validation again, push the fix, and repeat
exact-head verification against the new PR head.

| Blocker | Minimal response |
| --- | --- |
| Head mismatch | Stop readiness for the old SHA and verify the requested new head. |
| Branch mismatch | Switch to the existing PR branch instead of creating a new branch. |
| Base branch drift | Rebase onto `origin/master`; use a merge only when rebase is unsafe or conflict-heavy. |
| Failing, pending, cancelled, missing, or wrong-head checks | Fix the failing check, wait for completion, or rerun the missing check before readiness. |
| Dirty merge state | Resolve only the mergeability issue. |
| Overclaiming scenario, artifact, or readiness language | Edit the canonical wording and regenerate adapters if affected. |
| Stale generated adapter | Regenerate adapters from canonical sources. |
| Asset validation failure | Fix the invalid scenario or persona asset. |
| Quality gate failure | Fix only the failing gate's root cause. |
| No files changed and no no-op evidence | Return an explicit no-op justification with exact-head readiness evidence. |
| Unrelated changes | Remove the unrelated change from the readiness work. |
