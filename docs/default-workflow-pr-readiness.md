# Default-workflow PR readiness

Default-workflow PR readiness is the no-timeout exact-head recovery gate used
when a pull request needs a clear readiness, review, or finalization decision
and an outer workflow did not produce useful output.

The workflow verifies the current checkout, validates the repository evidence
that applies to the PR, checks GitHub metadata for the same branch head, and
then records either a bounded readiness decision or a bounded no-op
justification. It does not merge the PR.

There is no single repository command named "finalization recovery." Treat this
guide as the executable evidence checklist: run the existing repository commands
and GitHub metadata queries below, then produce an owner-free handoff note. Post
that note as a PR comment or body update only when explicitly authorized.

## Contents

- [Quick start](#quick-start)
- [Readiness contract](#readiness-contract)
- [Evidence record template](#evidence-record-template)
- [Recovery component reference](#recovery-component-reference)
- [Generic readiness procedure](#generic-readiness-procedure)
- [Configuration](#configuration)
- [GitHub metadata fields](#github-metadata-fields)
- [Preserved patch recovery](#preserved-patch-recovery)
- [Sharing-readiness recovery profile](#sharing-readiness-recovery-profile)
- [Generated Gadugi adapter freshness](#generated-gadugi-adapter-freshness)
- [Three-cycle quality audit](#three-cycle-quality-audit)
- [Merge-ready decision](#merge-ready-decision)
- [No-op justification](#no-op-justification)
- [Readiness comment](#readiness-comment)
- [Blocker handling](#blocker-handling)

## Quick start

Use this path when a PR is already expected to be clean, but the previous
workflow stopped before producing owner-free finalization evidence.

The PR head reported by GitHub is the source of truth. Resolve the live head
first; for the PR `#173` recovery profile, set `PR_NUMBER=173`:

```bash
export NODE_OPTIONS=--max-old-space-size=32768
PR_NUMBER=173
PR_HEAD_SHA="$(gh pr view "${PR_NUMBER}" --json headRefOid --jq .headRefOid)"
```

Move the local checkout to the same branch head without rewriting history:

```bash
gh pr checkout "${PR_NUMBER}"
git pull --ff-only
LOCAL_HEAD_SHA="$(git rev-parse HEAD)"
test "${LOCAL_HEAD_SHA}" = "${PR_HEAD_SHA}"
```

Read checks and mergeability for the same SHA:

```bash
gh pr view "${PR_NUMBER}" \
  --json headRefOid,headRefName,state,mergeStateStatus,mergeable,isDraft,reviewDecision,statusCheckRollup,url
```

If the local head matches `headRefOid`, the worktree is clean, the GitHub checks
are current and green for that SHA, and the scoped review finds no gap, the
handoff records a literal `No-op` justification. If any evidence is stale,
missing, red, ambiguous, or tied to a different SHA, run the repository checks
that cover the affected surface before making the decision.

Immediately before producing the final handoff note, re-read GitHub's PR head and
require it to match the SHA that was validated:

```bash
FINAL_PR_HEAD_SHA="$(gh pr view "${PR_NUMBER}" --json headRefOid --jq .headRefOid)"
test "${FINAL_PR_HEAD_SHA}" = "${PR_HEAD_SHA}"
test "$(git rev-parse HEAD)" = "${FINAL_PR_HEAD_SHA}"
gh pr view "${PR_NUMBER}" \
  --json headRefOid,mergeStateStatus,mergeable,statusCheckRollup,reviewDecision,state,url
```

If either equality check fails, the PR moved after validation. Stop and report
`NOT_MERGE_READY` instead of emitting readiness or `No-op`.

Do not wrap the workflow or validation commands in shell `timeout` helpers. A
long-running command should complete normally or fail with its own diagnostics.

## Readiness contract

A PR is default-workflow ready only when every required gate passes for the
current branch head being reviewed.

| Gate | Required result |
| --- | --- |
| Current checkout | The worktree is on the intended branch, the current `HEAD` is recorded, and the final validation worktree is clean. |
| PR association | GitHub reports that the PR head branch is the same branch being recovered. |
| Preserved recovery patch | When recovery depends on a saved uncommitted patch, the patch is readable, inspected directly, and compared with the current branch before any no-op or readiness decision. |
| GitHub checks | Required checks are green for the PR head SHA. |
| Merge state | `mergeStateStatus` is `CLEAN`. |
| Mergeability | `mergeable` is `MERGEABLE`. |
| Asset validation | Persona and scenario assets validate successfully when the PR touches or documents asset behavior, or when GitHub evidence for that surface is stale, missing, red, or ambiguous. |
| Gadugi freshness | Generated adapters are fresh when canonical scenario assets are involved, or when adapter freshness is otherwise the evidence gap. |
| Documentation build | `mkdocs build --strict` succeeds when documentation changes, readiness docs are part of the PR, or docs evidence is otherwise missing. |
| Quality gate | `./scripts/quality-gates.sh` succeeds when full repository readiness is required or GitHub evidence is insufficient for the current head. |
| Runnable QA | Current-head command evidence covers only the assets, generated adapters, tests, docs, and repository gates that apply to the PR scope or close an evidence gap. |
| Quality audit | At least three SEEK / VALIDATE / FIX cycles have been completed, and the final cycle is clean. |
| PR description | The PR body or readiness handoff contains current-head evidence and no stale SHA-bound readiness claims. |
| Claim boundary | The final statement names only the evidence that was executed for the current head. |
| Scope | Repository changes are limited to the minimal files needed to satisfy the evidence. |

A wrapper failure, rate-limit exit, or owner-free exit classified as
`NO_OP_GUARD` is not itself a blocker when direct current-head verification
passes and the final claim stays inside the executed evidence boundary. A
`NO_OP_GUARD` owner-free exit must not be treated as `MERGE_READY` until the
workflow records direct current-head verification, then emits either a
workflow-accepted no-op justification or `NOT_MERGE_READY` blockers.

Green checks, including green GitHub Actions, and workflow completion are
necessary but not sufficient. The final decision also needs applicable runnable
QA/scenario evidence, documentation impact review, focused diff scope, PR
description evidence, and three quality-audit SEEK / VALIDATE / FIX cycles with
a clean final cycle.

## Evidence record template

The workflow records evidence as a small, inspectable record. The record is a
review artifact, not a source file that must be committed.

| Field | Meaning |
| --- | --- |
| `repository` | Repository owner and name, such as `rysweet/eatme`. |
| `evidence_collected_at` | UTC timestamp when GitHub metadata and local evidence were collected. |
| `branch` | Local branch under review. |
| `head_sha` | Current local `HEAD` SHA from `git rev-parse HEAD`. |
| `worktree_status` | `git status --short --branch` result. Readiness evidence is accepted only from a clean final worktree. |
| `pr_number` | Pull request number being recovered. |
| `pr_head_branch` | GitHub PR head branch from `headRefName`. |
| `pr_head_sha` | GitHub PR head SHA from `headRefOid`. |
| `preserved_patch_review` | Required when a saved uncommitted patch is part of recovery. Records the patch source, inspection result, affected paths, and whether the patch is already represented by the current branch. |
| `checks` | Required check names, conclusions, and source URLs for `pr_head_sha`. |
| `merge_state` | `mergeStateStatus` and `mergeable`. |
| `asset_validation` | Result of `assets validate --json`, when applicable. |
| `gadugi_freshness` | Result of `assets generate-gadugi --check --json`, when applicable. |
| `docs_build` | Result of `mkdocs build --strict`, when applicable. |
| `relevant_tests` | Focused Rust tests or other repository tests that exercise the PR-specific readiness guards, when applicable. |
| `quality_gate` | Result of `TMPDIR=/tmp ./scripts/quality-gates.sh`, when full readiness is required or GitHub evidence is insufficient. |
| `docs_impact` | Documentation files reviewed, strict build result when docs are in scope, and unsupported claims removed or confirmed absent. |
| `quality_audit_cycles` | Three SEEK / VALIDATE / FIX cycles, including the clean final cycle. |
| `diff_scope` | Changed files grouped by surface, with unrelated changes called out as blockers. |
| `pr_description_evidence` | PR body or readiness handoff evidence tied to the evaluated head and free of stale readiness claims. |
| `workflow_readiness_evidence` | Current-head workflow readiness summary tying the executed gates to the evaluated branch and SHA. |
| `review_evidence` | Review-relevant PR metadata, check rollup, and bounded claim review used to decide whether readiness can be recorded. |
| `finalization_evidence` | Finalization-relevant state showing whether the workflow may record readiness, no-op acceptance, or a blocker without claiming merge completion. |
| `decision` | `MERGE_READY`, `NOT_MERGE_READY`, or `BLOCKED`, with explicit blockers or evidence. A no-op recovery that passes every applicable gate is recorded as `MERGE_READY` with a no-op justification. `BLOCKED` means required recovery evidence could not be inspected, so no readiness decision was made. |
| `bounded_claim` | Short statement of what the executed evidence proves and what it does not prove. |

## Recovery component reference

The recovery workflow is composed of small, auditable components. These names
describe responsibilities in the evidence record; they are not separate binaries.

| Component | Input | Output | Failure behavior |
| --- | --- | --- | --- |
| `pr-head-resolver` | PR number and authenticated `gh` access | Live `headRefOid`, `headRefName`, state, merge fields, review decision, status rollup, and PR URL | Stop before readiness if GitHub cannot return the current head metadata. |
| `local-head-verifier` | Live `headRefOid` and local repository checkout | Confirmation that `git rev-parse HEAD` exactly equals the PR head SHA, plus clean/dirty worktree state | Emit `NOT_MERGE_READY` when local `HEAD` differs from the PR head or the final worktree is dirty. |
| `check-evidence-reader` | PR number and live `headRefOid` | Exact required check and check-run names, conclusions, workflow names, and source URLs for the same SHA | Treat missing, pending, failed, cancelled, stale, skipped-required, or wrong-head checks as blockers. |
| `repo-validation-runner` | Evidence gap and PR scope | Focused or repository-standard validation output for the matched local head | Run only when GitHub evidence is stale, missing, red, ambiguous, or insufficient for the scope; do not substitute old logs or require unrelated supplemental gates. |
| `scope-gate` | PR diff, worktree status, readiness docs, scenarios, generated adapters, and guard tests | Confirmation that changes stay inside the recovery/finalization scope | Reject unrelated edits and overbroad claims before recording readiness or no-op output. |
| `handoff-evidence-writer` | Current-head evidence, final PR-head re-check, validation results, scope review, and claim boundary | Self-contained owner-free final note; PR comment or body update only when explicitly authorized | Emit `MERGE_READY`, `NOT_MERGE_READY`, or `BLOCKED`; use literal `No-op` only when no repository change is required and every applicable gate is clean for the current head. |

## Generic readiness procedure

Run the gate from the repository root. Use only existing repository validation
commands and focused tests that already exist; name every command in the
handoff evidence.

Record an evidence collection timestamp, then resolve the live PR head:

```bash
date -u +"%Y-%m-%dT%H:%M:%SZ"
PR_NUMBER="${PR_NUMBER:?set PR_NUMBER to the pull request number}"
PR_HEAD_SHA="$(gh pr view "${PR_NUMBER}" --json headRefOid --jq .headRefOid)"
```

1. Query the full PR metadata for the PR being recovered:

   ```bash
   gh pr view "${PR_NUMBER}" \
     --json number,title,headRefName,headRefOid,baseRefName,isDraft,mergeStateStatus,mergeable,statusCheckRollup,reviewDecision,state,url
   ```

   Expand `statusCheckRollup` into exact check or check-run names and
   conclusions. Do not record only a summarized check status without the
   concrete check details it summarizes.

2. Fetch or check out the PR branch, then require local `HEAD` to match the live
   `headRefOid` before judging readiness:

   ```bash
   gh pr checkout "${PR_NUMBER}"
   git pull --ff-only
   git --no-pager rev-parse --abbrev-ref HEAD
   git --no-pager rev-parse HEAD
   test "$(git rev-parse HEAD)" = "${PR_HEAD_SHA}"
   ```

   If the equality check fails, stop readiness and report `NOT_MERGE_READY` with
   the mismatch. Local validation is not proof for the PR unless local `HEAD`
   exactly equals the live PR head SHA.

3. Confirm the branch, local `HEAD`, and worktree state:

   ```bash
   git --no-pager status --short --branch
   ```

   The final validation evidence is accepted only when this status is clean.
   Uncommitted documentation being prepared for the same head may be built or
   reviewed during recovery, but it is not final readiness evidence until it is
   committed or explicitly separated from the readiness claim.

4. Inspect the preserved recovery patch when the workflow provides one. Read the
   patch directly, record its affected paths and claims, compare those changes
   with the current branch, and stop with `BLOCKED` if the patch cannot be read
   or validated.

   Do not infer patch coverage from matching-looking repository state alone. For
   example, a version value in `pyproject.toml` is only an observation until the
   preserved patch itself shows that the value was part of the recovered change.

5. Validate persona and scenario assets when the PR touches or documents asset
   behavior, or when GitHub check evidence for assets is stale, missing, red, or
   ambiguous:

   ```bash
   cargo run -q -p eatme-cli -- assets validate --json
   ```

6. Check generated Gadugi adapter freshness when canonical scenario assets or
   generated adapters are in scope:

   ```bash
   cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
   ```

7. Build the documentation site in strict mode when documentation changes or
   readiness docs are part of the PR:

   ```bash
   mkdocs build --strict
   ```

8. Run the repository quality gate when full readiness is required or when
   GitHub evidence is insufficient for the current head. Do not require this
   supplemental gate for an unrelated surface when current GitHub checks already
   provide adequate evidence for the PR scope:

   ```bash
   TMPDIR=/tmp ./scripts/quality-gates.sh
   ```

9. When committing a recovered repository change, let the repository's commit
   hooks run. If the global `pre-commit` hook is installed but this repository
   has no `.pre-commit-config.yaml`, use `PRE_COMMIT_ALLOW_NO_CONFIG=1` only
   because the repository has no pre-commit config and the project uses Cargo and
   MkDocs quality gates instead of a pre-commit-managed hook set.

10. Run focused tests for the PR-specific guard behavior when such tests exist.
   For the sharing-readiness guard tests, run:

   ```bash
   cargo test -q -p eatme-assets outside_in_alice_expansion_tests
   ```

11. Inspect the changed-file list and reject unrelated scope expansion:

   ```bash
   gh pr diff "${PR_NUMBER}" --name-only
   ```

12. Inspect the relevant documentation, scenario assets, generated adapters,
    guard tests, and PR description for overbroad or stale claims.

13. Complete three quality-audit cycles. Each cycle records a SEEK target, the
    VALIDATE command or inspection used, and the FIX result. If no repository
    change is required, the FIX result states why the current head already
    satisfies the target.

14. Immediately before final handoff, re-read `headRefOid` and require both the
    previously validated PR head and local `HEAD` to still match:

    ```bash
    FINAL_PR_HEAD_SHA="$(gh pr view "${PR_NUMBER}" --json headRefOid --jq .headRefOid)"
    test "${FINAL_PR_HEAD_SHA}" = "${PR_HEAD_SHA}"
    test "$(git rev-parse HEAD)" = "${FINAL_PR_HEAD_SHA}"
    gh pr view "${PR_NUMBER}" \
      --json headRefOid,mergeStateStatus,mergeable,statusCheckRollup,reviewDecision,state,url
    ```

    If the PR head moved, stop and report `NOT_MERGE_READY`; prior local
    validation belongs to the old head and cannot support a no-op or readiness
    handoff for the new head.

15. If all applicable gates pass and no stale claims are found, record
    `MERGE_READY`. When no repository changes are needed, record `MERGE_READY`
    with a no-op justification instead of treating no-op as a separate readiness
    state. If a gate fails because a document, scenario, adapter, test, check,
    worktree state, or PR description is stale, make the smallest targeted change
    and rerun the affected gates plus any broader validation needed to close the
    evidence gap.

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

Use authenticated `gh` access for read-only PR metadata checks by default. Use it
to post comments or update PR text only when the workflow explicitly authorizes
that mutation. Do not place tokens, secrets, local credential paths, environment
dumps, or raw credential output in readiness records or PR comments.

## GitHub metadata fields

The readiness gate consumes these `gh pr view` fields:

| Field | Required value |
| --- | --- |
| `headRefName` | The PR branch being recovered. |
| `headRefOid` | The PR head SHA that GitHub checks and mergeability describe. |
| `mergeStateStatus` | `CLEAN`. |
| `mergeable` | `MERGEABLE`. |
| `statusCheckRollup` | Required check and check-run names, conclusions, `detailsUrl`, `workflowName`, and source URLs for `headRefOid`. |
| `reviewDecision` | Review state used as review/finalization context, not as a replacement for executable evidence. |
| `state` | The PR remains open unless a separate merge workflow closes it. |

`statusCheckRollup` is green only when every required check for `headRefOid` has
completed successfully. Record the exact check-run names, conclusions,
`detailsUrl`, `workflowName`, and source URLs so reviewers can trace each status
back to the GitHub run or external check that produced it. A required check
blocks readiness when it is pending, queued, in progress, requested, failing,
errored, timed out, skipped when branch protection requires it to run,
cancelled, missing, or reported for a different head.

Older PR-body claims tied to previous SHAs are context only. Ignore them, or
supersede them with a timestamped current-head handoff note before using the PR
description as readiness evidence. Post that note back to GitHub only when the
workflow explicitly authorizes a PR comment or body update.

If the local `HEAD` differs from `headRefOid`, the recovery record must say which
state was evaluated. Do not describe local validation as proof for the published
PR head unless the SHAs match or the checked files are intentionally uncommitted
documentation being prepared for that head.

## Preserved patch recovery

A preserved patch is authoritative recovery evidence when an outer workflow saved
uncommitted changes before failing. Inspect it before changing repository files,
running expensive gates for a no-op decision, or recording readiness.

The rule is to treat the preserved patch as untrusted input until inspected. The
patch review must reject absolute paths, reject `..` path traversal, reject
secrets and credentials, reject session artifacts and machine-specific files, and
modify only repository files proven intentional by the readable patch.

The patch review records the patch source, readability, affected paths, intended
change, and current-head comparison in the recovery artifact or handoff note, not
as point-in-time committed documentation.

If the preserved patch is unreadable, missing, restricted by access policy, or
otherwise cannot be inspected, the workflow output is `BLOCKED`. It is not
`MERGE_READY`, not a workflow-accepted no-op, and not evidence that the current
branch already contains the patch. Do not commit, push, post readiness, or run a
final no-op path until the patch has been inspected or the recovery requirement
has been explicitly replaced by a new source of truth.

When the patch is readable and already represented by the current branch, record
that comparison in the recovery artifact, then continue through the normal
current-head gates. When it is readable but not represented, make the smallest
repository change that applies the patch's intended behavior and rerun affected
gates.

For pyproject package metadata recovery, compare the preserved patch hunk with
the current branch before touching `pyproject.toml`. A `project.version` value
such as the one in `[project]` is not enough on its own: do not treat a matching
version value as confirmation. The workflow must reproduce only the metadata
change represented by the readable patch.

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
automation, rendering correctness, grading correctness, creative assessment,
full Tweedle/player decode unless directly proven, Save completion, lesson
completion, production readiness, deployment success, merge completion, or
manual merge.

The published PR #173 statement must include the evidence collection timestamp,
the exact PR head SHA, merge state, mergeability, and the exact required check or
check-run names with conclusions. A stale PR-body claim from an older head must
be explicitly superseded rather than reused as merge-ready evidence.

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

## Three-cycle quality audit

The readiness workflow uses three explicit SEEK / VALIDATE / FIX cycles before a
merge-ready decision. The cycles are a review method, not a separate test
framework, and their output belongs in a handoff note, review note, workflow
artifact, or explicitly authorized PR comment rather than a committed status
file.

Each cycle has this shape:

| Step | Required content |
| --- | --- |
| SEEK | The risk or claim being searched for. |
| VALIDATE | The command, diff review, metadata check, or document inspection used to prove the state for the current head. |
| FIX | The repository change made, or a no-repository-change justification when the current head already satisfies the target. |

Use these default cycles for sharing-readiness recovery:

| Cycle | SEEK | VALIDATE | FIX when clean |
| --- | --- | --- | --- |
| 1. Scope and claim accuracy | Overbroad sharing, deployment, UI, grading, creative-assessment, lesson-completion, or merge-completion claims, including claims that a preserved patch is already represented without direct patch inspection. | Review the preserved patch when provided, `docs/sharing-readiness-boundary.md`, this guide, the PR description, changed scenario assets, generated adapters, and guard tests. | No repository change required when every claim stays within classroom handoff evidence and any preserved patch has been inspected and matched to current head. |
| 2. Canonical and generated asset consistency | Drift between canonical eatme scenarios and generated Gadugi adapters. | Run `cargo run -q -p eatme-cli -- assets validate --json` and `cargo run -q -p eatme-cli -- assets generate-gadugi --check --json`. | No repository change required when assets validate and check mode reports fresh generated adapters. |
| 3. Gate completeness and final readiness | Missing runnable QA, failing or incomplete Actions, stale PR evidence, docs impact gaps, unfocused diff, dirty worktree, or a final overclaim. | Run applicable focused tests such as `cargo test -q -p eatme-assets outside_in_alice_expansion_tests`, `mkdocs build --strict`, `TMPDIR=/tmp ./scripts/quality-gates.sh`, `gh pr view`, `gh pr diff --name-only`, and local git status checks. | No repository change required only when the final cycle is clean, the worktree is clean, and all evidence points to the same head. |

If a cycle finds a defect, fix only that defect, rerun the affected validation,
and repeat the cycle. The final readiness decision cannot be `MERGE_READY` until
the third cycle is clean.

## Merge-ready decision

The workflow outcome is one of three states:

| Decision | Use when |
| --- | --- |
| `MERGE_READY` | Local branch, local `HEAD`, PR head SHA, required checks, mergeability, applicable runnable QA, docs impact review, focused diff, PR description evidence, any required preserved patch review, and three quality-audit cycles all pass for the same current head. |
| `NOT_MERGE_READY` | Any required gate is missing, failing, stale, tied to a different head, or broader than the evidence proves. |
| `BLOCKED` | Required recovery evidence, such as a preserved patch, cannot be inspected. This stops the workflow before a readiness or no-op decision. |

`MERGE_READY` does not mean the workflow merged the PR. It means the evidence is
complete enough for a separate merge mechanism or maintainer decision.

`BLOCKED` is reserved for cases where the workflow cannot inspect required
evidence. It is not a weaker readiness decision; it means the workflow did not
evaluate enough evidence to decide readiness.

`NOT_MERGE_READY` and `BLOCKED` outputs must name concrete blockers. Good
blockers are specific and actionable:

```text
NOT_MERGE_READY

Blockers:
- GitHub Actions are still pending for ${PR_HEAD_SHA}.
- The PR description cites readiness evidence from ${OLD_SHA}, not the current
  PR head.
- Cycle 2 found generated Gadugi adapter drift; regenerate adapters from the
  canonical eatme scenarios and rerun check mode.
```

Do not downgrade a missing gate into a warning. Green Actions alone are not
enough, and a skipped manual or deploy-only job is non-evidence rather than
readiness proof unless branch protection requires that job.

## No-op justification

A workflow-accepted no-op justification is not an additional decision state. It
is a `MERGE_READY` outcome with a no-op justification, accepted only when
current-head evidence, review evidence, finalization evidence, and any required
preserved patch review prove that no repository changes were required. The
output must tie current head/checks, PR head checks, preserved patch coverage
when applicable, and the current PR head to merge-ready blockers or evidence;
when that tie is present and clean, the report uses an explicit
workflow-accepted No-op justification.

The justification should include:

| Item | Required content |
| --- | --- |
| Evidence timestamp | UTC timestamp for the metadata and local evidence collection. |
| Branch and head | Local branch and `HEAD` SHA. |
| Worktree state | Clean final worktree state. |
| PR metadata | PR number, head branch, head SHA, merge state, mergeability, and exact check names with conclusions. |
| Executed gates | Commands that passed for the evaluated state, limited to gates that apply to the PR scope or close a current-head evidence gap. |
| Preserved patch coverage | Required only when recovery supplied a saved patch. State that the patch was inspected and is already represented by the current head, or do not use no-op wording. |
| Claim boundary | The exact readiness claim and explicit non-claims. |
| No-op reason | Why in-scope docs, assets, generated adapters, and tests already satisfy the contract. |
| Changed-file scope | Changed-file scope reviewed for the current PR head, with unrelated changes rejected before no-op output. |
| Blockers | Blockers list showing `none` only when every applicable gate is clean; otherwise emit `NOT_MERGE_READY` with specific blockers. |
| Audit cycles | The three SEEK / VALIDATE / FIX cycles, including the clean final cycle. |

When a saved patch is part of recovery, the output must use a literal `No-op`
only after recording the current PR head SHA, current check status, mergeability
state, and confirmation that the preserved patch is already represented. The rule
is: do not use no-op wording when the preserved patch is unreadable.

Do not emit `No-op` when local `HEAD` differs from the PR head. Do not emit
`No-op` when the final worktree is dirty. Either case must produce
`NOT_MERGE_READY` with specific blockers instead of a success-shaped no-op
justification.

For PR #173, use this final sentence only after current GitHub metadata confirms
the clean protected-flow state for the recorded head:

```text
No-op justification: PR #173 current head ${PR_HEAD_SHA} is the sole review basis; current GitHub checks/mergeability show the PR is clean/green for the protected flow; sharing readiness is limited to classroom review handoff readiness, not deployed/hosted/product readiness; no merge-ready blockers remain; no repository edits or commits were required.
```

Example no-op wording:

```text
MERGE_READY

Workflow-accepted no-op recovery recorded for PR #173 at current branch head
${HEAD_SHA}. Evidence was collected at ${EVIDENCE_COLLECTED_AT}. The local
branch matches PR head ${PR_HEAD_SHA}, the worktree is clean, and the final
GitHub metadata re-check still reports that same head. Current-head evidence
passed for the gates in scope, such as asset validation, generated Gadugi
freshness, focused readiness-guard tests, strict documentation build, quality
gates when required, review evidence, and PR metadata review.

No repository changes were required because the committed sharing-readiness docs,
scenario assets, generated adapters, and guard tests already preserve the
classroom review handoff boundary. The three SEEK / VALIDATE / FIX cycles found
no remaining defects in scope, asset consistency, generated adapters, gate
coverage, docs impact, PR evidence, or final claim boundaries. Finalization
evidence records that no manual merge was performed.

This records bounded silver-thread/e2e sharing-readiness evidence only. It does
not claim hosted sharing, deployed sharing, platform success, full UI
automation, rendering correctness, grading correctness, creative assessment,
full Tweedle/player decode unless directly proven, Save completion, lesson
completion, production readiness, deployment success, or merge completion.
```

## Readiness comment

Produce readiness only after all required gates pass for the evaluated head and
the final PR-head re-check still matches the validated SHA. The owner-free
handoff body is also the readiness comment body when posting is explicitly
authorized. It should name the evidence timestamp, exact head, merge state,
mergeability, and required check details, then avoid broader product-readiness
claims.

Create a handoff/comment body from the evidence record. Replace the PR number
for other recoveries; the PR #173 recovery uses this wording:

```bash
HEAD_SHA="$(git rev-parse HEAD)"
EVIDENCE_COLLECTED_AT="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
cat > readiness-handoff.txt <<EOF
Default-workflow recovery recorded for PR #173 at current branch head ${HEAD_SHA}.

Evidence collected at: ${EVIDENCE_COLLECTED_AT}
PR head: ${PR_HEAD_SHA}
Final PR head re-check: ${FINAL_PR_HEAD_SHA}
Merge state: ${MERGE_STATE_STATUS}
Mergeability: ${MERGEABLE}

Required checks for ${PR_HEAD_SHA}:

| Check | Conclusion | Workflow | Source |
| --- | --- | --- | --- |
| ${CHECK_NAME_1} | ${CHECK_CONCLUSION_1} | ${WORKFLOW_NAME_1} | ${DETAILS_URL_1} |
| ${CHECK_NAME_2} | ${CHECK_CONCLUSION_2} | ${WORKFLOW_NAME_2} | ${DETAILS_URL_2} |

Validated current-head gates: ${VALIDATED_GATES}; include asset validation,
generated Gadugi freshness, strict documentation build, quality gates, PR
metadata review, and bounded sharing-readiness claim review when each is in
scope or needed to close an evidence gap.

Changed-file scope: ${CHANGED_FILE_SCOPE}
Blockers: ${BLOCKER_SUMMARY}

Supersedes stale PR-body evidence: this handoff is the current-head evidence for
${PR_HEAD_SHA}; older PR-body readiness claims are context only.

Skipped or manual jobs treated as non-evidence:

| Job | Treatment |
| --- | --- |
| Deploy to GitHub Pages | Non-evidence for classroom sharing-readiness unless required branch protection makes it a required check for this head. |
| manual real Alice launch smoke | Non-evidence unless separately executed and recorded for this head. |

The recovery supports classroom sharing handoff readiness only. It does not
claim hosted sharing, deployed sharing, platform success, full UI automation,
rendering correctness, grading correctness, creative assessment, Save
completion, lesson completion, full Tweedle/player decode unless directly
proven, production readiness, deployment success, merge completion, or manual
merge.
EOF
```

By default, stop after producing the owner-free handoff output. If the workflow
explicitly authorizes posting to GitHub, post the same body with:

```bash
gh pr comment "${PR_NUMBER}" --body-file readiness-handoff.txt
```

Do not post readiness when any gate is failing, pending, stale, or tied to a
different head. Do not produce owner-free readiness output for any required gate
that is failing, pending, stale, or tied to a different head without an explicit
state separation.

## Blocker handling

If any required gate fails, do not produce readiness. Fix only the minimal issue that
caused the blocker, run the relevant validation again, and repeat current-head
verification.

| Blocker | Minimal response |
| --- | --- |
| Wrong branch | Switch to the PR branch worktree or stop recovery for the current checkout. |
| Local/PR head mismatch | State the mismatch and verify the intended head before making readiness claims. |
| Preserved patch unreadable or unavailable | Record `BLOCKED`; do not infer a no-op, commit, push, or post readiness until the patch can be inspected or a new recovery source of truth replaces it. |
| Preserved patch not represented by current head | Apply the minimal intended change from the patch, then rerun affected gates and current-head verification. |
| Failing, pending, cancelled, missing, or wrong-head checks | Fix the failing check, wait for completion, or rerun the missing check before readiness. |
| Dirty merge state | Resolve only the mergeability issue. |
| Overclaiming docs or scenario language | Edit the canonical documentation or scenario wording and rerun affected gates. |
| Stale generated adapter | Regenerate adapters from canonical sources. |
| Asset validation failure | Fix the invalid scenario or persona asset. |
| Documentation build failure | Fix the broken doc, navigation, link, or MkDocs configuration. |
| Quality gate failure | Fix the failing repository gate without bypassing it. |
| Unrelated changes | Remove the unrelated change from the readiness work. |
