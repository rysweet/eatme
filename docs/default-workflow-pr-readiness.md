# Default-workflow PR readiness

Default-workflow PR readiness is the exact-head recovery gate used when a pull
request needs a clear readiness, review, or finalization decision and an outer
workflow did not produce useful output.

The workflow verifies the current checkout, validates the repository evidence
that applies to the PR, checks GitHub metadata for the same branch head, and
then records either a bounded readiness decision or a bounded no-op
justification. It does not merge the PR.

## Contents

- [Readiness contract](#readiness-contract)
- [Evidence record template](#evidence-record-template)
- [Generic readiness procedure](#generic-readiness-procedure)
- [Configuration](#configuration)
- [GitHub metadata fields](#github-metadata-fields)
- [Sharing-readiness recovery profile](#sharing-readiness-recovery-profile)
- [Generated Gadugi adapter freshness](#generated-gadugi-adapter-freshness)
- [No-op justification](#no-op-justification)
- [Readiness comment](#readiness-comment)
- [Blocker handling](#blocker-handling)

## Readiness contract

A PR is default-workflow ready only when every required gate passes for the
current branch head being reviewed.

| Gate | Required result |
| --- | --- |
| Current checkout | The worktree is on the intended branch and the current `HEAD` is recorded. |
| PR association | GitHub reports that the PR head branch is the same branch being recovered. |
| GitHub checks | Required checks are green for the PR head SHA. |
| Merge state | `mergeStateStatus` is `CLEAN`. |
| Mergeability | `mergeable` is `MERGEABLE`. |
| Asset validation | Persona and scenario assets validate successfully when the PR touches or documents asset behavior. |
| Gadugi freshness | Generated adapters are fresh when canonical scenario assets are involved. |
| Documentation build | `mkdocs build --strict` succeeds when documentation changes or readiness docs are part of the PR. |
| Quality gate | `./scripts/quality-gates.sh` succeeds when full repository readiness is required. |
| Claim boundary | The final statement names only the evidence that was executed for the current head. |
| Scope | Repository changes are limited to the minimal files needed to satisfy the evidence. |

A wrapper failure, rate-limit exit, or owner exit is not itself a blocker when
direct current-head verification passes and the final claim stays inside the
executed evidence boundary.

## Evidence record template

The workflow records evidence as a small, inspectable record. The record is a
review artifact, not a source file that must be committed.

| Field | Meaning |
| --- | --- |
| `repository` | Repository owner and name, such as `rysweet/eatme`. |
| `branch` | Local branch under review. |
| `head_sha` | Current local `HEAD` SHA from `git rev-parse HEAD`. |
| `worktree_status` | `git status --short --branch` result summarized as clean or dirty. |
| `pr_number` | Pull request number being recovered. |
| `pr_head_branch` | GitHub PR head branch from `headRefName`. |
| `pr_head_sha` | GitHub PR head SHA from `headRefOid`. |
| `checks` | Required check states for `pr_head_sha`. |
| `merge_state` | `mergeStateStatus` and `mergeable`. |
| `asset_validation` | Result of `assets validate --json`, when applicable. |
| `gadugi_freshness` | Result of `assets generate-gadugi --check --json`, when applicable. |
| `docs_build` | Result of `mkdocs build --strict`, when applicable. |
| `quality_gate` | Result of `TMPDIR=/tmp ./scripts/quality-gates.sh`, when full readiness is required. |
| `workflow_readiness_evidence` | Current-head workflow readiness summary tying the executed gates to the evaluated branch and SHA. |
| `review_evidence` | Review-relevant PR metadata, check rollup, and bounded claim review used to decide whether readiness can be posted. |
| `finalization_evidence` | Finalization-relevant state showing whether the workflow may record readiness, no-op acceptance, or a blocker without claiming merge completion. |
| `decision` | `ready`, `blocked`, or `no-op accepted`. |
| `bounded_claim` | Short statement of what the executed evidence proves and what it does not prove. |

## Generic readiness procedure

Run the gate from the repository root.

1. Confirm the branch, local `HEAD`, and worktree state:

   ```bash
   git --no-pager status --short --branch
   git --no-pager rev-parse --abbrev-ref HEAD
   git --no-pager rev-parse HEAD
   ```

2. Query the PR metadata for the PR being recovered:

   ```bash
   gh pr view 173 \
     --json number,title,headRefName,headRefOid,baseRefName,mergeStateStatus,mergeable,statusCheckRollup,reviewDecision,state,url
   ```

3. Validate persona and scenario assets:

   ```bash
   cargo run -q -p eatme-cli -- assets validate --json
   ```

4. Check generated Gadugi adapter freshness:

   ```bash
   cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
   ```

5. Build the documentation site in strict mode:

   ```bash
   mkdocs build --strict
   ```

6. Run the repository quality gate when full readiness is required:

   ```bash
   TMPDIR=/tmp ./scripts/quality-gates.sh
   ```

7. Inspect the relevant documentation, scenario assets, generated adapters, and
   guard tests for overbroad claims.

8. If all gates pass and no stale claims are found, record a no-op justification.
   If a gate fails because a document, scenario, adapter, or guard test is stale,
   make the smallest targeted change and rerun the affected gates plus the full
   quality gate.

Do not wrap these commands in shell `timeout` helpers. Long-running commands
should finish naturally or fail with their own diagnostics.

## Configuration

Use the repository's saved Node heap preference when Node-based wrappers or
repository workflows are involved:

```bash
export NODE_OPTIONS=--max-old-space-size=32768
```

The Rust asset validation and Gadugi generator commands do not require Node, but
keeping the variable exported is safe for repository-wide workflow runs.

Use a short temporary directory root for deep worktrees when running the quality
gate:

```bash
TMPDIR=/tmp ./scripts/quality-gates.sh
```

Use authenticated `gh` access only for read-only PR metadata checks and comments.
Do not place tokens, secrets, local credential paths, environment dumps, or raw
credential output in readiness records or PR comments.

## GitHub metadata fields

The readiness gate consumes these `gh pr view` fields:

| Field | Required value |
| --- | --- |
| `headRefName` | The PR branch being recovered. |
| `headRefOid` | The PR head SHA that GitHub checks and mergeability describe. |
| `mergeStateStatus` | `CLEAN`. |
| `mergeable` | `MERGEABLE`. |
| `statusCheckRollup` | Required checks green for `headRefOid`. |
| `reviewDecision` | Review state used as review/finalization context, not as a replacement for executable evidence. |
| `state` | The PR remains open unless a separate merge workflow closes it. |

`statusCheckRollup` is green only when every required check for `headRefOid` has
completed successfully. A required check blocks readiness when it is pending,
queued, in progress, requested, failing, errored, timed out, skipped when branch
protection requires it to run, cancelled, missing, or reported for a different
head.

If the local `HEAD` differs from `headRefOid`, the recovery record must say which
state was evaluated. Do not describe local validation as proof for the published
PR head unless the SHAs match or the checked files are intentionally uncommitted
documentation being prepared for that head.

## Sharing-readiness recovery profile

Use this profile for PRs that recover classroom sharing readiness, including PR
`#173` on branch `wave6-deployed-sharing-gap-1778302300`.

| Surface | Required boundary |
| --- | --- |
| `docs/sharing-readiness-boundary.md` | Describes classroom review handoffs, not hosted sharing or deployment. |
| `docs/default-workflow-pr-readiness.md` | Describes current-head evidence collection, no-op justification, and bounded finalization. |
| `assets/scenarios/eatme/student-artifact-package-share-evidence.yaml` | Student packet contract for artifact reference, student change, visible run result, attribution or context, next revision, and review boundary. |
| `assets/scenarios/eatme/teacher-community-sharing-loop.yaml` | Teacher-facing share card, classroom handoff note, accessibility notes, attribution, student evidence expectations, and remix feedback. |
| `assets/scenarios/eatme/first-lessons-real-ui-actions.yaml` | Real Alice action contract; not a full UI automation pass. |
| `assets/scenarios/gadugi/*.yaml` | Generated adapters must preserve source scenario boundaries and stay fresh. |
| Rust guard tests | Enforce the sharing-readiness boundary and generated adapter linkage. |

The final PR #173 statement may say that current-head evidence supports bounded
classroom sharing-readiness review artifacts only when the gates above pass. It
must not claim hosted sharing, deployed sharing, platform success, full UI
automation, rendering correctness, grading correctness, creative assessment, Save
completion, lesson completion, production readiness, deployment success, merge
completion, or manual merge.

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
still may be run as current-head evidence, but it should not be described as
proof of behavior outside the generated asset contract.

## No-op justification

A workflow-accepted no-op justification is accepted when current-head evidence,
review evidence, and finalization evidence prove that no repository changes were
required.

The justification should include:

| Item | Required content |
| --- | --- |
| Branch and head | Local branch and `HEAD` SHA. |
| Worktree state | Clean state, or a narrow explanation of documentation-only changes being finalized. |
| PR metadata | PR number, head branch, head SHA, merge state, mergeability, and check summary. |
| Executed gates | Commands that passed for the evaluated state. |
| Claim boundary | The exact readiness claim and explicit non-claims. |
| No-op reason | Why docs, assets, generated adapters, and tests already satisfy the contract. |

Example no-op wording:

```text
Default-workflow no-op recovery accepted for PR #173 at current branch head
${HEAD_SHA}. Current-head evidence passed for asset validation, generated Gadugi
freshness, strict documentation build, quality gates, review evidence, and PR
metadata review.

No repository changes were required because the committed sharing-readiness docs,
scenario assets, generated adapters, and guard tests already preserve the
classroom review handoff boundary. Finalization evidence records that no manual
merge was performed.

This records bounded silver-thread/e2e sharing-readiness evidence only. It does
not claim hosted sharing, deployed sharing, platform success, full UI
automation, rendering correctness, grading correctness, creative assessment,
Save completion, lesson completion, production readiness, deployment success, or
merge completion.
```

## Readiness comment

Publish readiness only after all required gates pass for the evaluated head. The
comment should name the head and avoid broader product-readiness claims.

Create a comment body from the evidence record:

```bash
HEAD_SHA="$(git rev-parse HEAD)"
cat > readiness-comment.txt <<EOF
Default-workflow recovery recorded for PR #173 at current branch head ${HEAD_SHA}.

Verified current-head gates: asset validation, generated Gadugi freshness,
strict documentation build, quality gates, PR metadata review, and bounded
sharing-readiness claim review.

The recovery supports classroom sharing handoff readiness only. It does not
claim hosted sharing, deployed sharing, platform success, full UI automation,
rendering correctness, grading correctness, creative assessment, Save
completion, lesson completion, production readiness, deployment success, merge
completion, or manual merge.
EOF
```

Post the comment with:

```bash
gh pr comment 173 --body-file readiness-comment.txt
```

Do not post readiness when any gate is failing, pending, stale, or tied to a
different head without an explicit state separation.

## Blocker handling

If any gate fails, do not publish readiness. Fix only the minimal issue that
caused the blocker, run the relevant validation again, and repeat current-head
verification.

| Blocker | Minimal response |
| --- | --- |
| Wrong branch | Switch to the PR branch worktree or stop recovery for the current checkout. |
| Local/PR head mismatch | State the mismatch and verify the intended head before making readiness claims. |
| Failing, pending, cancelled, missing, or wrong-head checks | Fix the failing check, wait for completion, or rerun the missing check before readiness. |
| Dirty merge state | Resolve only the mergeability issue. |
| Overclaiming docs or scenario language | Edit the canonical documentation or scenario wording and rerun affected gates. |
| Stale generated adapter | Regenerate adapters from canonical sources. |
| Asset validation failure | Fix the invalid scenario or persona asset. |
| Documentation build failure | Fix the broken doc, navigation, link, or MkDocs configuration. |
| Quality gate failure | Fix the failing repository gate without bypassing it. |
| Unrelated changes | Remove the unrelated change from the readiness work. |
