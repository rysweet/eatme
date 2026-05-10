# Default-workflow PR readiness

Default-workflow PR readiness is the exact-head recovery gate for pull requests
that need an owner-free, no-timeout merge-ready decision after an uncertain
workflow exit.

Use this page to recover a pull request through the no-timeout default-workflow
readiness path. The workflow evaluates the published PR branch head as the
source of truth, collects bounded evidence for that exact commit, and emits
`MERGE_READY` only when every implemented gate passes. Green checks and a
completed workflow are necessary evidence, but they are not sufficient by
themselves.

## Contents

- [Readiness contract](#readiness-contract)
- [Feature component mapping](#feature-component-mapping)
- [Configuration](#configuration)
- [Exact-head setup](#exact-head-setup)
- [GitHub evidence](#github-evidence)
- [PR-state review](#pr-state-review)
- [Local QA evidence](#local-qa-evidence)
- [Scenario evidence review](#scenario-evidence-review)
- [Run/observe readiness scope audit](#runobserve-readiness-scope-audit)
- [Documentation impact review](#documentation-impact-review)
- [Focused diff review](#focused-diff-review)
- [PR description evidence](#pr-description-evidence)
- [Quality-audit cycles](#quality-audit-cycles)
- [Decision gate](#decision-gate)
- [Verdicts](#verdicts)
- [No-op recovery output](#no-op-recovery-output)
- [PR #171 run/observe recovery profile](#pr-171-runobserve-recovery-profile)
- [Readiness comment](#readiness-comment)
- [Blocker handling](#blocker-handling)
- [Implementation consistency](#implementation-consistency)
- [Bounded evidence language](#bounded-evidence-language)
- [Troubleshooting](#troubleshooting)
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

## Readiness contract

A pull request receives a default-workflow `MERGE_READY` verdict only when every
implemented gate passes for the exact commit being evaluated.

| Evidence area | Required result |
| --- | --- |
| Exact remote head | The local `HEAD`, `origin/<branch>`, and PR `headRefOid` all name the same commit. |
| Published branch source of truth | The workflow uses the current remote PR branch head. It does not manually merge, rebase, force-push, or rewrite PR history. |
| No timeout wrappers | Validation commands run directly. The workflow does not use shell `timeout` wrappers or retry wrappers that hide the real exit status. |
| Workflow completion | The default-workflow run has completed for the evaluated head. Owner-free exits classified as `FAILED_OR_UNKNOWN` are recovered only by collecting fresh readiness evidence. |
| GitHub Actions | Check rollup entries treated as required by this gate are completed and green for the evaluated head. Pending, missing, failing, stale, or wrong-head required checks block readiness. |
| PR-state review | Draft PRs, blocking labels, requested changes, and stale decisive review evidence block readiness. Owner-free `REVIEW_REQUIRED` remains acceptable when every other gate is clear. |
| Runnable QA | Existing repository validation commands pass for the evaluated head. Missing or unsupported required tooling blocks readiness instead of being replaced with weaker evidence. |
| Scenario evidence | Applicable scenario evidence is runnable and bounded to what the evidence directly proves. |
| Run/observe scope | Recovery decisions stay scoped to the run/observe readiness gap and do not expand into unrelated docs, assets, workflows, refactors, or feature work. |
| Docs impact | Documentation changes and readiness claims match the implementation scope. Missing docs or overbroad docs block readiness. |
| Focused diff | Changed files match the PR purpose and do not widen scope with unrelated or stale changes. |
| PR description | The PR body names the evaluated head, final verdict, validation evidence, docs impact, focused scope, and remaining blockers or non-claims. It may link to a detailed readiness comment, but a stale PR body still blocks readiness. |
| Quality audit | At least three `SEEK / VALIDATE / FIX` cycles are documented, and the final cycle is clean. |

If any implemented gate is missing, stale, inconsistent, or unavailable, the
workflow emits `NOT_MERGE_READY` with explicit blockers.

## Feature component mapping

The implementation keeps the feature as small, explicit components that map
directly to the readiness gates:

| Component | Doc sections | Responsibility |
| --- | --- | --- |
| `ExactHeadVerifier` | [Exact-head setup](#exact-head-setup), [GitHub evidence](#github-evidence) | Fetch the published PR branch and verify local `HEAD`, `origin/<branch>`, and PR `headRefOid` all equal the evaluated head before any evidence is accepted. |
| `PrMetadataCollector` | [GitHub evidence](#github-evidence), [PR-state review](#pr-state-review), [Troubleshooting](#troubleshooting) | Collect PR number, title, body, head branch, head SHA, mergeability, merge state, draft status, labels, review decision, latest reviews, check rollup, and changed files through `gh pr view` with typed response parsing. |
| `LocalQARunner` | [Local QA evidence](#local-qa-evidence), [Troubleshooting](#troubleshooting) | Run existing repository validation commands directly, without timeout wrappers or substituted commands. |
| `ReadinessScopeAuditor` | [Scenario evidence review](#scenario-evidence-review), [Run/observe readiness scope audit](#runobserve-readiness-scope-audit), [Documentation impact review](#documentation-impact-review) | Design-level concept (distributed across the `reviews/` modules in code) that inspects docs, scenario assets, generated Gadugi adapters, and default-workflow readiness tests for concrete run/observe readiness gaps. |
| `DocsImpactReviewer` | [Documentation impact review](#documentation-impact-review), [Bounded evidence language](#bounded-evidence-language) | Confirm documentation matches the implementation scope and does not contain stale status, progress, or overbroad readiness claims. |
| `FocusedDiffReviewer` | [Focused diff review](#focused-diff-review) | Confirm changed files belong to the PR purpose and generated assets are fresh when canonical assets change. |
| `QualityAuditRunner` | [Quality-audit cycles](#quality-audit-cycles) | Document at least three `SEEK / VALIDATE / FIX` cycles and require the final cycle to be clean. |
| `DecisionGate` | [Focused diff review](#focused-diff-review), [Decision gate](#decision-gate), [Verdicts](#verdicts), [No-op recovery output](#no-op-recovery-output) | Choose a literal no-op when evidence is current and complete, or require the smallest focused readiness patch when a real run/observe readiness gap exists. |
| `ChangeReporter` | [PR description evidence](#pr-description-evidence), [Verdicts](#verdicts), [No-op recovery output](#no-op-recovery-output) | Publish the final `MERGE_READY` or `NOT_MERGE_READY` record, including files modified, literal no-op justification when applicable, and explicit blockers. |

## Configuration

Run all commands from the repository root.

For Node-based workflow wrappers, use the saved large-heap preference:

```bash
export NODE_OPTIONS=--max-old-space-size=32768
```

This setting is not a repository readiness gate for Rust, MkDocs, or shell
validation commands; it only prevents Node wrapper memory limits from becoming
noise while orchestration runs.

Use `/tmp` for repository quality gates in deep worktrees:

```bash
TMPDIR=/tmp ./scripts/quality-gates.sh
```

Use authenticated `gh` access for pull request and check metadata. Do not print
or store GitHub tokens, credential-helper output, local secret paths, or raw
environment dumps in readiness evidence.

## Exact-head setup

Fetch the published PR branch and inspect the exact remote head:

```bash
git fetch origin wave6-scenario-run-observe-gap-1778302300
git rev-parse origin/wave6-scenario-run-observe-gap-1778302300
```

Use an existing local tracking branch when it already points at the published PR
branch:

```bash
git switch wave6-scenario-run-observe-gap-1778302300
git pull --ff-only origin wave6-scenario-run-observe-gap-1778302300
```

If no local branch exists, create one from the remote branch:

```bash
git switch --track -c wave6-scenario-run-observe-gap-1778302300 \
  origin/wave6-scenario-run-observe-gap-1778302300
```

Verify the evaluated commit:
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
git rev-parse HEAD
git rev-parse origin/wave6-scenario-run-observe-gap-1778302300
gh pr view 171 --json headRefName,headRefOid
```

Readiness is blocked unless all three values identify the same PR head. Do not
merge `origin/master`, rebase the branch, or manually merge PR content during
this recovery workflow.

## GitHub evidence

Collect the pull request metadata enforced by the v1 automated gate:

```bash
gh pr view 171 \
  --json number,title,body,headRefName,headRefOid,mergeStateStatus,mergeable,isDraft,labels,reviewDecision,latestReviews,statusCheckRollup,files
```

The GitHub evidence passes only when:

- `headRefName` is the expected PR branch.
- `headRefOid` matches local `HEAD`.
- `mergeStateStatus` is `CLEAN` or `HAS_HOOKS` (both satisfy repository branch protection).
- `mergeable` does not report an unmergeable state.
- the PR is not a draft.
- no configured blocking label is present.
- `reviewDecision` does not request changes.
- decisive `latestReviews` evidence applies to `headRefOid`.
- every check treated as required by the gate has completed successfully for
  `headRefOid`.

Treat a green check summary as one input to the readiness gate, not as the final
readiness decision.

The v1 source of truth for required checks is `statusCheckRollup`: skipped
entries are treated as optional, and every non-skipped entry is treated as
required. If repository branch-protection APIs or explicit required-check
configuration become the source of truth later, update the service, gate, tests,
and this page together.

The GitHub service adapter must surface command failures, unavailable `gh`
access, and malformed JSON as readiness blockers instead of replacing missing
service evidence with success-shaped defaults. Retry policy is bounded and does
not apply to local QA commands, which must still run directly without timeout or
retry wrappers.

## PR-state review

Draft state, labels, review decisions, and latest review evidence are automated
readiness gates. The service collects them from the same `gh pr view` response as
the head and check evidence so the gate does not infer review state from green
checks alone.

The PR-state gate blocks readiness when:

- `isDraft` is true;
- a blocking label such as `do-not-merge`, `blocked`, `wip`, or `hold` is present;
- `reviewDecision` is `CHANGES_REQUESTED`;
- any latest review requests changes;
- a decisive latest review approval or change request belongs to a commit other
  than the evaluated head.

Owner-free `REVIEW_REQUIRED` is not a blocker by itself. It means no approval is
being claimed; readiness still depends on exact head, checks, local QA,
scenario/docs/scope/diff evidence, current PR body, and clean audit cycles. Do
not infer owner approval from green checks, a clean branch, or a successful local
QA run.

## Local QA evidence

Run existing repository validation commands directly, without shell timeout
wrappers:

```bash
cargo run -q -p eatme-cli -- assets validate --json
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
mkdocs build --strict
TMPDIR=/tmp ./scripts/quality-gates.sh
```

The local QA gate passes only when every required command exits successfully for
the evaluated head. If a required command cannot run because local tooling is
missing or unsupported, emit `NOT_MERGE_READY` with that command as a blocker.
Do not substitute an unrelated command or downgrade the requirement to
"unavailable evidence."

When canonical scenario assets change, generated Gadugi adapters must be fresh.
If check mode reports stale or missing output, regenerate adapters from the
canonical assets and run check mode again:

```bash
cargo run -q -p eatme-cli -- assets generate-gadugi --json
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
```

## Scenario evidence review

Scenario evidence is runnable only when the PR exposes a concrete command,
manifest, generated adapter, report, or asset validation path that reviewers can
repeat for the evaluated head.

Reviewers accept only bounded scenario claims:

| Evidence | Accepted claim |
| --- | --- |
| Asset validation JSON | Persona and scenario assets satisfy repository schema and contract checks. |
| Gadugi check output | Generated adapters match canonical scenario assets. |
| Launch-smoke manifest or report | The named run recorded the launch, window, screenshot, log, or report evidence stated by that artifact. |
| Run/observe readiness report | The selected scenario and run show only the listed `shown` evidence and retain all `not yet shown` gaps. |

Do not upgrade scenario evidence into claims of full UI automation, visible
rendering correctness, grading, creative assessment, full lesson completion,
full world execution, Save completion, deployed sharing/platform success, or
full Tweedle/player decode unless that exact behavior is directly proven by
runnable evidence.

Explicit missing Run-window and observe-state evidence remains a readiness gap
until a runnable artifact proves the missing state for the evaluated head.

## Run/observe readiness scope audit

The scope audit keeps owner-free recovery focused on the run/observe readiness
gap. It reviews only the surfaces that can prove, document, or accidentally
overstate that gap:

| Surface | Accepted scope |
| --- | --- |
| `docs/run-observe-readiness.md` | Usage, JSON, generated-adapter, validation, and no-op guard documentation for bounded Run/observe evidence. |
| `docs/default-workflow-pr-readiness.md` | Exact-head recovery, PR-state boundaries, focused diff classification, no-op output, and decision gate documentation. |
| `assets/scenarios/eatme/` | Canonical scenario wording for run dispatch, observe-state evidence, gap files, and unproven claims. |
| `assets/scenarios/gadugi/` | Generated adapters that mirror canonical run/observe readiness scenarios. |
| `crates/eatme-core/src/default_workflow_pr_readiness/` | Readiness gate implementation for exact-head, PR metadata, checks, docs, scope, PR description, quality-audit, and no-op decisions. |
| `crates/eatme-core/tests/default_workflow_pr_readiness*.rs` | Tests that lock the readiness gate and service parsing behavior. |
| `crates/eatme-assets/src/*run_observe*` and generated-asset tests | Tests that keep canonical scenario and generated adapter behavior aligned. |

Files outside these surfaces are allowed only when they are the smallest
required support for the run/observe readiness gap. Broad refactors, unrelated
docs, unrelated assets, workflow permission changes, branch protection changes,
review manipulation, and feature expansion are out of scope.

## Documentation impact review

Review documentation whenever the PR changes user-facing behavior, scenario
wording, validation commands, generated adapter behavior, evidence artifacts, or
readiness boundaries.

The docs impact review passes when:

- changed behavior is documented in `docs/`;
- existing docs do not contradict the new behavior;
- examples use existing repository commands and realistic paths;
- readiness wording stays within the same evidence boundaries as assets, tests,
  and generated adapters;
- temporal status, one-off logs, and progress reports remain in PR comments or
  CI logs instead of permanent docs.

If no documentation change is required, the readiness evidence states why the
diff does not affect documented behavior.

## Focused diff review

Review the PR file list against the PR purpose. The diff is focused when every
file belongs to one of these categories:

| Category | Examples |
| --- | --- |
| Canonical assets | `assets/scenarios/eatme/*.yaml`, persona assets |
| Generated assets | `assets/scenarios/gadugi/*.yaml` generated from canonical sources |
| Tests | Rust tests that enforce the changed scenario or evidence contract |
| Documentation | `docs/*.md`, `mkdocs.yml` navigation for changed docs |
| Tooling glue | Minimal command or workflow glue required by the readiness feature |

Unrelated formatting churn, broad rewrites, stale generated files, unrelated
feature work, or hidden manual fallback artifacts block readiness until removed
or explicitly justified by the PR scope.

For PR #171, the focused diff reviewer uses a narrower rule: every changed file
must be directly tied to run/observe readiness evidence, exact-head recovery, PR
metadata collection, generated adapter freshness, or documentation of
those same boundaries. A clean CI result does not make an unrelated file focused.

## PR description evidence

The PR body is the durable readiness record. It may link to a detailed readiness
comment, but the body itself must name the evaluated head, final verdict, and
evidence summary. A linked readiness comment is acceptable only when the PR body
is current and clearly points reviewers to the detailed evidence.

```text
Default-workflow recovery for PR #171

Evaluated head: <head-sha>
Branch: wave6-scenario-run-observe-gap-1778302300
Workflow exit recovered from: owner-free FAILED_OR_UNKNOWN
Final verdict: <MERGE_READY | NOT_MERGE_READY>

GitHub Actions: <all gate-required checks completed and green for head-sha>
Local QA:
- cargo run -q -p eatme-cli -- assets validate --json: <pass>
- cargo run -q -p eatme-cli -- assets generate-gadugi --check --json: <pass>
- mkdocs build --strict: <pass>
- TMPDIR=/tmp ./scripts/quality-gates.sh: <pass>

Scenario evidence: <bounded runnable evidence or NOT_MERGE_READY blocker>
Run/observe scope: <focused / blocker>
Docs impact: <documented / no docs impact with reason / blocker>
Focused diff: <focused / blocker>
PR state: <not draft; labels/reviews non-blocking or blocker>
Quality audit cycles: <three cycles documented; final clean>
Files modified: <list of repository files changed by recovery, or none>
No-op justification: <required when Files modified is none>

Evidence boundary: no claim of full UI automation, visible rendering
correctness, grading, creative assessment, full lesson completion, full world
execution, Save completion, deployed sharing/platform success, or full
Tweedle/player decode unless directly proven above.
```

If the PR body is stale, update it. If it cannot be updated, emit
`NOT_MERGE_READY` with a specific stale-description blocker even when a separate
readiness comment exists.

## Quality-audit cycles

Run at least three quality-audit cycles before declaring readiness. Each cycle
uses the same structure:

| Step | Meaning |
| --- | --- |
| `SEEK` | Identify a concrete readiness risk. |
| `VALIDATE` | Check that risk with command output, PR metadata, file review, or docs evidence. |
| `FIX` | Fix the issue, record why no repository change is required, or list the remaining blocker. |

The standard recovery cycles are:

| Cycle | SEEK | VALIDATE | FIX |
| --- | --- | --- | --- |
| 1. Exact head and checks | Wrong commit, stale checks, or incomplete workflow evidence. | Compare local `HEAD`, `origin/<branch>`, PR `headRefOid`, and GitHub check rollup. | Resync to the remote PR head, wait for or rerun checks, or block readiness. |
| 2. Runnable QA and docs | Local validation, scenario evidence, or docs impact is missing. | Run the repository validation commands and review applicable scenario/docs surfaces. | Fix stale assets/docs/tests, or record missing evidence as `NOT_MERGE_READY`. |
| 3. Scope and bounded claims | Diff, PR body, or readiness text overclaims or includes unrelated work. | Review changed files, PR body, generated adapters, and evidence wording. | Remove overclaims/unrelated changes, update PR evidence, or block readiness. |

The final cycle must be clean: `SEEK` finds no unresolved readiness risk,
`VALIDATE` confirms all gates are satisfied for the evaluated head, and `FIX`
records no repository change or blocker is needed. If the final cycle is not
clean, emit `NOT_MERGE_READY`.

## Decision gate

The decision gate runs after exact-head, GitHub, PR-state, local QA, scenario,
docs, scope, focused diff, PR description, and quality-audit evidence have been
collected for the evaluated head.

| Evidence result | Decision |
| --- | --- |
| All gates pass and no repository file requires a change | Emit `MERGE_READY` with a literal no-op justification. |
| A real run/observe readiness gap exists and can be fixed narrowly | Patch only the directly related file, run the narrowest relevant existing validation, push the focused fix, then recollect exact-head evidence for the new head. |
| Any required evidence is missing, stale, wrong-head, blocked, or out of scope | Emit `NOT_MERGE_READY` with explicit blockers. |

The gate never merges the pull request, approves reviews, dismisses reviews,
changes branch protection, alters workflow permissions, rewrites history, or
uses manual fallback logs as readiness evidence.

## Verdicts

Use one of two final statuses.

`MERGE_READY` is valid only when every gate passes:

```text
MERGE_READY

Evaluated head: <head-sha>
Branch: <branch>
Checks: green and complete for <head-sha>
Local QA: pass
Scenario evidence: bounded runnable evidence reviewed
Run/observe scope: focused
Docs impact: reviewed
Focused diff: reviewed
PR state: reviewed
PR description: current
Quality audit: three cycles documented; final cycle clean
Files modified: <list, or none>
```

`NOT_MERGE_READY` is required when any criterion is missing:

```text
NOT_MERGE_READY

Evaluated head: <head-sha>
Blockers:
- <missing green check / missing local QA / stale PR body / unfocused diff / unresolved audit risk>

Files modified: <list, or none>
```

Do not soften blockers with phrases such as "probably ready" or "ready except."
If a required criterion is absent, the final status is `NOT_MERGE_READY`.

## No-op recovery output

A no-op recovery is acceptable only when no repository file changes are required
and the workflow evidence proves that the current head already satisfies the
readiness contract.

The implementation formats a no-op as one line. Put the concrete rationale
inside the justification text rather than relying on a separate structured block:

```text
Workflow-accepted no-op justification: <concise rationale covering exact head, gate-required green checks, local QA, bounded scenario/docs/scope/diff evidence, current PR body, clean audit cycles, and why no repository file changed>
```

If any item is unavailable or blocked, the no-op output must say
`NOT_MERGE_READY` instead of claiming workflow acceptance.

## PR #171 run/observe recovery profile

PR #171 uses this reusable profile template. The branch name and PR number below
are specific to this PR — adapt them when reusing for a different recovery.
Record the exact evaluated SHA in the PR body or readiness comment, not as a
permanent hard-coded value here:

```text
PR: 171
Branch: wave6-scenario-run-observe-gap-1778302300
Recovery trigger: owner-free default-workflow exit classified as FAILED_OR_UNKNOWN
Evaluation target: <evaluated-head-sha>
Base verification: git fetch origin wave6-scenario-run-observe-gap-1778302300
Head check: local HEAD, origin/<branch>, and PR headRefOid are identical
Allowed scope: run-observe readiness gap only
Accepted history shape: published PR branch head only; no manual base merge,
rebase, force-push, or rewritten history is readiness evidence
```

The PR #171 recovery workflow:

1. Fetches `origin/wave6-scenario-run-observe-gap-1778302300`.
2. Verifies local `HEAD`, the remote branch head, and PR `headRefOid` match.
3. Collects GitHub Actions and PR metadata for that exact head.
4. Runs asset validation, Gadugi adapter check mode, docs strict build, and
   repository quality gates without timeout wrappers.
5. Reviews runnable scenario evidence without overclaiming UI, grading, visual,
   lesson-completion, Save, deployed-sharing, or Tweedle/player behavior.
6. Audits the run/observe readiness scope across docs, scenario assets, generated
   Gadugi adapters, and default-workflow readiness code/tests.
7. Reviews documentation impact, focused diff scope, and PR description evidence.
8. Documents three quality-audit cycles and requires the final cycle to be clean.
9. Emits `MERGE_READY` only when every gate passes; otherwise emits
   `NOT_MERGE_READY` with explicit blockers.

This recovery profile does not merge PR content manually during recovery and
does not use `origin/master` ancestry or merging as readiness evidence. Do not
rebase the branch for this recovery profile.

## Readiness comment

Use a readiness comment only as detailed evidence linked from a current PR body:

```text
Default-workflow readiness recorded for PR #171 at exact head <final-head-sha>.

Exact head evidence:
- `git rev-parse HEAD`: <final-head-sha>
- `gh pr view 171 --json headRefOid,mergeStateStatus,mergeable,isDraft,labels,reviewDecision,latestReviews,statusCheckRollup`: <head/check/PR-state evidence>

Command evidence:
- `cargo run -q -p eatme-cli -- assets generate-gadugi --check --json`: <pass>
- `cargo run -q -p eatme-cli -- assets validate --json`: <pass>
- `mkdocs build --strict`: <pass>
- `TMPDIR=/tmp ./scripts/quality-gates.sh`: <pass>

Scenario and docs evidence:
- Scenario evidence: <bounded runnable evidence or blocker>
- Docs impact: <review result or blocker>
- Focused diff: <review result or blocker>
- PR state: <not draft; no blocking labels or requested changes; decisive reviews match head>
- Quality audit: <three SEEK / VALIDATE / FIX cycles; final cycle clean>
```

Gate summaries are not enough without the concrete command results.

## Blocker handling

Invalid manual fallback evidence is a blocker. Discard it from the readiness decision
and keep the final status `NOT_MERGE_READY` until fresh exact-head evidence
satisfies every gate.

Prior manual fallback logs are not readiness evidence.

## Starter-project evidence boundary

Starter-project preflight evidence is bounded setup evidence for opening the
bundled starter project and recording reviewable launch artifacts. It is not PR
readiness, mergeability, production suitability, complete lesson execution,
full Alice UI automation, visible rendering correctness, Save/reopen/export
completion, grading, creative assessment, or complete Alice coverage.

The source contract for this boundary is split across:

- `docs/default-workflow-pr-readiness.md`
- `docs/starter-project-preflight-evidence.md`

The wording must stay plain and bounded. It may say that the scenario records
real Alice launch/opened-project evidence for the bundled starter project, an
editable starter-world change note, an attempted run or observation, and
readiness-gap notes. It must keep missing Run-window and observe-state evidence
visible when either state is not shown.

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

## Implementation consistency

The implementation, tests, docs, and PR evidence must keep the same bounded
language. In particular, run/observe recovery does not prove first-lesson
completion, full UI behavior, grading, Save behavior, sharing/platform success,
or full Tweedle/player decode unless direct runnable evidence proves that exact
claim for the evaluated head.

## Bounded evidence language

Use conservative wording that names the exact evidence:

| Say | Avoid |
| --- | --- |
| `Asset validation passed for the evaluated head.` | `The lesson is fully complete.` |
| `Generated Gadugi adapters are fresh for canonical scenarios.` | `The player fully decoded the world.` |
| `Run/observe evidence lists these shown states and these gaps.` | `The UI automation is complete.` |
| `Screenshot evidence was recorded as an observation artifact.` | `Visible rendering is correct.` |
| `The readiness report keeps Save completion unproven.` | `Save succeeded.` |
| `The PR is NOT_MERGE_READY because checks are missing.` | `Checks are probably fine.` |

Readiness evidence should be concise enough for reviewers, but every claim must
trace back to a command, PR metadata field, artifact, or audited file.

## Troubleshooting

| Problem | Response |
| --- | --- |
| Local `HEAD` differs from PR `headRefOid` | Stop and resync to the published remote PR branch head before collecting evidence. |
| Checks are green for an older SHA | Treat them as stale; wait for checks on the evaluated head. |
| A validation command hangs or is slow | Let the command run normally or stop it explicitly; do not add a shell timeout wrapper and call the result readiness evidence. |
| Required tooling is missing | Emit `NOT_MERGE_READY` with the missing command or setup requirement as a blocker unless repository setup documentation provides an existing supported installation path that can be completed before the verdict. |
| Scenario evidence is not runnable | Emit `NOT_MERGE_READY` with a runnable-evidence blocker. |
| PR body omits evidence | Update the PR body, optionally linking to a detailed readiness comment; otherwise block readiness as stale PR evidence. |
| Final audit cycle finds an issue | Fix and rerun the relevant evidence collection, then document a new clean final cycle. |
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

