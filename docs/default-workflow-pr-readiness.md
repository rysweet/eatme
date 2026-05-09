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
