# Default-workflow PR readiness

Default-workflow PR readiness is the exact-head gate used when a pull request
needs a clear final readiness decision and the wrapper workflow did not produce
useful output.

This page specifies the readiness behavior the workflow should enforce: verify
the exact PR head, inspect GitHub metadata for that same head, preserve the
bounded starter-project evidence contract, require fresh generated Gadugi
adapters when scenario assets are involved, and publish a narrowly scoped
readiness comment only after every gate passes.

## Contents

- [Readiness contract](#readiness-contract)
- [Generic readiness procedure](#generic-readiness-procedure)
- [Configuration](#configuration)
- [GitHub metadata fields](#github-metadata-fields)
- [Starter-project evidence boundary](#starter-project-evidence-boundary)
- [Generated Gadugi adapter freshness](#generated-gadugi-adapter-freshness)
- [Silver-thread/e2e gap-matrix readiness](#silver-threade2e-gap-matrix-readiness)
- [PR #193 finalization evidence boundary](#pr-193-finalization-evidence-boundary)
- [PR #164 readiness example](#pr-164-readiness-example)
- [No-op finalization record](#no-op-finalization-record)
- [Readiness comment](#readiness-comment)
- [Blocker handling](#blocker-handling)

## Readiness contract

A PR is default-workflow ready only when every gate passes for the exact commit
being reviewed.

| Gate | Required result |
| --- | --- |
| Exact head | The PR head SHA equals the requested SHA. A mismatch blocks readiness. |
| GitHub checks | Required checks are green for that same SHA. |
| Merge state | `mergeStateStatus` is `CLEAN`. |
| Mergeability | `mergeable` is `MERGEABLE`. |
| Starter-project wording | The canonical scenario uses plain, bounded, user-facing language. |
| Overclaim boundary | The scenario does not claim first-lesson completion, grading, creative assessment, full UI automation, visible rendering correctness, full Save completion, or complete Alice coverage. |
| Gadugi adapters | Generated adapters are fresh whenever canonical scenario assets are affected. |
| Silver-thread/e2e gap matrix | When the PR touches lesson-session readiness docs, the scenario map and gap matrix name only the bounded user journey, remaining missing proof, and evidence still needed. |
| Scope | No unrelated files or behavior are changed. |

A previous wrapper failure is not a blocker when direct verification proves the
same head, green checks, clean mergeability, bounded wording, and fresh
generated adapters.

## Generic readiness procedure

Run the gate in this order:

1. Verify the PR head equals the exact requested SHA.
2. Verify GitHub checks are green for that same SHA.
3. Verify `mergeStateStatus=CLEAN` and `mergeable=MERGEABLE`.
4. Inspect the starter-project preflight scenario wording if the PR touches that
   evidence contract.
5. Run the generated Gadugi adapter freshness check if any canonical scenario
   asset is affected.
6. Inspect the lesson-session scenario map and gap matrix if the PR touches
   silver-thread/e2e readiness documentation.
7. Validate assets.
8. Publish the readiness comment only when every required gate passed.

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

For GitHub checks, use authenticated `gh` access to the repository that owns the
PR. Do not place tokens, secrets, local credential paths, environment dumps, or
raw command output in readiness comments.

## GitHub metadata fields

The readiness gate consumes these `gh pr view` fields:

| Field | Required value |
| --- | --- |
| `headRefOid` | Exact requested SHA |
| `mergeStateStatus` | `CLEAN` |
| `mergeable` | `MERGEABLE` |
| `statusCheckRollup` | Required checks green for `headRefOid` |

Fetch the PR head, merge state, mergeability, and check summary:

```bash
gh pr view 164 \
  --json headRefOid,mergeStateStatus,mergeable,statusCheckRollup
```

`statusCheckRollup` is green only when every required check for `headRefOid` has
completed successfully. A required check blocks readiness when it is pending,
queued, in progress, requested, failing, errored, timed out, skipped when the
branch protection requires it to run, cancelled, missing, or reported for a
different head.

If the head changes during review, stop and restart the readiness verification
for the newly requested SHA.

## Starter-project evidence boundary

Review the canonical source scenario when starter-project preflight wording is
part of the PR:

```text
assets/scenarios/eatme/starter-project-open-save-export-preflight.yaml
```

The wording must stay plain and bounded. It may say that the scenario records
real Alice launch/opened-project evidence for the bundled starter project, an
editable starter-world change note, an attempted run or observation, and
readiness-gap notes.

When older wording or generated output uses the phrase "action evidence," read it
only as bounded launch/opened-project evidence. It does not mean user-like UI
automation, save/reopen/export completion, learner-world grading, or creative
assessment.

The wording must not say or imply that the scenario proves:

| Unsupported claim | Required boundary |
| --- | --- |
| First-lesson completion | It is starter-project preflight evidence only. |
| Grading or learner-world grading | It records evidence for review; it does not grade. |
| Creative assessment | It may name an editable change; it does not assess creativity. |
| Full UI automation | It records bounded launch/opened-project evidence and explicit gaps. |
| Visible rendering correctness | Screenshot or window evidence is observation evidence only. |
| Full Save completion | Save, reopen, and export remain readiness gaps until user-like evidence exists. |
| Complete Alice coverage | The scenario covers only the stated preflight contract. |

Use the generated adapter only as a consumer of this contract. Do not hand-edit
generated Gadugi YAML to change mission intent.

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
When no scenario asset or generated adapter target is affected, adapter freshness
is not part of the readiness decision.

Validate committed scenario and persona assets:

```bash
cargo run -q -p eatme-cli -- assets validate --json
```

The validation gate passes only when the JSON report has `passed: true` and no
blocking errors.

## Silver-thread/e2e gap-matrix readiness

Use this lane when a PR documents the lesson-session silver thread instead of
adding runtime Alice automation. The documentation is ready only when it gives a
reader the same bounded implementation specification that the tests enforce:

| Documentation surface | Required content |
| --- | --- |
| Scenario map | The canonical scenario rows are present, ordered, and written in user-facing language. |
| Scenario-to-gap matrix | Each row states what the user is trying to do, what the current evidence does not yet show, and what evidence would be needed before a broader capability could be claimed. |
| Evidence boundary | The page says the lane verifies documentation, asset validation, adapter freshness, and exact-head PR evidence only. |
| Non-claims | The page keeps full UI automation, rendering correctness, grading, creative assessment, Save completion, and lesson-completion claims explicitly outside the lane unless separate exact-head evidence proves them. |

This lane is allowed to support silver-thread/e2e readiness language because it
connects scenario intent, missing proof, and executable repository checks. It is
not allowed to imply live classroom use, runtime rendering correctness, automated
assessment, or completion of the Alice lesson journey.

For documentation-only changes, use the exact same command evidence as a code
change unless the workflow has an accepted no-op reason. A no-op is acceptable
only when the current PR head already contains the required documentation,
doc-test guardrails, and test-module wiring, and the reviewer records the exact
head, command outcomes, GitHub check state, and no-merge decision.

Do not treat uncommitted local documentation edits as evidence for a PR head.
Commit and push the docs first, then gather fresh command and GitHub evidence for
the new `headRefOid`.

## PR #193 finalization evidence boundary

PR #193 uses this committed section only as a readiness contract. Do not record a
specific PR #193 `headRefOid`, evidence timestamp, or "passed for this head"
claim inside this file: the commit that changes this file also changes the PR
head, so committed exact-head evidence here would become self-stale.

After the final commit is pushed, gather fresh exact-head evidence with:

```bash
gh pr view 193 --json number,url,state,headRefName,headRefOid,mergeStateStatus,mergeable,statusCheckRollup
```

Record the resulting `headRefOid`, command outcomes, GitHub check state, bounded
change scope, and no-manual-merge decision outside the repository commit, such as
in the PR body, PR comment, or status summary. Do not treat uncommitted local
documentation edits as evidence for a PR head.

The external PR #193 evidence must include these command outcomes for the final
pushed head:

| Command | Required result |
| --- | --- |
| `cargo run -q -p eatme-cli -- assets validate --json` | JSON report has `passed: true` and no errors. |
| `cargo run -q -p eatme-cli -- assets generate-gadugi --check --json` | JSON report has `passed: true` and no changed generated adapters. |
| `mkdocs build --strict` | Strict documentation build succeeds. |
| `TMPDIR=/tmp ./scripts/quality-gates.sh` | Repository quality gate exits successfully. |

The allowed readiness conclusion for PR #193 is narrow: the final pushed head may
support only the lesson-session silver-thread/e2e gap-matrix documentation lane,
with asset validation, generated adapter freshness, strict docs build,
repository quality gates, and GitHub checks tied to the externally recorded exact
head. It does not claim full UI automation, rendering correctness, grading,
creative assessment, Save completion, lesson completion, live classroom use,
manual merge completion, or any evidence from uncommitted local files.

## PR #164 readiness example

This subsection is a concrete example for the PR #164 finalization gate. Do not
reuse its PR number or SHA for future readiness decisions.

For PR #164, the exact accepted head is:

```text
eb0bb29b7cc1f8647e9a36c0bc8200fb3fdc5cba
```

The GitHub metadata gate passes only when `gh pr view 164 --json
headRefOid,mergeStateStatus,mergeable,statusCheckRollup` reports:

```json
{
  "headRefOid": "eb0bb29b7cc1f8647e9a36c0bc8200fb3fdc5cba",
  "mergeStateStatus": "CLEAN",
  "mergeable": "MERGEABLE"
}
```

Because PR #164 changes starter-project scenario wording and generated Gadugi
output, these gates are mandatory for that PR:

```bash
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
cargo run -q -p eatme-cli -- assets validate --json
```

The readiness decision for PR #164 is valid only if those commands pass, the
GitHub checks are green for
`eb0bb29b7cc1f8647e9a36c0bc8200fb3fdc5cba`, and the scenario wording stays
within the starter-project evidence boundary above.

## No-op finalization record

Use a no-op finalization only when the repository already contains the required
implementation and documentation at the exact PR head. The record must include:

| Field | Required value |
| --- | --- |
| PR and branch | PR number and exact branch name. |
| Exact head | `headRefOid` evaluated after local checks and immediately before finalization. |
| Diff scope | The changed files reviewed for the readiness claim. |
| Local commands | Each repository-defined command, its exit result, and the bounded fact it proves. |
| GitHub state | PR state, merge state, and check summary for the same head. |
| No-op reason | A plain statement that no repository file changes are required because the current head already satisfies the documented contract. |
| No-merge statement | A statement that readiness was recorded without manually merging the PR. |

The no-op record must not use future-tense assumptions. If any command fails,
the PR head changes, or the reviewed docs imply unsupported runtime claims, the
workflow is not a no-op; fix the minimal issue and gather fresh evidence for the
new head.

A dirty worktree is never a no-op finalization source. Local edits that have not
been committed and pushed make any exact-head readiness record stale.

## Readiness comment

Publish readiness only after all required gates pass for the exact head. The
comment should name the head and avoid broader product-readiness claims.

Example:

```text
Default-workflow readiness recorded for PR #164 at exact head eb0bb29b7cc1f8647e9a36c0bc8200fb3fdc5cba.

Verified gates: exact PR head, green GitHub checks for that head, mergeStateStatus=CLEAN, mergeable=MERGEABLE, bounded starter-project preflight wording, no unsupported claims for first-lesson completion/grading/creative assessment/full UI automation/visible rendering correctness/full Save completion, generated Gadugi adapter freshness, and asset validation.

The prior non-zero wrapper exit is not treated as a blocker because direct verification passed at this exact head.
```

Post the comment with:

```bash
gh pr comment 164 --body-file readiness-comment.txt
```

## Blocker handling

If any gate fails, do not publish readiness. Fix only the minimal issue that
caused the blocker, run the relevant validation again, push the fix, and repeat
exact-head verification against the new PR head.

| Blocker | Minimal response |
| --- | --- |
| Head mismatch | Stop readiness for the old SHA and verify the requested new head. |
| Failing, pending, cancelled, missing, or wrong-head checks | Fix the failing check, wait for completion, or rerun the missing check before readiness. |
| Dirty merge state | Resolve only the mergeability issue. |
| Overclaiming scenario language | Edit the canonical scenario wording and regenerate adapters if affected. |
| Stale generated adapter | Regenerate adapters from canonical sources. |
| Asset validation failure | Fix the invalid scenario or persona asset. |
| Unrelated changes | Remove the unrelated change from the readiness work. |
