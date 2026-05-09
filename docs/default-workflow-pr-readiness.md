# Default-workflow PR readiness

Default-workflow PR readiness is the exact-head gate used when a pull request
needs a clear final readiness decision. It is also the recovery path when an
outer wrapper fails without a useful structured result.

This page describes the finished behavior for PR readiness recovery: incorporate
current `master`, verify the exact pull request head, keep evidence-artifact
claims bounded, validate docs/assets/generated adapters/tests, and return a
structured result that always explains whether files changed.

## Contents

- [Readiness contract](#readiness-contract)
- [Configuration](#configuration)
- [Usage](#usage)
- [Evidence-artifact boundary](#evidence-artifact-boundary)
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
| Exact head | The local `HEAD` SHA equals the PR `headRefOid`. A mismatch blocks readiness. |
| GitHub checks | Required checks are green for that same SHA. |
| Merge state | `mergeStateStatus` is `CLEAN`. |
| Mergeability | `mergeable` is `MERGEABLE`. |
| Evidence wording | User-facing wording is plain, bounded, and scoped to evidence packaging/readiness. |
| Overclaim boundary | The PR does not claim full UI automation, full world execution, visible rendering correctness, grading, creative assessment, Save completion, deployed sharing/platform success, or first-lesson completion. |
| Gadugi adapters | Generated adapters are fresh whenever canonical scenario assets are affected. |
| Scope | Fixes are limited to conflicts, failed checks, stale generated adapters, or wording that violates the evidence-artifact contract. |
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

Use this procedure for PR 175 evidence-artifact readiness recovery and for later
PRs that need the same exact-head readiness gate.

1. Start on the existing PR branch.

   ```bash
   git switch wave6-evidence-artifact-contract-1778302300
   ```

2. Fetch the current base branch and PR metadata.

   ```bash
   git fetch origin master wave6-evidence-artifact-contract-1778302300 --prune
   gh pr view 175 --json headRefName,headRefOid,mergeStateStatus,mergeable,statusCheckRollup
   ```

3. Incorporate current `master`.

   ```bash
   git rebase origin/master
   ```

   If rebase is unsafe or conflict-heavy, stop the rebase and use the minimal
   merge path instead:

   ```bash
   git rebase --abort
   git merge origin/master
   ```

4. Confirm the local head equals the PR head.

   ```bash
   git rev-parse HEAD
   gh pr view 175 --json headRefOid
   ```

5. Inspect evidence-artifact wording and generated adapter freshness when the PR
   touches scenario assets, generated adapters, readiness docs, or artifact
   contract code.

6. Run the validation gates in [Validation gates](#validation-gates).

7. Return the structured workflow output described in
   [Structured workflow output](#structured-workflow-output).

## Evidence-artifact boundary

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

The contract applies to user-facing scenario text, readiness output, PR comments,
and recovery summaries. Generated Gadugi adapters consume the canonical scenario
contract; do not hand-edit generated Gadugi YAML to change mission intent.

For artifact field-level rules, see
[Evidence Artifact Contract](evidence-artifact-contract.md).

## Validation gates

Run the gates from the repository root after master incorporation and after any
fix.

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

The gate is ready only when each command exits successfully for the exact head
that matches the PR head.

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
gh pr view 175 \
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
master, the local HEAD matched PR 175 headRefOid, evidence-artifact wording
stayed within the bounded contract, generated adapters were fresh, and all
required validation gates passed at that exact head.

Exact-head readiness evidence:
- Branch: wave6-evidence-artifact-contract-1778302300
- Local HEAD: <sha>
- PR headRefOid: <same-sha>
- mergeStateStatus: CLEAN
- mergeable: MERGEABLE
- Required checks: green for <same-sha>
- Validation: mkdocs build --strict; assets validate --json; assets generate-gadugi --check --json; TMPDIR=/tmp ./scripts/quality-gates.sh
```

The readiness summary should name only gates that actually ran and passed at the
exact head. Do not include full logs, secrets, credential paths, or environment
dumps.

## Examples

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
- evidence-artifact wording remains bounded
- generated Gadugi adapters are fresh
- docs, assets, adapter freshness, and repository quality gates pass
```

### PR 175 with no source changes

Use this shape when the branch is already ready after fetching current master:

```text
Default-workflow readiness recovery for PR 175 completed at exact head <sha>.

No-op justification:
No files were changed because the existing branch already satisfied the
evidence-artifact contract after current master verification.

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
