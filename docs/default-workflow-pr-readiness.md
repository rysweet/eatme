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
| GitHub checks | Required checks report explicit success for the PR head SHA; skipped required checks are blockers, not green evidence. |
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

Offline JSON passed to `eatme pr-readiness finalize --evidence` must include
explicit PR `state` and `draft` fields from current GitHub metadata. Missing
fields are invalid evidence; closed, merged, or draft PR evidence must produce
`NOT_MERGE_READY`, never a no-op justification.

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
| `state` | Offline evidence must include GitHub PR `state`; readiness requires `OPEN`. |
| `draft` | Offline evidence must include whether the PR is a draft; readiness requires `false`. This may be populated from GitHub `isDraft`. |
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

5. Validate persona and canonical scenario assets when the PR touches or
   documents source asset behavior, or when GitHub check evidence for assets is
   stale, missing, red, or ambiguous. Generated Gadugi adapters are not
   canonical scenario source assets and do not satisfy this gate by themselves:

   ```bash
   cargo run -q -p eatme-cli -- assets validate --json
   ```

6. Check generated Gadugi adapter freshness when canonical scenario assets or
   generated adapter paths under `assets/scenarios/gadugi/` are in scope:

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
      --json headRefOid,mergeStateStatus,mergeable,statusCheckRollup,reviewDecision,state,isDraft,url
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
| `isDraft` | Must be `false`; draft PRs are not readiness evidence. |

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
enough. A skipped optional manual or deploy-only job is non-evidence rather than
readiness proof, and a skipped required job is a blocker because required checks
pass only on explicit success.

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

---


## PR #175 evidence contract

This page is the self-contained recovery artifact for PR #175. It records
bounded evidence for a validated PR evidence head, GitHub PR metadata observed
at evidence-capture time, and the publication-head boundary for this artifact.

This is evidence-contract finalization, not validation completion. Treat every
claim below as limited to the command, timestamp, and observed value that
supports it.

