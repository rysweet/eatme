# Default-workflow PR readiness

Default-workflow PR readiness is the exact-head gate used when a pull request
needs a final readiness decision without a manual merge.

For scenario-link recovery work, the gate establishes only that the current
branch head is reviewable, generated assets are reproducible, documentation
builds, and the first-lesson evidence path stays bounded. It does not establish
full UI automation, rendering correctness, grading, creative assessment, Save
completion, lesson completion, or broad Alice compatibility.

## Contents

- [Readiness contract](#readiness-contract)
- [Real branch workspace](#real-branch-workspace)
- [Generic readiness procedure](#generic-readiness-procedure)
- [Configuration](#configuration)
- [GitHub metadata fields](#github-metadata-fields)
- [Scenario-link evidence boundary](#scenario-link-evidence-boundary)
- [Generated Gadugi adapter freshness](#generated-gadugi-adapter-freshness)
- [Scenario-link recovery procedure](#scenario-link-recovery-procedure)
- [Draft and owner-free review handling](#draft-and-owner-free-review-handling)
- [Review and finalization packet](#review-and-finalization-packet)
- [Readiness note](#readiness-note)
- [Blocker handling](#blocker-handling)

## Readiness contract

A PR is default-workflow ready only when every gate passes for the exact commit
being reviewed.

| Gate | Required result |
| --- | --- |
| Exact head | The PR head SHA equals the requested SHA. A mismatch blocks readiness. |
| Real branch | The local workspace is on the PR's real `headRefName`, not a detached checkout. |
| Draft state | Draft PRs are `NOT_MERGE_READY` unless the workflow intentionally marks the PR ready for review. |
| GitHub checks | Required checks are green for that same SHA. |
| Merge state | `mergeStateStatus` is `CLEAN`. |
| Mergeability | `mergeable` is `MERGEABLE`. |
| Review state | `reviewDecision` is captured for the exact head. Missing or owner-free review is recorded instead of guessed. |
| Scenario-link wording | Canonical scenarios use plain, bounded, user-facing prerequisite, evidence, and follow-on language. |
| Overclaim boundary | Scenario, generated runner, and docs wording do not claim first-lesson completion, grading, creative assessment, full UI automation, visible rendering correctness, full Save completion, or complete Alice coverage. |
| Gadugi adapters | Generated adapters are fresh whenever canonical scenario assets or generator output are affected. |
| Documentation | MkDocs builds in strict mode when documentation is changed. |
| Quality gate | The repository quality gate passes for the current head before finalization. |
| Scope | No unrelated files or behavior are changed. |

A previous wrapper failure is not a blocker when direct verification confirms the
same head, green checks, clean mergeability, bounded wording, and fresh
generated adapters. The workflow records readiness; it does not merge the pull
request.

## Real branch workspace

Recover PRs on the real pull request branch. A detached `pull/<number>/head`
checkout is valid for inspection, but it is not valid for workflow-owned
finalization because it cannot safely receive focused fixes.

Fetch the PR, check out the advertised branch, and compare local `HEAD` with
GitHub's `headRefOid` before editing or pushing:

```bash
PR_NUMBER="${PR_NUMBER:?set PR_NUMBER to the pull request number}"
HEAD_REF="$(gh pr view "$PR_NUMBER" --json headRefName --jq .headRefName)"
git fetch origin "$HEAD_REF"
git switch "$HEAD_REF"

LOCAL_HEAD="$(git rev-parse HEAD)"
REMOTE_HEAD="$(gh pr view "$PR_NUMBER" --json headRefOid --jq .headRefOid)"
test "$LOCAL_HEAD" = "$REMOTE_HEAD"
```

If the comparison fails, fetch again and restart finalization for the new head.
Do not push from a workspace whose `HEAD` does not equal the current
`headRefOid`.

## Generic readiness procedure

Run the gate in this order:

1. Verify the PR head equals the exact requested SHA.
2. Verify the workspace is on the PR's real branch.
3. Capture draft status, review decision, mergeability, merge state, checks, and
   changed-file scope for that same SHA.
4. Verify GitHub checks are green for that same SHA.
5. Verify `mergeStateStatus=CLEAN` and `mergeable=MERGEABLE`.
6. Treat `isDraft=true` as `NOT_MERGE_READY` unless the workflow intentionally
   marks the PR ready for review.
7. Inspect scenario-link wording if the PR touches canonical scenarios,
   generated adapters, or docs that describe the first-lesson evidence path.
8. Run the generated Gadugi adapter freshness check if any canonical scenario
   asset or generator output is affected.
9. Validate assets.
10. Build docs in strict mode when docs are changed.
11. Run the repository quality gate.
12. Prepare the finalization packet with `MERGE_READY` or `NOT_MERGE_READY`.

## Configuration

Run commands from the repository root.

If running Node-based workflow wrappers, set the repository's large-heap Node
option before invoking the wrapper:

```bash
export NODE_OPTIONS=--max-old-space-size=32768
```

The Rust asset validation and Gadugi generator commands do not require Node, but
the environment variable is safe to keep exported for repository-wide workflow
commands.

Deep worktrees can exceed Unix socket path limits during the full repository
quality gate. Run that gate with a short temporary directory:

```bash
TMPDIR=/tmp ./scripts/quality-gates.sh
```

For GitHub checks, use authenticated `gh` access to the repository that owns the
PR. Do not place tokens, secrets, local credential paths, environment dumps, or
raw command output in readiness notes.

## GitHub metadata fields

The readiness gate consumes these `gh pr view` fields:

| Field | Required value |
| --- | --- |
| `headRefName` | Branch checked out locally |
| `headRefOid` | Exact requested SHA |
| `isDraft` | `false` for `MERGE_READY`; `true` means `NOT_MERGE_READY` unless the workflow marks ready |
| `mergeStateStatus` | `CLEAN` |
| `mergeable` | `MERGEABLE` |
| `reviewDecision` | Captured exactly; empty or missing review is recorded as owner-free review state |
| `changedFiles` | Count captured with the file list used for scope validation |
| `statusCheckRollup` | Required checks green for `headRefOid` |

Fetch the PR head, merge state, mergeability, and check summary:

```bash
PR_NUMBER="${PR_NUMBER:?set PR_NUMBER to the pull request number}"
gh pr view "$PR_NUMBER" \
  --json headRefName,headRefOid,isDraft,mergeStateStatus,mergeable,reviewDecision,changedFiles,statusCheckRollup
gh pr diff "$PR_NUMBER" --name-only
```

`statusCheckRollup` is green only when every required check for `headRefOid` has
completed successfully. A required check blocks readiness when it is pending,
queued, in progress, requested, failing, errored, timed out, skipped when the
branch protection requires it to run, cancelled, missing, or reported for a
different head.

If the head changes during review, stop and restart the readiness verification
for the newly requested SHA.

Changed-file scope is valid for scenario-link recovery only when every changed
file belongs to the scenario/docs-link lane:

```text
assets/scenarios/eatme/
assets/scenarios/gadugi/
crates/eatme-assets/src/gadugi*.rs
crates/eatme-assets/src/*scenario*link*tests*.rs
crates/eatme-assets/src/outside_in_alice_expansion_tests/
docs/
mkdocs.yml
```

Reject unrelated package metadata, unrelated CLI behavior, unrelated test
fixtures, and any generated file that cannot be tied back to the canonical
scenario or generated-runner contract.

## Scenario-link evidence boundary

Review canonical source scenarios when scenario-link, prerequisite, evidence, or
follow-on wording is part of the PR:

```text
assets/scenarios/eatme/
```

The canonical scenario YAML is the source of truth. Generated Gadugi YAML is a
consumer of that source and must not add independent behavior. Reader-facing
scenario links may connect prerequisites, first-lesson evidence, instructor
handoff, student next action, and follow-on paths only when those links are
represented by the editable scenario assets or generated docs.

Use plain wording for the first-lesson silver thread:

| Link surface | Acceptable claim |
| --- | --- |
| Prerequisites | Required tools, Alice homes, real-Alice gate, and scenario assets are named before execution. |
| Evidence | The run records only the manifest, log, window, screenshot, action-contract, and readiness fields named by the scenario or report. |
| Follow-on path | The next step is a bounded action or handoff, not a declaration that the lesson is complete. |
| Generated runner | Gadugi invokes eatme commands and checks emitted evidence; eatme owns Alice desktop launch behavior. |
| Documentation | Docs route readers from scenario authoring to generated runners, lesson readiness, and evidence interpretation without broad product claims. |

The wording must not say or imply that the scenario, generated adapter, or docs
claim:

| Unsupported claim | Required boundary |
| --- | --- |
| First-lesson completion | It is first-lesson readiness or first-action evidence only. |
| Grading or learner-world grading | It records evidence for review; it does not grade. |
| Creative assessment | It may name an editable change; it does not assess creativity. |
| Full UI automation | It records bounded launch evidence, action-contract evidence, and explicit gaps. |
| Visible rendering correctness | Screenshot or window evidence is observation evidence only. |
| Full Save completion | Save, reopen, and export remain readiness gaps until user-like evidence exists. |
| Complete Alice coverage | The scenario covers only its stated readiness contract. |

For starter-project preflight wording, keep the same boundary: opened starter
project, manifest/log/window/screenshot evidence, bounded starter-world notes,
and readiness-gap artifacts only.

## Generated Gadugi adapter freshness

Whenever a canonical scenario asset changes, the generated Gadugi adapter
freshness check is mandatory:

```bash
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
```

If the check reports stale or missing generated output, regenerate adapters and
run check mode again:

```bash
cargo run -q -p eatme-cli -- assets generate-gadugi --json
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
```

Commit the canonical scenario change and regenerated adapter change together.
When no scenario asset or generated adapter target is affected, adapter
freshness is not part of the readiness decision.

Validate committed scenario and persona assets:

```bash
cargo run -q -p eatme-cli -- assets validate --json
```

The validation gate passes only when the JSON report has `passed: true` and no
blocking errors.

## Scenario-link recovery procedure

Use this procedure when a pull request touches scenario links, generated runner
wording, or first-lesson evidence-path documentation. The recovery gate treats
the existing branch as the review surface. It records readiness for the checked
head and does not merge the pull request manually.

Use this bounded command set for the branch:

```bash
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
cargo run -q -p eatme-cli -- assets validate --json
mkdocs build --strict
TMPDIR=/tmp ./scripts/quality-gates.sh
```

The scenario-link recovery is ready only when:

1. The PR head matches the branch head being reviewed.
2. Canonical scenario links, prerequisites, learner-facing boundaries, and
   follow-on paths stay in `assets/scenarios/eatme/`.
3. Generated Gadugi adapters are reproducible from the generator.
4. Generated adapter descriptions use bounded evidence-source wording.
5. Scenario-link silver-thread tests cover reader routing without implying UI
   rendering, grading, creative assessment, or lesson completion.
6. Documentation states only executable checked-head evidence boundaries.

If the working tree already contains exactly the required generator, generated
adapter, test, and documentation changes, keep them and complete the checks. If
the current branch already satisfies every gate without additional repository
changes, record a no-op finalization instead of editing unrelated files.

## Draft and owner-free review handling

Draft status is a merge blocker. The finalization packet may still be useful
when a draft PR is otherwise clean and green, but its outcome is
`NOT_MERGE_READY` until the PR is intentionally marked ready for review.

Owner-free review means the workflow cannot infer approval from branch hygiene,
green checks, or mergeability. Capture `reviewDecision` exactly as GitHub reports
it. When it is empty, unavailable, or not approved, record that review is absent
or pending instead of converting it into approval.

Use this decision table:

| Condition | Final outcome |
| --- | --- |
| `isDraft=true` | `NOT_MERGE_READY` |
| Local `HEAD` differs from `headRefOid` | `NOT_MERGE_READY` |
| Required check is failing, pending, missing, skipped when required, or wrong-head | `NOT_MERGE_READY` |
| `mergeStateStatus` is not `CLEAN` or `mergeable` is not `MERGEABLE` | `NOT_MERGE_READY` |
| Changed-file scope includes unrelated files | `NOT_MERGE_READY` |
| Scenario assets, generated runners, docs build, or quality gate fail | `NOT_MERGE_READY` |
| Non-draft exact head has clean mergeability, acceptable review state, valid scope, fresh generated runners, passing assets, passing docs, passing quality gate, and green required checks | `MERGE_READY` |

If the workflow owns the transition from draft to ready, make that transition
explicit before the final metadata capture and repeat exact-head verification.
Do not silently ignore draft status.

## Review and finalization packet

The finalization packet is the evidence summary used by reviewers. It should be
short and tied to current-head checks, not to expected future behavior.

Include:

| Field | Content |
| --- | --- |
| Outcome | `MERGE_READY` or `NOT_MERGE_READY`. |
| Branch | The reviewed branch name. |
| Head | The exact commit SHA checked by GitHub metadata and local commands. |
| Draft status | `isDraft` value and whether it blocks merge readiness. |
| Review state | `reviewDecision` value or explicit owner-free/no-review note. |
| Checks | Current-head check summary. |
| Mergeability | `mergeStateStatus` and `mergeable`. |
| Scope | Scenario-link silver thread, generated Gadugi adapter wording, tests, and docs. |
| Commands | The repository-native commands that passed for that head. |
| Boundaries | Explicit non-claims for full UI automation, rendering correctness, grading, creative assessment, Save completion, lesson completion, and broad Alice coverage. |
| Merge handling | State that the workflow recorded readiness and did not manually merge. |
| Implementation output | Include `Files modified` when repository files changed, or `No-op justification:` when no further edit is needed. |

A no-op finalization is acceptable only when the branch is already clean or the
only dirty files are intentionally preserved generated/test/doc changes required
by the recovery, all required current checks pass for the current head, and no
additional repository edits would change the readiness result.

A no-op justification must name the current branch, current head SHA, current
draft status, current check state, current changed-file scope, and why editing
scenario/docs-link files would not improve the readiness outcome.

## Readiness note

Prepare readiness only after all required gates pass for the exact head. The
note should name the head and avoid broader product-readiness claims. Use
`NOT_MERGE_READY` when the exact head is draft, owner-free in a way that blocks
the repository's policy, or otherwise blocked.

Example:

```text
Outcome: MERGE_READY
Default-workflow readiness recorded for exact head <head-sha> on <branch>.

Verified gates: non-draft PR, exact PR head, green GitHub checks for that head, mergeStateStatus=CLEAN, mergeable=MERGEABLE, acceptable review state, bounded changed-file scope, bounded scenario-link wording, generated Gadugi adapter freshness, asset validation, strict documentation build, and repository quality gate.

Scope: scenario-link silver-thread asset/docs/generator validation only. This does not claim full UI automation, rendering correctness, grading, creative assessment, Save completion, lesson completion, or broad Alice compatibility.

Files modified: <changed files, or `No-op justification:` with the checked-head reason>
```

Draft example:

```text
Outcome: NOT_MERGE_READY
Default-workflow finalization recorded for exact head <head-sha> on <branch>.

Blocker: isDraft=true. Draft status is a merge blocker until the workflow intentionally marks the PR ready for review and repeats exact-head verification.

Current state: GitHub checks green for this head, mergeStateStatus=CLEAN, mergeable=MERGEABLE, reviewDecision=<value>, changed-file scope limited to scenario/docs-link recovery.

No-op justification: no focused scenario/docs-link edit is needed because generated runners are fresh, assets validate, docs build strictly, quality gates pass, and the only remaining blocker is draft status.
```

## Blocker handling

If any gate fails, do not record readiness. Fix only the minimal issue that
caused the blocker, run the relevant validation again, push the fix, and repeat
exact-head verification against the new PR head.

| Blocker | Minimal response |
| --- | --- |
| Head mismatch | Stop readiness for the old SHA and verify the requested new head. |
| Detached HEAD workspace | Check out the PR's real branch and verify local `HEAD` equals `headRefOid`. |
| Draft status | Report `NOT_MERGE_READY` or explicitly mark ready for review when the workflow owns that action, then repeat exact-head verification. |
| Owner-free or missing review state | Record the exact `reviewDecision`; do not invent approval. |
| Failing, pending, cancelled, missing, or wrong-head checks | Fix the failing check, wait for completion, or rerun the missing check before readiness. |
| Dirty merge state | Resolve only the mergeability issue. |
| Overclaiming scenario language | Edit the canonical scenario wording and regenerate adapters if affected. |
| Stale generated adapter | Regenerate adapters from canonical sources. |
| Asset validation failure | Fix the invalid scenario or persona asset. |
| Documentation build failure | Fix the broken doc link, heading, nav entry, or markdown issue. |
| Quality gate failure | Fix the failing repository-native check without broadening readiness claims. |
| Unrelated changes | Remove the unrelated change from the readiness work. |
