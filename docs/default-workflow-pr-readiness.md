# Default-workflow PR readiness

## PR #175 evidence contract

This page is the self-contained recovery artifact for PR #175. It records
bounded evidence for a validated PR evidence head, GitHub PR metadata observed
at evidence-capture time, and the publication-head boundary for this artifact.

This is evidence-contract finalization, not validation completion. Treat every
claim below as limited to the command, timestamp, and observed value that
supports it.

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