- [PR #175 evidence contract](#pr-175-evidence-contract)
- [Readiness evidence](#readiness-evidence)
- [Evidence-only recovery after no-op guard failure](#evidence-only-recovery-after-no-op-guard-failure)
- [Configuration](#configuration)
- [Review evidence](#review-evidence)
- [Starter-project evidence boundary](#starter-project-evidence-boundary)
- [Generated Gadugi adapter freshness](#generated-gadugi-adapter-freshness)
- [Save/reopen recovery output shape](#savereopen-recovery-output-shape)
- [Readiness comment](#readiness-comment)
- [Blocker handling](#blocker-handling)

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

## Evidence-only recovery after no-op guard failure

Use evidence-only recovery when an existing pull request already contains the
source, documentation, tests, contracts, and generated outputs under review, but
the wrapper workflow failed before it produced an accepted implementation
summary. Recovery does not recreate the PR and does not introduce source edits
unless exact-head verification exposes stale, missing, or overbroad artifacts.

The recovery lane has four components:

| Component | Responsibility |
| --- | --- |
| Evidence collector | Fetch `pull/<number>/head`, resolve the exact head SHA, and collect PR metadata, changed files, mergeability, reviews, and check rollup from GitHub for that same head. |
| Evidence verifier | Inspect the fetched PR ref, not a stale local branch, for docs, tests, contracts, generated outputs, and bounded readiness wording. |
| Readiness statement builder | Compose continuation/review readiness language only from verified evidence and explicit non-claims. |
| Workflow output builder | Emit either a real `Files modified` list or an explicit `No-op justification` accepted by the workflow. |

Run recovery in this order:

1. Fetch and resolve the PR head:

   ```bash
   PR_NUMBER=123 # replace with the pull request number under review
   git fetch origin "pull/${PR_NUMBER}/head:refs/remotes/origin/pr/${PR_NUMBER}" --quiet
   git rev-parse "refs/remotes/origin/pr/${PR_NUMBER}"
   ```

2. Query GitHub metadata for the same PR:

   ```bash
   gh pr view "$PR_NUMBER" \
     --json headRefOid,mergeStateStatus,mergeable,statusCheckRollup,files,commits
   gh pr checks "$PR_NUMBER" --json name,state,bucket,completedAt,link
   ```

3. Compare the fetched ref SHA to `headRefOid`. A mismatch blocks recovery for
   the old head.

4. Inspect the changed files at the fetched ref. Treat GitHub PR metadata,
   completed checks, the fetched PR ref, committed docs, Rust tests, and evidence
   contracts as the evidence sources.

5. For save/reopen work, verify the [Save/reopen Readiness](save-reopen-readiness.md)
   contract directly. Reopen readiness depends on accepted `save-project` proof
   from the same run and an explicit `reopen-project` probe or report. Do not
   infer reopen proof from starter-project preflight evidence.

6. If the fetched ref already contains the required evidence contract and wording,
   do not edit files to satisfy the wrapper. Emit a `No-op justification` instead.
   If verification finds stale or missing artifacts, make only the smallest
   documentation, test, contract, or generated-output change needed and emit
   `Files modified`.

Safe recovery wording:

```text
Ready for continuation/review based on available bounded evidence at exact PR head <sha>.

Evidence sources: GitHub PR metadata, fetched pull/<number>/head, completed checks
for that head, changed-file list, committed docs, and committed tests/contracts.

Limitations: This does not claim full Alice UI automation, grading correctness,
creative assessment, visible rendering correctness, Save completion,
first-lesson completion, or end-to-end user success.
```

When no source or documentation edits are needed, use this output shape:

```text
No-op justification: Evidence-only recovery for existing PR #<number>. The exact
PR head was refreshed, the fetched PR ref matched GitHub `headRefOid`, metadata
and current check status were reviewed for that same head, and committed
docs/tests/contracts already express the bounded starter/save-reopen readiness
boundary. No files were changed because there was no stale or missing artifact
to fix.
```

When files change, use this output shape:

```text
Files modified:
- docs/default-workflow-pr-readiness.md - Documents evidence-only recovery output
  after a no-op guard failure.
```

Do not publish recovery readiness as proof of end-to-end user success. The
strongest accepted claim is ready for continuation/review based on available
bounded evidence for the exact verified head.

## Configuration

Run commands from the repository root.

If running Node-based workflow wrappers, set the repository's large-heap Node
option before invoking the wrapper:

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

## Starter-project evidence boundary

Starter-project preflight evidence is bounded setup evidence for opening the
bundled starter project and recording reviewable launch artifacts. It is not PR
readiness, mergeability, production suitability, complete lesson execution,
full Alice UI automation, visible rendering correctness, Save/reopen/export
completion, grading, creative assessment, or complete Alice coverage.

The source contract for this boundary is split across:

- `docs/default-workflow-pr-readiness.md`
- `docs/starter-project-preflight-evidence.md`

## Executable starter-project boundary check

The current executable starter-project boundary check lives in
`crates/eatme-assets/src/starter_project_preflight_boundary_tests.rs`. It reads
this contract table and applies the same overclaim rules to the canonical
scenario text, generated Gadugi adapter output, and scoped starter-project
preflight evidence documentation.

| Prohibited phrase | Bounded replacement |
| --- | --- |
| `PR ready` | `starter-project preflight evidence recorded` |
| `merge ready` | `starter-project evidence boundary satisfied` |
| `production ready` | `bounded preflight evidence available for review` |
| `ready for merge` | `readiness gaps are documented for later gates` |
| `readiness guaranteed` | `readiness depends on the separate readiness gates` |
| `complete PR readiness` | `starter-project preflight evidence only` |
| `proves visible rendering correctness` | `screenshot or window evidence is observation evidence only` |
| `proves save/reopen/export` | `save, reopen, and export remain readiness gaps` |
| `first lesson is complete` | `starter-project preflight evidence only` |
| `grades learner work` | `records evidence for review; it does not grade` |
| `assesses creativity` | `names an editable change without assessing creativity` |

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

## Save/reopen recovery output shape

This subsection is the reusable output contract for save/reopen recovery after a
rate-limit failure, wrapper no-op guard failure, or other workflow failure that
prevented a useful final evidence summary. It does not create a new PR and does
not broaden the save/reopen evidence boundary. Put the exact PR number, branch,
and head SHA in the PR comment or workflow summary produced for that recovery;
do not bake point-in-time branch or SHA evidence into this durable document.

The recovery output must include this evidence shape:

```text
Default-workflow recovery recorded for PR #<number> at exact head <exact-head-sha>.

Evidence inspected:
- GitHub PR metadata for PR #<number> at the same head.
- Fetched pull/<number>/head, or the local branch when it matches GitHub
  `headRefOid`.
- `crates/eatme-alice/src/launch_save_project.rs` save proof conditions.
- `crates/eatme-alice/src/launch_reopen_project.rs` reopen proof conditions.
- `crates/eatme-alice/src/compare/ui_action_contract.rs` and
  `crates/eatme-alice/src/compare/ui_action_contract/save.rs` action-contract
  wiring for passed proof versus no-go states.
- `docs/save-reopen-readiness.md` and
  `docs/starter-project-preflight-evidence.md` bounded evidence wording.

Files modified:
- docs/save-reopen-readiness.md - Adds the save/reopen PR review evidence shape
  and finalization wording.
- docs/default-workflow-pr-readiness.md - Adds the save/reopen recovery example and
  required output boundary.

Checks run:
- List only commands actually executed for this finalization.

Limitations:
- No full Alice UI automation claim.
- No grading validation claim.
- No creative-assessment validation claim.
- No full Save completion claim.
- No first-lesson completion claim.
- No export completion claim.
- No broad product-readiness claim.
```

When save/reopen finalization changes no files, replace `Files modified` with:

```text
No-op justification: Evidence-only recovery for existing PR #<number>. The exact
head <exact-head-sha> was verified against branch <branch-name>, the fetched PR
ref matched GitHub `headRefOid`, current check status was reviewed for that same
head, and the committed save/reopen docs, tests, and action-contract code already
expressed the bounded starter/save-reopen readiness boundary. No files were
changed because no stale, missing, or overbroad artifact was found.
```

The strongest accepted conclusion is that the PR has bounded save/reopen
evidence suitable for continuation or review at the exact verified head. Do not
rewrite that conclusion as Alice UI automation success, grading success,
creative-assessment success, or product readiness.

## Readiness comment

Publish readiness only after all required gates pass for the exact head. The
comment should name the head and avoid broader product-readiness claims.

Example:

```text
Default-workflow readiness recorded for PR #<number> at exact head <exact-head-sha>.

Verified gates: exact PR head, green GitHub checks for that head, mergeStateStatus=CLEAN, mergeable=MERGEABLE, bounded starter-project preflight wording, no unsupported claims for first-lesson completion/grading/creative assessment/full UI automation/visible rendering correctness/full Save completion, generated Gadugi adapter freshness, and asset validation.

The prior non-zero wrapper exit is not treated as a blocker because direct verification passed at this exact head.
```

Post the comment with:

```bash
gh pr comment "$PR_NUMBER" --body-file readiness-comment.txt
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

