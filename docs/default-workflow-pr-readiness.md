# Default workflow PR readiness

Default workflow PR readiness is the recovery contract for a branch that must be
brought current, validated, and handed off through a pull request without
overstating Alice evidence. It is a documentation and PR-readiness workflow: it
ties validation evidence to the final Git commit and keeps launch-smoke readiness
bounded to the evidence the repository actually inspects.

Use this workflow for any recovery branch or readiness PR that updates Alice
readiness docs, scenario assets, generated adapters, readiness tests, or a
worktree-root no-op guard.

## Contents

- [What this workflow proves](#what-this-workflow-proves)
- [Worktree-root guard contract](#worktree-root-guard-contract)
- [Recovery usage](#recovery-usage)
- [Authoritative quality gate](#authoritative-quality-gate)
- [Documentation strict build](#documentation-strict-build)
- [Exact-head evidence](#exact-head-evidence)
- [PR update contract](#pr-update-contract)
- [API and output boundaries](#api-and-output-boundaries)

## What this workflow proves

The workflow proves only that the final branch `HEAD` has:

- been updated from current `master` through a clean merge or rebase;
- resolved only task-scoped conflicts in readiness docs, scenario assets,
  generated Gadugi adapters, readiness tests, or the no-op guard;
- passed the authoritative repository quality gate from the actual Git worktree
  root; and
- preserved bounded real Alice launch-smoke readiness wording.

It keeps the canonical non-claims visible:

```text
First-lesson completion is not proven.
Full world execution is not proven.
Grading is not proven.
Creative assessment is not proven.
Full Alice UI automation is not proven.
Visible rendering correctness is not proven.
Save completion is not proven.
Deployed sharing/platform success is not proven.
```

## Worktree-root guard contract

Every no-op or TDD guard in this workflow resolves the active repository root at
runtime:

```bash
repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"
```

If `git rev-parse --show-toplevel` fails, the guard fails clearly and exits
non-zero. It must not treat a non-Git directory as a clean no-op, and it must not
use a hard-coded linked-worktree path from an earlier recovery run.

All Git checks that decide whether a change exists run from that resolved root:

```bash
git diff --quiet
git diff --cached --quiet
test -z "$(git status --porcelain)"
```

The guard may report "no task-scoped changes" only after those commands have run
inside the active Git worktree and `git status --porcelain` is empty. Untracked
files are still changes. A stale path, missing `.git` link, detached session
directory, or non-empty porcelain status is a guard failure, not success.

## Recovery usage

Start from the actual repository root:

```bash
repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"
export NODE_OPTIONS=--max-old-space-size=32768
```

Record the branch, current `HEAD`, and dirty status before changing anything:

```bash
git branch --show-current
git rev-parse HEAD
git status --short
```

Preserve unrelated dirty files. They are not part of the readiness claim unless
they directly conflict with task-scoped recovery files.

Use the existing recovery branch:

```bash
git switch <recovery-branch>
git fetch origin master
git merge origin/master
```

Prefer merge when it preserves the branch's recovery history and avoids risky
conflict churn. Use rebase only when it is clearly the lower-risk clean path.

Resolve conflicts only in task-scoped files:

- readiness documentation;
- canonical scenario YAML under `assets/scenarios/eatme/`;
- generated Gadugi adapters under `assets/scenarios/gadugi/`;
- readiness/reporting Rust tests or code directly touched by the merge; and
- the no-op/TDD guard that resolves the Git worktree root.

Do not refactor unrelated code, rewrite unrelated history, or broaden the
readiness claim while resolving conflicts.

## Authoritative quality gate

`scripts/quality-gates.sh` is the authoritative local validation entrypoint for
default-workflow recovery. It owns the combined fmt, clippy, test, module-size,
and coverage/quality expectations:

```bash
export NODE_OPTIONS=--max-old-space-size=32768
TMPDIR=/tmp ./scripts/quality-gates.sh
```

Use `TMPDIR=/tmp` in deep linked worktrees so test sockets and temporary paths do
not fail because of checkout path length. Do not weaken, bypass, or replace this
script when producing readiness evidence.

The gate covers:

| Check | Evidence boundary |
| --- | --- |
| `cargo fmt --check` | Rust formatting is current. |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | Workspace clippy warnings are repaired instead of suppressed through broad allowances. |
| `cargo test --workspace --all-features` | Existing Rust tests pass without requiring real Alice UI automation. |
| Rust module-size check | Source modules under `crates/` stay within the repository line-count contract. |
| Coverage/quality check | The repository coverage/quality gate passes at the same commit. |

Targeted commands are useful for diagnosis, but they are not a substitute for the
authoritative gate. Run targeted fmt, clippy, tests, asset validation, or Gadugi
freshness checks only to locate and repair failures before rerunning the full
gate.

## Documentation strict build

`mkdocs build --strict` is the authoritative documentation check. It is a
separate docs-site validation command, not part of `scripts/quality-gates.sh`.
Run it when readiness documentation changes:

```bash
mkdocs build --strict
```

Do not describe a passing `scripts/quality-gates.sh` run as docs-site evidence.
When both Rust readiness and documentation changed, record both commands against
the same final commit SHA.

## Exact-head evidence

Capture the final SHA only after the merge, conflict resolution, generated
outputs, documentation updates, and any recovery commits are complete and the
worktree is clean:

```bash
test -z "$(git status --porcelain)"
final_head="$(git rev-parse HEAD)"
printf 'Final HEAD: %s\n' "$final_head"
```

Run validation on that exact `HEAD`. If a command is rerun after another commit,
replace the old SHA and old command results with the new exact-head evidence.

Required validation evidence:

```bash
export NODE_OPTIONS=--max-old-space-size=32768
TMPDIR=/tmp ./scripts/quality-gates.sh
```

If readiness documentation changed, also run the docs strict build from the same
final `HEAD`:

```bash
mkdocs build --strict
```

If canonical scenario assets changed, also verify the source assets and generated
adapters from the same final `HEAD`:

```bash
cargo run -q -p eatme-cli -- assets validate --json
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
```

Do not hand-edit generated Gadugi assets. Regenerate them through the existing
generator when canonical scenario assets change, then rerun the authoritative
quality gate.

Document unrelated pre-existing blockers plainly. Do not turn a blocked, skipped,
or failed command into readiness evidence.

## PR update contract

Update the existing PR for the recovery branch when the working tree is clean.
Create a new PR only when no suitable PR exists for that branch.

The PR body includes:

- final `HEAD` SHA;
- the authoritative quality-gate command that passed on that SHA;
- docs strict-build result when readiness documentation changed;
- asset-validation and generated-adapter freshness results when canonical
  scenario assets changed;
- bounded real Alice launch-smoke/readiness claim; and
- explicit non-claims.

Safe bounded claim:

```text
This PR preserves bounded real Alice launch-smoke/readiness evidence for the
final HEAD only. It reports repository quality-gate, docs strict-build,
asset/readiness, and launch-smoke wording evidence only.
```

Required non-claims:

```text
Canonical non-claims:
- First-lesson completion is not proven.
- Full world execution is not proven.
- Grading is not proven.
- Creative assessment is not proven.
- Full Alice UI automation is not proven.
- Visible rendering correctness is not proven.
- Save completion is not proven.
- Deployed sharing/platform success is not proven.
```

Unsafe PR wording is any statement that treats the bounded readiness evidence as
proof of a completed lesson, full UI automation, full world execution, visible
rendering correctness, grading, creative assessment, Save completion, or deployed
sharing/platform success. Do not include those statements in docs, generated
adapters, validation summaries, or PR text.

PR #188 uses the same bounded evidence shape. Its final evidence belongs in the
PR body or review handoff after the final passing gate exists, not as a stale
point-in-time result in this reference document. The recovery record names the
exact final commit SHA, the passing `NODE_OPTIONS=--max-old-space-size=32768
TMPDIR=/tmp ./scripts/quality-gates.sh` result for that commit, and the
`mkdocs build --strict` result when docs changed. If another commit is added
after the PR body or review comment is updated, replace the evidence with the new
commit SHA and rerun results before requesting review.

## API and output boundaries

No new public API is introduced by this workflow. It uses the existing readiness
schema:

```text
eatme.alice-lesson-session-readiness/v1
```

PR automation and reviewers consume the same fields documented in
[Real Alice Launch-Smoke Readiness](real-alice-launch-smoke-readiness.md),
[First-Lesson Evidence Readiness](first-lesson-evidence-readiness.md), and
[Lesson Session Readiness](lesson-session-readiness.md). New or updated output
must keep `unproven_claims` visible and must not reinterpret launch-smoke
evidence as first-lesson completion, assessment, rendering correctness, Save
completion, full world execution, Full Alice UI automation, or deployed
sharing/platform success.

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

This page also preserves the PR #175 publication-head evidence contract that was
already on `master` when this recovery branch was brought current.

| Field | Observed value |
| --- | --- |
| Validated evidence head | `a951f34a0a187adfa24cfe0555ca00da6a04197d` |
| Artifact publication head | not embedded in this committed artifact; committing a documentation refinement changes the PR head. |

The validated evidence head is the PR head whose metadata and check rollup were
captured for PR #175. The Artifact publication head is intentionally not
embedded in this committed artifact, because committing a documentation
refinement changes the PR head. This page therefore does not claim that its own
eventual publication commit has checked itself.

## Readiness evidence

### Local Git observations

The local Git evidence for PR #175 was captured before this refinement changed
the artifact/test files. It is a pre-refinement observation for the validated
evidence head and not a claim about the post-edit worktree or the eventual
publication head.
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

This refinement intentionally changes only the readiness artifact and the
contract tests that guard it:

```text
docs/default-workflow-pr-readiness.md
crates/eatme-assets/src/default_workflow_pr_readiness_contract_tests.rs
```

### GitHub PR #175 observations

PR #175 metadata was recorded as GitHub metadata for the validated evidence
head. Those observations are evidence for that head only and do not approve,
merge, or broaden this recovery branch.

### Validated evidence-head executable evidence

Validated evidence-head commands record `NODE_OPTIONS=--max-old-space-size=32768`
and use no timeout wrapper.

```bash
cargo run -q -p eatme-cli -- assets validate --json
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
mkdocs build --strict
TMPDIR=/tmp ./scripts/quality-gates.sh
```

These commands are executable evidence for the recorded PR #175 validated
evidence head only. They do not prove manual real Alice desktop launch, full UI
automation, visible rendering correctness, grading, creative assessment, Save
completion, or lesson completion.

### Historical same-head outside-in testing evidence

Historical `uvx` checks used a branch ref as resolved at execution time, not an
immutable SHA-pinned install reference. Any same-head claim depends on the
recorded execution context, not on the mutable branch name alone.

## Review evidence

Review evidence keeps PR metadata, local Git observations, and executable
validation results separate. Skipped, not-measured, no-execute, and historical
states remain nonclaims instead of success evidence.

## Finalization evidence

PR #175 remains unmerged. No manual merge was performed. The PR #175 record is
workflow readiness/review/finalization evidence, not proof of approval or broad
product readiness.

Finalization status: `merge-ready-after-publication-head-checks` for the PR #175
evidence-contract recovery. The post-push publication head/check rollup recorded
outside this file is required before using it as final merge evidence.

### External publication-head evidence record

The exact publication-head evidence must be recorded outside this committed
artifact after push. A final no-op or merge-ready statement must be a literal
no-op justification tied to the publication head, check rollup, and focused
artifact-contract scope. Owner-free finalization does not require owner
intervention.

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
- No manual merge is claimed.
- No blanket CI success is claimed beyond the listed evidence-head or
  publication-head check rollup.
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

