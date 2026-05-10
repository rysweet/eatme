# Default-workflow PR readiness

Default-workflow PR readiness is the exact-head gate used when a pull request
needs a clear final readiness decision and the wrapper workflow did not produce
useful output.

This page specifies the readiness behavior the gate requires: verify the exact
PR head, run repository evidence commands directly without timeout wrappers,
inspect GitHub metadata for that same head, classify required green checks
separately from optional skipped jobs, require three quality-audit
SEEK/VALIDATE/FIX cycles with a clean final cycle, preserve the bounded evidence
contracts, and emit a narrowly scoped final artifact only after every gate
passes.

## Contents

- [Readiness contract](#readiness-contract)
- [Generic readiness procedure](#generic-readiness-procedure)
- [Finalization components](#finalization-components)
- [Configuration](#configuration)
- [GitHub metadata fields](#github-metadata-fields)
- [Check classification](#check-classification)
- [Starter-project evidence boundary](#starter-project-evidence-boundary)
- [Generated Gadugi adapter freshness](#generated-gadugi-adapter-freshness)
- [Runnable evidence commands](#runnable-evidence-commands)
- [Quality-audit SEEK/VALIDATE/FIX cycles](#quality-audit-seekvalidatefix-cycles)
- [Silver-thread/e2e gap-matrix readiness](#silver-threade2e-gap-matrix-readiness)
- [Diff scope and docs impact review](#diff-scope-and-docs-impact-review)
- [PR #193 finalization evidence boundary](#pr-193-finalization-evidence-boundary)
- [Historical PR #164 starter-project example](#historical-pr-164-starter-project-example)
- [PR description evidence](#pr-description-evidence)
- [No-op finalization record](#no-op-finalization-record)
- [Final output contract](#final-output-contract)
- [Blocker handling](#blocker-handling)

## Readiness contract

A PR is default-workflow ready only when every gate passes for the exact commit
being reviewed.

| Gate | Required result |
| --- | --- |
| Exact head | The PR head SHA equals the requested SHA. A mismatch blocks readiness. |
| Direct execution | Repository evidence commands run directly, without shell timeout wrappers or background kill wrappers. |
| Runnable evidence | Applicable repository-supported QA and scenario commands pass for the same local checkout. |
| GitHub checks | Every required current-head check run is complete and successful for that same SHA. Optional skipped jobs are recorded separately and do not make a required green check look green. |
| Merge state | `mergeStateStatus` is `CLEAN`. |
| Mergeability | `mergeable` is `MERGEABLE`. |
| Quality audit | At least three SEEK/VALIDATE/FIX cycles complete, and the final cycle is clean. |
| Starter-project wording | The canonical scenario uses plain, bounded, user-facing language. |
| Overclaim boundary | The scenario does not claim first-lesson completion, grading, creative assessment, full UI automation, visible rendering correctness, full Save completion, or complete Alice coverage. |
| Gadugi adapters | Generated adapters are fresh whenever canonical scenario assets are affected. |
| Silver-thread/e2e gap matrix | When the PR touches lesson-session readiness docs, the scenario map and gap matrix name only the bounded user journey, remaining missing proof, and evidence still needed. |
| Docs impact | Documentation changes build under strict MkDocs and do not record stale point-in-time evidence. |
| Scope | No unrelated files or behavior are changed. |
| Final artifact | The result begins with `No-op` for an evidence refresh that needs no repository edits, `MERGE_READY_EVIDENCE` for a successful readiness decision, or `NOT_MERGE_READY` when any gate blocks readiness. |
| PR evidence | The same-repository PR body or a trusted PR comment records current-head evidence, or the final artifact is `NOT_MERGE_READY` with the missing evidence named. |

A previous wrapper failure is not a blocker when direct verification proves the
same head, runnable evidence, successful current-head checks, clean
mergeability, bounded wording, fresh generated adapters, focused scope, current
PR evidence, and a clean final quality-audit cycle. If any item is missing,
stale, pending, ambiguous, or tied to a different head, the gate reports
`NOT_MERGE_READY` instead of readiness.

## Generic readiness procedure

Run the gate in this order:

1. Use a local checkout only when file inspection or command evidence is needed.
   Before using local evidence, verify the checked-out PR branch and local
   `HEAD` both match the PR `headRefOid`.
2. Run applicable repository evidence commands directly, without timeout
   wrappers.
3. Classify the current-head check rollup into required green checks, optional
   skipped jobs, failed checks, pending checks, and unknown checks.
4. Verify `mergeStateStatus=CLEAN` and `mergeable=MERGEABLE`.
5. Inspect the starter-project preflight scenario wording if the PR touches that
   evidence contract.
6. Run the generated Gadugi adapter freshness check if any canonical scenario
   asset is affected.
7. Inspect the lesson-session scenario map and gap matrix if the PR touches
   silver-thread/e2e readiness documentation.
8. Review the diff scope against the PR purpose and reject unrelated changes.
9. Review docs impact and ensure strict MkDocs evidence covers doc changes.
10. Complete three quality-audit SEEK/VALIDATE/FIX cycles; the third cycle must
    be clean.
11. Review the same-repository PR body and trusted comments for current-head evidence.
12. If the PR body or trusted comments are updated, reconfirm the PR head after the
    update. The reconfirmed head must still match the local evidence head.
13. Emit `No-op` when the requested finalization needs no repository edits and
    every no-op gate passed for the reconfirmed head. Emit
    `MERGE_READY_EVIDENCE` when every readiness gate passed for the reconfirmed
    head. Otherwise emit `NOT_MERGE_READY` with the blocker.

## Finalization components

The finalization workflow is implemented as a small evidence pipeline. Each
component owns one decision and passes explicit facts to the next component; no
component infers readiness from an older PR head or from uncommitted files.

| Component | Responsibility | Output |
| --- | --- | --- |
| PR Metadata Reader | Reads live PR metadata with `gh pr view`, including `headRefOid`, state, draft status, mergeability, review decision, files, body, comments, reviews, and check rollup. | Current PR facts bound to one head SHA. |
| Check Classifier | Separates required green checks from optional skipped jobs, failed checks, pending checks, and unknown checks. | A check summary with blockers named explicitly. |
| Gap-Matrix Evidence Inspector | Confirms that gap-matrix or readiness evidence names the current `headRefOid` and matches the scoped lane. | Current-head evidence status: current, stale, missing, or inconsistent. |
| Repository Scope Inspector | Confirms changed files are explained by the gap-matrix/readiness lane before accepting evidence. | Focused-scope or blocker decision. |
| Edit Decision Gate | Allows repository edits only when evidence is stale, missing, or inconsistent with the documented scope. | Edit-required or no-op decision. |
| Finalization Reporter | Emits the final artifact with head, checks, scoped evidence, blockers, and no-manual-merge status. | `No-op`, `MERGE_READY_EVIDENCE`, or `NOT_MERGE_READY`. |

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

Run repository commands directly. Do not wrap readiness commands in `timeout`,
background watchdogs, or shell snippets that kill the command after a fixed
duration. A long-running command should finish, fail, or be reported as blocked
with the command still tied to the reviewed head.

For GitHub checks, use authenticated `gh` access to the repository that owns the
PR. Do not place tokens, secrets, local credential paths, environment dumps, or
raw command output in final artifacts.

The workflow never runs `git merge`, `gh pr merge`, force-pushes, rebases, or
equivalent manual merge operations while collecting readiness evidence.

## GitHub metadata fields

The readiness gate consumes these `gh pr view` fields:

| Field | Required value |
| --- | --- |
| `headRefOid` | Exact requested SHA |
| `state` | `OPEN` while finalization evidence is collected |
| `isDraft` | `false` |
| `mergeStateStatus` | `CLEAN` |
| `mergeable` | `MERGEABLE` |
| `reviewDecision` | Reported as evidence; a blocking value is surfaced as a blocker. |
| `files` | All changed files are explained by the scoped readiness lane. |
| `body`, `comments`, and `reviews` | Same-repository or trusted evidence can be matched to the current `headRefOid`. |
| `statusCheckRollup` | Every required current-head check run is complete and successful for `headRefOid`; optional skipped jobs are reported separately. |

Fetch the PR head, review state, changed files, trusted evidence surfaces, and
check summary:

```bash
gh pr view 193 \
  --json number,url,state,isDraft,headRefName,headRefOid,mergeStateStatus,mergeable,reviewDecision,files,body,comments,reviews,statusCheckRollup
```

A successful check rollup for an older SHA is not evidence for the current head.
If the head changes during review, stop and restart the readiness verification
for the newly requested SHA.

## Check classification

The Check Classifier turns `statusCheckRollup` into five buckets before any
readiness or no-op decision is made:

| Bucket | Meaning | Readiness effect |
| --- | --- | --- |
| Required green checks | Required checks whose `status` is complete and whose `conclusion` is success or neutral when branch protection accepts neutral. | Required for readiness. |
| Optional skipped jobs | Jobs skipped by workflow conditions, such as deploy jobs that run only on the default branch or manually gated real Alice smoke jobs. | Recorded as optional evidence; not a blocker when branch protection does not require them. |
| Failed checks | Required or unclassified checks with failure, error, timed-out, cancelled, action-required, or startup-failure conclusions. | Block readiness. |
| Pending checks | Required or unclassified checks that are queued, requested, waiting, expected, or in progress. | Block readiness until complete. |
| Unknown checks | Checks whose required/optional status cannot be determined from branch protection, workflow policy, or repository convention. | Block readiness until classified. |

Skipped status is safe only for optional jobs. A skipped required check, a missing
required check, or a check reported for any head other than `headRefOid` blocks
readiness. The final artifact names both the required green checks and optional
skipped jobs so reviewers can see that skipped jobs were classified rather than
ignored.

## Starter-project evidence boundary

The source contract for this boundary is split across:

- `docs/default-workflow-pr-readiness.md`
- `docs/starter-project-preflight-evidence.md`

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

## Runnable evidence commands

For a PR that touches lesson-session readiness docs, scenario ids, generated
adapters, or the default-workflow readiness contract, the workflow runs these
repository-supported commands from the repository root:

```bash
cargo run -q -p eatme-cli -- assets validate --json
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
mkdocs build --strict
TMPDIR=/tmp ./scripts/quality-gates.sh
```

The evidence is valid only for the checkout whose `git rev-parse HEAD` matches
the PR `headRefOid`. The asset commands prove committed asset validity and
generated-adapter freshness. The MkDocs command proves strict documentation
buildability. The quality-gates script proves the repository's Rust formatting,
linting, tests, module-size, and coverage gates as implemented by the script.

These commands do not prove full UI automation, visible rendering correctness,
grading, creative assessment, full Save completion, lesson completion, live
classroom use, or full Tweedle/player decode.

## Quality-audit SEEK/VALIDATE/FIX cycles

The readiness workflow performs at least three quality-audit cycles after the
initial evidence commands and before the final readiness claim.

| Phase | Required action |
| --- | --- |
| SEEK | Look for head mismatches, stale PR evidence, failed or pending checks, overclaiming documentation, unrelated diff scope, missing docs impact, stale generated adapters, and missing command evidence. |
| VALIDATE | Bind each suspected issue to exact evidence: command output, `gh` metadata, trusted PR body/comment text, changed files, or committed documentation. Unsupported or ambiguous findings become blockers. |
| FIX | Apply the minimal repository or PR-evidence correction when a fix is in scope. If no repository change is needed, record a no-op rationale tied to the exact head instead of editing files. |

Cycle 1 establishes the first complete blocker list. Cycle 2 validates that
fixes or no-op rationales did not create new gaps. Cycle 3 is the final clean
cycle: it must find no unresolved blockers, no stale evidence, and no new fixes
needed. If the third cycle finds any issue, the workflow fixes or blocks it and
then starts another cycle until the final cycle is clean.

The audit result is evidence metadata, not product capability proof. It supports
only the merge-ready gate for the reviewed PR head.

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

## Diff scope and docs impact review

Diff scope is focused when every changed file is necessary for the PR's stated
purpose and the documentation, tests, or code all point at the same evidence
contract. For the lesson-session gap-matrix lane, the expected scope is the
lesson-session readiness documentation, default-workflow readiness
documentation, and any Rust doc tests or module wiring that enforce those docs.

The scope review checks:

| Surface | Required result |
| --- | --- |
| Changed files | Each path is explained by the PR purpose. Unrelated source, generated, or docs changes block readiness. |
| Documentation impact | Strict MkDocs succeeds, new docs are linked from navigation or an existing parent page, and docs avoid point-in-time command transcripts. |
| Test impact | Doc-test or quality-gate changes enforce the documented contract without expanding runtime claims. |
| Claim boundary | The diff does not introduce claims beyond the runnable evidence commands and exact-head GitHub metadata. |

When the diff is documentation-only, strict MkDocs is still required. Asset and
Gadugi checks are also required when the docs mention scenario ids, persona
assets, generated adapters, or evidence contracts that depend on those assets.

## PR #193 finalization evidence boundary

PR #193 uses this committed section only as a readiness contract. Do not record a
specific PR #193 `headRefOid`, evidence timestamp, or "passed for this head"
claim inside this file: the commit that changes this file also changes the PR
head, so committed exact-head evidence here would become self-stale.

After the final commit is pushed, gather fresh exact-head evidence with:

```bash
gh pr view 193 \
  --json number,url,state,isDraft,headRefName,headRefOid,mergeStateStatus,mergeable,reviewDecision,files,body,comments,reviews,statusCheckRollup
```

Record the resulting `headRefOid`, command outcomes, required green checks,
optional skipped jobs, bounded change scope, gap-matrix evidence status, and
no-manual-merge decision outside the repository commit, such as in the
same-repository PR body, a trusted PR comment, or status summary. Do not treat
uncommitted local documentation edits as evidence for a PR head.

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
repository quality gates, three clean-ending quality-audit cycles, focused diff
scope, current PR evidence, and required green GitHub checks tied to the
externally recorded exact head. Optional skipped jobs are listed as skipped and
optional; they are not counted as green checks. The conclusion does not claim
full UI automation, rendering correctness, grading, creative assessment, Save
completion, lesson completion, live classroom use, manual merge completion, or
any evidence from uncommitted local files.

## Historical PR #164 starter-project example

This subsection is a concrete example for the PR #164 finalization gate. Do not
reuse its PR number, SHA, or narrowed starter-project scope as the template for
future readiness decisions.

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

Under the current contract, the readiness decision for PR #164 would be valid
only if those commands pass, every check run is complete and successful for
`eb0bb29b7cc1f8647e9a36c0bc8200fb3fdc5cba`,
`mergeStateStatus=CLEAN`, `mergeable=MERGEABLE`, the scenario wording stays
within the starter-project evidence boundary above, generated adapters are
fresh, the diff scope is focused, docs impact is reviewed, PR evidence names the
current head, no manual merge was run, and three SEEK/VALIDATE/FIX cycles
complete with a clean final cycle. The final artifact would begin with
`MERGE_READY_EVIDENCE` only after those gates pass; any missing item would
produce `NOT_MERGE_READY`.

## PR description evidence

The same-repository PR body or a trusted PR comment must contain evidence for the
current final head before readiness is recorded. Trusted comments are comments
from `OWNER`, `MEMBER`, or `COLLABORATOR` author associations. The evidence may
be concise, but it must be specific enough that a reviewer can match every claim
to the exact head.

| Evidence item | Required content |
| --- | --- |
| Head identity | PR number, branch name, and current `headRefOid`. |
| Local command evidence | The applicable repository commands and pass/fail result for the same local checkout. |
| GitHub Actions | Check summary showing required current-head checks complete and successful for the same head, with optional skipped jobs listed separately. |
| Diff scope | Changed files reviewed and why the scope is focused. |
| Docs impact | Strict MkDocs result and the bounded documentation claim being supported. |
| Quality audit | Three SEEK/VALIDATE/FIX cycles with the final cycle clean, or an explicit blocker. |
| No manual merge | Statement that no `git merge`, `gh pr merge`, or equivalent manual merge was run. |

Stale PR evidence is a blocker. Evidence is stale when it names an older SHA,
omits the current SHA, relies only on previous local output after the branch has
moved, or claims readiness without the final clean quality-audit cycle.

If the PR description cannot be updated, the gate records
`NOT_MERGE_READY` with the missing evidence item rather than relying on implied
checks.

## No-op finalization record

Use a no-op finalization only when the repository already contains the required
implementation and documentation at the exact PR head. The record begins with the
literal marker `No-op` and must include:

| Field | Required value |
| --- | --- |
| PR and branch | PR number and exact branch name. |
| Exact head | `headRefOid` evaluated after local checks and immediately before finalization. |
| Diff scope | The changed files reviewed for the readiness claim. |
| Local commands | Each repository-defined command, its exit result, and the bounded fact it proves. |
| GitHub state | PR state, merge state, and check summary for the same head. |
| No-op reason | A plain statement that no repository file changes are required because the current head already satisfies the documented contract. |
| Gap-matrix evidence | Confirmation that scoped gap-matrix lane evidence is current for the same `headRefOid`, or the specific stale/missing evidence blocker. |
| Quality audit | Three SEEK/VALIDATE/FIX cycles completed with a clean final cycle. |
| No-merge statement | A statement that readiness was recorded without manually merging the PR. |

The no-op record must not use future-tense assumptions. If any command fails,
the PR head changes, or the reviewed docs imply unsupported runtime claims, the
workflow is not a no-op; fix the minimal issue and gather fresh evidence for the
new head.

A dirty worktree is never a no-op finalization source. Local edits that have not
been committed and pushed make any exact-head readiness record stale.

Use this shape for a no-op evidence refresh:

```text
No-op
PR: #<number> (<headRefName>)
Exact head: <headRefOid>
Checks: required checks green; optional skipped jobs: <names or none>
Gap-matrix lane evidence: current for <headRefOid> and scoped to <bounded lane>
No-op reason: no repository file changes are required because the current head already satisfies the documented contract
Blockers: none
No manual merge: no git merge, gh pr merge, force-push, rebase, or equivalent manual merge operation was run
```

## Final output contract

The final result must begin with exactly one literal marker:

| Marker | When allowed |
| --- | --- |
| `No-op` | The requested work is an evidence refresh or finalization check, the current PR head already satisfies the documented contract, required checks are green, optional skipped jobs are classified, scoped gap-matrix evidence is current, and no repository edits are required. |
| `MERGE_READY_EVIDENCE` | Every readiness gate passed for the reconfirmed exact PR head, and any trusted PR body/comment mutation has been followed by another head check. |
| `NOT_MERGE_READY` | Any gate is failing, pending, missing, ambiguous, tied to the wrong head, or not yet recorded as current-head PR evidence. |

`No-op` is an edit-decision artifact, not a manual merge artifact. It is allowed
only when the final report also states the exact head, required green checks,
optional skipped jobs, scoped gap-matrix evidence, blockers, and no-manual-merge
status.

`MERGE_READY_EVIDENCE` is the successful readiness artifact for a merge-ready
decision. It must name the exact head and the bounded evidence it proves:

```text
MERGE_READY_EVIDENCE
PR: #<number> (<headRefName>)
Exact head: <headRefOid>
Local head: <git rev-parse HEAD>

Command evidence for this head:
- cargo run -q -p eatme-cli -- assets validate --json: passed
- cargo run -q -p eatme-cli -- assets generate-gadugi --check --json: passed
- mkdocs build --strict: passed
- TMPDIR=/tmp ./scripts/quality-gates.sh: passed

GitHub evidence:
- required checks: every required current-head check run complete and successful
- optional skipped jobs: <names or none>
- mergeStateStatus: CLEAN
- mergeable: MERGEABLE

Review evidence:
- diff scope: focused on <bounded PR purpose>
- docs impact: strict MkDocs passed; docs claim only <bounded evidence lane>
- PR evidence: trusted body/comment records this exact head and command evidence
- quality audit: three SEEK/VALIDATE/FIX cycles completed; final cycle clean
- no manual merge: no git merge, gh pr merge, force-push, rebase, or equivalent manual merge operation was run

Boundary: readiness covers only the documented exact-head evidence lane. It does not claim full UI automation, rendering correctness, grading, creative assessment, full Save completion, lesson completion, live classroom use, or manual merge completion.
```

`NOT_MERGE_READY` must name the blocker and the minimal next action:

```text
NOT_MERGE_READY
PR: #<number> (<headRefName>)
Observed head: <headRefOid or unknown>
Blocker: <specific failed, pending, missing, ambiguous, stale, or wrong-head gate>
Required next action: <minimal fix or evidence step before readiness can be retried>
```

## Blocker handling

If any gate fails, do not publish readiness. Emit `NOT_MERGE_READY` with the
specific blocker. Fix only the minimal issue that caused the blocker, run the
relevant validation again, push the fix when a repository change is required,
and repeat exact-head verification against the new PR head.

| Blocker | Minimal response |
| --- | --- |
| Head mismatch | Stop readiness for the old SHA and verify the requested new head. |
| Failing, pending, cancelled, missing, wrong-head, or unclassified checks | Fix the failing check, wait for completion, rerun the missing check, or classify optional skipped jobs before readiness. |
| Dirty merge state | Resolve only the mergeability issue. |
| Overclaiming scenario language | Edit the canonical scenario wording and regenerate adapters if affected. |
| Stale generated adapter | Regenerate adapters from canonical sources. |
| Asset validation failure | Fix the invalid scenario or persona asset. |
| Unrelated changes | Remove the unrelated change from the readiness work. |
| Missing quality-audit cycle | Run the missing SEEK/VALIDATE/FIX cycle and require a clean final cycle. |
| Stale or untrusted PR evidence | Update the same-repository PR body or a trusted comment with current-head evidence, then reconfirm the head. |
| Missing no-op rationale | Record why no repository changes are required for the current head, or make the minimal required change. |

---

# PR #175 evidence contract (preserved from master)

The following sections preserve the PR #175 evidence contract that was on master
when this PR was rebased. The contract tests in
`default_workflow_pr_readiness_contract_tests.rs` and the overclaim rules in
`starter_project_preflight_boundary_tests.rs` validate content below.

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

