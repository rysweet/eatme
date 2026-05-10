# PR 174 persona/scenario gap-fill readiness

This page defines the readiness contract for recovered PR 174. The PR is
review-ready for the persona/scenario gap-fill scope only when final handoff
evidence is collected at the exact PR head and the worktree is clean.

The feature scope is editable persona assets, canonical EatMe scenario assets,
generated Gadugi adapter freshness, repository-local asset validation, and
exact-head GitHub metadata.

This is not Alice classroom evidence. It does not prove Alice UI automation,
grading, creative assessment, save/reopen/export completion, deployed sharing,
or lesson completion.

## Exact-head evidence model
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

Do not treat a checked-in commit SHA as current readiness evidence. Any
documentation commit changes `HEAD`, so exact-head evidence belongs in the
final PR handoff note or CI logs after the last commit has been pushed.

Collect exact-head evidence only after syncing to PR #174's actual head and
confirming a clean worktree:
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
git fetch origin pull/174/head:pr-174-persona-gap-fill
git switch pr-174-persona-gap-fill
git status --short
git rev-parse HEAD
gh pr view 174 --json headRefName,headRefOid,mergeStateStatus,mergeable,statusCheckRollup
```

Record the starting HEAD with `git rev-parse HEAD` before making recovery
changes. The final handoff must record these fields for the same commit:

| Field | Required value |
| --- | --- |
| PR | `174` |
| PR `headRefName` | `wave6-persona-gap-fill-1778302300`. |
| Local evidence branch | The branch used to inspect the PR head, such as `pr-174-persona-gap-fill`, may differ from `headRefName`. |
| Local HEAD | The final commit SHA from `git rev-parse HEAD`. |
| PR `headRefOid` | The same SHA as Local HEAD. |
| GitHub merge state | `CLEAN`. |
| GitHub mergeability | `MERGEABLE`. |
| GitHub check rollup | No failed or pending checks for the same head. Skipped checks are acceptable only when they are outside the persona/scenario asset scope and do not expand the readiness claim. |

Repository-local asset checks for the same head must pass:

| Check | Evidence |
| --- | --- |
| `cargo run -q -p eatme-cli -- assets validate --json` | Passes with no errors or warnings. Expected PR 174 asset counts are `instructor_count: 11`, `student_count: 13`, and `scenario_asset_count: 95`. |
| `cargo run -q -p eatme-cli -- assets generate-gadugi --check --json` | Passes with no changed adapters and no errors. Expected PR 174 adapter counts are `generated_count: 47` and `checked_count: 47`. |

If the branch moves, refresh the handoff evidence after syncing to the new PR
head and rerunning the repository-local checks.

## Review evidence

Review evidence must be collected for the same exact head as readiness evidence:

```bash
gh pr view 174 --json headRefOid,reviewDecision,reviews,comments
```

The review gate passes only when:

| Field | Required value |
| --- | --- |
| `headRefOid` | `headRefOid` must match `git rev-parse HEAD`. |
| `reviewDecision` | Record the current `reviewDecision` for the same exact head. |
| `reviews` | Review entries must apply to the same exact head or be treated as historical context only. |
| `comments` | Review comments must be bounded to editable persona/scenario assets, generated adapter freshness, or repository-local asset checks. |

Do not use stale review comments, skipped checks, or local-only validation as
review evidence for a moved branch.

## Finalization evidence

Do not manually merge PR 174 or perform equivalent merge actions.
Do not use shell-level timeout wrapper commands.

Finalization evidence must be based on checks actually run at the exact current
head. Collect readiness, review, and finalization evidence together after the
last repository change and before publishing the final handoff:

```bash
git status --short
git rev-parse HEAD
cargo run -q -p eatme-cli -- assets validate --json
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
cargo test -q -p eatme-assets default_workflow_pr_readiness_tests
gh pr view 174 --json headRefName,headRefOid,mergeStateStatus,mergeable,reviewDecision,reviews,comments,statusCheckRollup
```

The finalization gate passes only when the worktree is clean, local `HEAD`
matches PR `headRefOid`, the repository-local checks above pass for that same
commit, and GitHub reports no failed or pending checks for that same head.

## Source-of-truth assets

Editable assets are the source of truth:

```text
assets/personas/alice-user-crew.yaml
assets/scenarios/eatme/*.yaml
```

Generated Gadugi adapters are review artifacts, not authoritative content:

```text
assets/scenarios/gadugi/*.yaml
```

Refresh generated adapters only with the existing generator when canonical
EatMe scenario assets change:

```bash
cargo run -q -p eatme-cli -- assets generate-gadugi --json
```

Then verify adapter freshness:

```bash
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
```

## Persona/scenario coverage contract

PR 174 is ready when these asset-level statements are true:

| Area | Required state |
| --- | --- |
| Persona crew | `assets/personas/alice-user-crew.yaml` defines instructor and student personas that cover setup, teaching, debugging, assessment-boundary, classroom logistics, creative, accessibility, VR/player, sharing, and reflection needs. |
| Constituencies | `constituency_coverage` links non-coder-editable constituency records to persona IDs and scenario IDs. |
| Scenario references | Canonical EatMe scenarios reference persona IDs that resolve through the persona crew asset. |
| Scenario wording | Scenario text stays at the editable asset level and does not claim full Alice UI automation, grading, creative assessment validation, save/reopen/export completion, or lesson completion. |
| Gadugi adapters | Generated adapters are reproducible from canonical EatMe scenario assets and pass check mode with no changes. |

## Refreshing readiness evidence

Run these commands from the repository root:

```bash
git rev-parse --show-toplevel
```

Sync to the current PR head before collecting evidence:

```bash
git fetch origin pull/174/head:pr-174-persona-gap-fill
git switch pr-174-persona-gap-fill
```

Record the local head and compare it to GitHub:

```bash
HEAD="$(git rev-parse HEAD)"
gh pr view 174 --json headRefName,headRefOid,mergeStateStatus,mergeable,statusCheckRollup
```

The GitHub gate passes only when:

| Field | Required value |
| --- | --- |
| `headRefName` | `wave6-persona-gap-fill-1778302300`. This is the PR branch name, not necessarily the local evidence branch created by `git fetch`. |
| `headRefOid` | Same value as `git rev-parse HEAD`. |
| `mergeStateStatus` | `CLEAN`. |
| `mergeable` | `MERGEABLE`. |
| `statusCheckRollup` | No failed or pending checks for the same head. Skipped checks are acceptable only when they are out of scope for persona/scenario asset readiness. A skipped manual Alice smoke check, for example, must not be cited as Alice UI evidence. |

Run the asset-level validation commands:

```bash
cargo run -q -p eatme-cli -- assets validate --json
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
```

Use the repository quality gate only as an additional repository health check,
not as Alice UI, grading, creative assessment, or lesson-completion evidence:

```bash
TMPDIR=/tmp ./scripts/quality-gates.sh
```

If a local Node-based wrapper around repository tooling hits a heap limit, prefix
that wrapper invocation with `NODE_OPTIONS=--max-old-space-size=32768`. Do not
treat `NODE_OPTIONS` as required project configuration or as part of the PR
readiness contract.

Block readiness if `git status --short` reports local changes, or if GitHub
reports a different `headRefOid`, a non-clean merge state, failed checks,
pending checks, or checks for another commit.

## Supported and unsupported claims

Supported review claims:

- PR 174 fills persona/scenario gaps in editable assets.
- Persona IDs used by canonical EatMe scenarios resolve through the persona crew
  asset.
- Generated Gadugi adapters are fresh relative to canonical EatMe scenarios.
- Repository-local asset validation passes for the exact PR head.
- GitHub metadata names the same exact head and reports the PR mergeable.

Unsupported review claims:

- Alice UI automation completed a full user journey.
- A learner completed a lesson.
- A student world was graded or creatively assessed automatically.
- Save/reopen/export was completed in a live Alice session.
- Visual rendering, deployed sharing, or classroom success was verified.

## Handoff note

Use this template after exact-head evidence is refreshed at the final PR head.
Generate the file list with `git diff --name-only <merge-base>...HEAD` and
paste that exact output under `Files modified:` before publishing a
change-bearing handoff. Documentation and test-source changes are allowed only
when they directly support the persona/scenario recovery contract and are listed
explicitly in this file list.

```text
PR 174 persona/scenario gap-fill readiness

PR branch: wave6-persona-gap-fill-1778302300
Local evidence branch: <git branch --show-current>
HEAD: <git rev-parse HEAD>
PR headRefName: <gh pr view 174 --json headRefName>
PR headRefOid: <gh pr view 174 --json headRefOid>
Merge state: CLEAN
Mergeable: MERGEABLE
Worktree: clean
Check rollup: no failed or pending checks for the same head; skipped checks are
out of scope and are not used as Alice UI, grading, creative assessment, or
lesson-completion evidence.

Editable source assets:
- assets/personas/alice-user-crew.yaml
- assets/scenarios/eatme/*.yaml

Generated review artifacts:
- assets/scenarios/gadugi/*.yaml

Repository-local checks:
- cargo run -q -p eatme-cli -- assets validate --json
- cargo run -q -p eatme-cli -- assets generate-gadugi --check --json

Files modified:
Run `git diff --name-only <merge-base>...HEAD` after the final commit and paste
the exact output here. Do not write `None` in this change-bearing template.
```

Use the no-op template only when implementation review finds no missing
persona/scenario asset, generated adapter, test, or directly linked
documentation work. The no-op path is invalid if `git status --short` is dirty,
if exact-head evidence has not been refreshed, or if the workflow has not
accepted the no-op rationale.

```text
PR 174 persona/scenario gap-fill readiness

PR branch: wave6-persona-gap-fill-1778302300
Local evidence branch: <git branch --show-current>
HEAD: <git rev-parse HEAD>
PR headRefName: <gh pr view 174 --json headRefName>
PR headRefOid: <gh pr view 174 --json headRefOid>
Merge state: CLEAN
Mergeable: MERGEABLE
Worktree: clean
Check rollup: no failed or pending checks for the same head; skipped checks are
out of scope and are not used as Alice UI, grading, creative assessment, or
lesson-completion evidence.

Files modified: None

No-op justification:
At HEAD `<git rev-parse HEAD>`, workflow readiness accepted no repository
changes because the exact-head checks below passed for PR 174's current
`headRefOid`, the worktree was clean before evidence collection, and the
bounded persona/scenario asset scope did not require additional implementation
or documentation files:
- `cargo run -q -p eatme-cli -- assets validate --json`
- `cargo run -q -p eatme-cli -- assets generate-gadugi --check --json`
- `cargo test -q -p eatme-assets default_workflow_pr_readiness_tests`
- `gh pr view 174 --json headRefName,headRefOid,mergeStateStatus,mergeable,reviewDecision,reviews,comments,statusCheckRollup`

Do not use this no-op justification when the worktree is dirty, the PR
`headRefOid` differs from local `HEAD`, required checks are failed or pending,
or the implementation review finds a missing file in the persona/scenario
silver-thread scope. Do not cite deleted, zero-test, or otherwise empty test
filters as no-op evidence.

Boundary: this evidence supports editable persona/scenario asset readiness and
generated adapter freshness for the exact head. It does not claim Alice UI
automation, grading, creative assessment, save/reopen/export completion,
deployed sharing, visual correctness, classroom success, or lesson completion.
```

## Publishing bounded PR evidence

Publish only asset-scoped evidence to PR 174 after the final commit is pushed
and the exact-head checks above have passed:

```bash
gh pr comment 174 --body-file /path/to/pr-174-evidence.md
```

The body file must use this bounded template:

```markdown
Persona/scenario gap-fill readiness refreshed for HEAD `<commit-sha>`.

Asset-scoped evidence:
- `cargo run -q -p eatme-cli -- assets validate --json` succeeded.
- `cargo run -q -p eatme-cli -- assets generate-gadugi --check --json` succeeded.
- Asset changes are limited to canonical persona assets, canonical EatMe scenario assets, and generator-produced Gadugi adapters under `assets/scenarios/gadugi/*.yaml`.
- Files modified are listed in the final handoff from `git diff --name-only <merge-base>...HEAD` and stay within persona/scenario assets, generated adapters, tests, and directly linked documentation.

Scope note: this evidence covers persona/scenario asset completeness, validation success, and generated-adapter freshness only. It does not claim Alice UI automation, grading correctness, creative assessment quality, completed lessons, or full lesson-flow coverage.
```

If GitHub publishing fails because of authentication, API errors, or rate
limiting, preserve the intended PR text unchanged outside the repository and
record the exact `gh` failure beside it. Do not commit fallback logs or local
publish-attempt files to the PR branch.

## Starter-project evidence boundary

Starter-project preflight evidence is bounded setup evidence for opening the
bundled starter project and recording reviewable launch artifacts. It is not PR
readiness, mergeability, production suitability, complete lesson execution,
full Alice UI automation, visible rendering correctness, save/reopen/export
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

## Related documentation

- [Starter Project Preflight Evidence](starter-project-preflight-evidence.md)
- [Save, Reopen, and Export Evidence Handoff](save-reopen-export-evidence-handoff.md)
- [Persona Assets](persona-assets.md)
- [Scenario Authoring](scenario-authoring.md)
- [Gadugi Adapters](gadugi-adapters.md)
- [Generated Asset Consistency](generated-asset-consistency.md)
- [Validation and Quality Gates](validation-quality-gates.md)
- [PR Publish-Failure Recovery](pr-publish-recovery.md)
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

