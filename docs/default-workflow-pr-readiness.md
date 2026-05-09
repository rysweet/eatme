# Default-workflow PR readiness

Default-workflow PR readiness defines the intended exact-head gate for pull
requests that need a clear final readiness decision when the wrapper workflow
does not produce useful output. It also defines the intended recovery path for
nonclaim audit lanes when a previous lane recorded an invalid manual fallback.

This page specifies the readiness behavior the workflow should enforce: verify
the exact PR head, keep work on the intended lane branch, inspect GitHub
metadata for that same head, preserve bounded scenario language, require fresh
generated Gadugi adapters when scenario assets are involved, and publish a
narrowly scoped readiness comment only after every gate passes.

## Contents

- [Readiness contract](#readiness-contract)
- [Implementation status](#implementation-status)
- [Recovery lane contract](#recovery-lane-contract)
- [Generic readiness procedure](#generic-readiness-procedure)
- [Recovery CLI usage](#recovery-cli-usage)
- [Manual wrapper bypass](#manual-wrapper-bypass)
- [Configuration](#configuration)
- [External GitHub adapter](#external-github-adapter)
- [GitHub metadata fields](#github-metadata-fields)
- [Validation commands](#validation-commands)
- [Quality-audit SEEK/VALIDATE/FIX cycles](#quality-audit-seekvalidatefix-cycles)
- [Focused diff scope audit](#focused-diff-scope-audit)
- [Documentation impact gate](#documentation-impact-gate)
- [PR description evidence](#pr-description-evidence)
- [Starter-project evidence boundary](#starter-project-evidence-boundary)
- [Scenario nonclaim boundary](#scenario-nonclaim-boundary)
- [Generated Gadugi adapter freshness](#generated-gadugi-adapter-freshness)
- [Evidence API](#evidence-api)
- [Recovery readiness input schema](#recovery-readiness-input-schema)
- [Recovery report output](#recovery-report-output)
- [No-op and files-modified evidence](#no-op-and-files-modified-evidence)
- [Wave7 nonclaim audit example](#wave7-nonclaim-audit-example)
- [PR #204 nonclaim audit readiness record](#pr-204-nonclaim-audit-readiness-record)
- [Historical stale/non-current PR #164 readiness example](#historical-stalenon-current-pr-164-readiness-example)
- [Readiness comment](#readiness-comment)
- [Blocker handling](#blocker-handling)

## Readiness contract

A PR is default-workflow ready only when every gate passes for the exact commit
being reviewed.

| Gate | Required result |
| --- | --- |
| Lane branch | The PR uses the intended branch. For wave7 nonclaim audit, that branch is `wave7-eatme-nonclaim-audit-1778303500`. |
| Exact head | The PR head SHA equals the expected evidence SHA: the full 40-character PR head intended for validation. A mismatch blocks readiness. |
| GitHub checks | Required checks complete successfully for that same SHA. Optional skipped checks may be reported as skipped, not passed. |
| Merge state | `mergeStateStatus` is `CLEAN`. |
| Mergeability | `mergeable` is `MERGEABLE`. |
| Scenario wording | Canonical scenarios preserve user-facing language unless a narrowly required nonclaim/readiness fix is needed. |
| Overclaim boundary | Scenarios, docs, PR text, and comments do not claim first-lesson completion, full lesson completion, grading, creative assessment, full Alice UI automation, full world execution, UI rendering correctness, visible rendering correctness, Save completion, deployed sharing/platform success, complete Alice coverage, or full Tweedle/player decode. |
| Asset validation | Scenario and persona assets pass the repository asset validator. |
| Gadugi adapters | Generated adapters are fresh whenever canonical scenario assets are affected or the recovery lane explicitly requires adapter evidence. |
| Scope | No unrelated files or behavior are changed. |

A previous wrapper failure is not a blocker when default-workflow verification
proves the same head, successful required checks, clean mergeability, bounded
wording, valid assets, and generated adapter freshness when required. A manual
fallback is never readiness evidence for this gate.

Each readiness or review evidence item must name the exact PR head SHA observed
when that item was collected. Phrases such as "current head", "tested head", or
"latest branch" are only acceptable when paired with the full 40-character SHA.
If any commit is added after evidence is collected, the evidence for the old SHA
is stale/non-current until the gate is rerun and the PR-facing record is updated
for the new head. Newer readiness notes supersede older PR-facing evidence;
older comments must be replaced, edited, or explicitly marked stale/non-current.

## Implementation status

This page documents the recovery evaluator and its surrounding manual evidence
workflow. The CLI evaluates collected evidence; it does not collect or publish
that evidence by itself.

The repository currently provides:

| Surface | Current behavior |
| --- | --- |
| Validation commands | `assets validate`, `assets generate-gadugi --check`, `./scripts/quality-gates.sh`, and `mkdocs build --strict` are runnable repository gates. |
| Rust readiness helpers | `crates/eatme-cli/src/pr_readiness.rs` exposes exact-head and recovery helper types used by tests. |
| Recovery input helper | `RecoveryReadinessInput` includes the expected PR #204 remote head, PR snapshot, trusted required-check names, validation SHA, local QA evidence, quality-audit cycles, diff scope, docs impact, PR-description evidence, stale-evidence handling, wrapper failures, and change outcome. |
| Check summaries | `CheckSummary` stores check name, status, conclusion, required/optional status, and the exact PR head SHA. |
| GitHub adapter | `pr-readiness github-snapshot` fetches PR metadata through authenticated `gh` and annotates caller-specified required checks for reporting; recovery gating uses the trusted input list. |
| Report renderer | `render_final_report` renders `MERGE_READY` or `NOT_MERGE_READY`, exact-head evidence sections, sanitized caller-controlled text, and historical wrapper-failure context. |
| CLI | `cargo run -q -p eatme-cli -- pr-readiness recovery-evaluate --input <file> --json` evaluates collected recovery evidence. |

The remaining workflow boundary is publication: the CLI evaluates evidence and
fetches GitHub metadata, but it does not push commits, mutate PR text, post
comments, bypass checks, or merge.

## Recovery lane contract

Default-workflow recovery keeps the lane deterministic and non-destructive:

1. Fetch origin refs.
2. If the required branch exists on origin, use that remote lane and a local
   branch that tracks it.
3. Reuse an existing local branch only after confirming that it tracks the
   remote lane, matches the remote lane, or was intentionally created for this
   recovery lane from `origin/master`.
4. If the required branch does not exist locally or on origin, create it from
   `origin/master`.
5. Apply only documentation, wording-boundary, canonical scenario, generated
   adapter, or readiness evidence changes that belong to the lane.
6. Commit and push only the intended lane changes.
7. Open or update the PR for that branch only.

For the wave7 eatme nonclaim audit lane, the branch name is:

```text
wave7-eatme-nonclaim-audit-1778303500
```

The branch is a recovery lane, not a product-readiness claim. Its PR can record
validated asset, adapter, documentation, mergeability, and exact-head evidence.
It must not imply full Alice UI automation, full world execution, UI rendering
correctness, visible rendering correctness, grading, creative assessment, Save
completion, deployed sharing/platform success, first-lesson completion, full
lesson completion, or full Tweedle/player decode.

## Generic readiness procedure

Run the gate in this order:

1. Fetch origin refs and switch to the required lane branch.
2. Verify the PR head equals the expected evidence SHA.
3. Verify required GitHub checks completed successfully for that same SHA; report
   optional skipped checks only as skipped.
4. Verify `mergeStateStatus=CLEAN` and `mergeable=MERGEABLE`.
5. Inspect scenario and readiness wording if the PR touches those contracts.
6. Run asset validation.
7. Run the generated Gadugi adapter freshness check when scenario assets are
   affected or the lane explicitly requires adapter evidence.
8. Run the repository quality gate when the lane requires whole-repository
   evidence.
9. Build the documentation site when docs changed.
10. Publish the readiness comment only when every required gate passed.
11. Re-query the PR head after the final commit and after posting the comment.
    If the PR head differs from the SHA named by the evidence, rerun the gate
    and replace the PR-facing evidence for the new head.

The workflow records the exact tested commit with:

```bash
git rev-parse HEAD
```

Any new commit invalidates earlier readiness evidence until the gate is rerun for
the new head.

Do not treat committed documentation as the authoritative "current head" record
for the commit that contains it. The authoritative current-head record is the
PR-facing readiness note posted after the last evidence-changing commit. Any
committed example that names an older SHA must be labeled stale/non-current or
historical.

## Recovery CLI usage

The recovery evaluator consumes a single JSON evidence file and prints either a
bounded readiness report or an explicit `NOT_MERGE_READY` blocker list. It does
not fetch refs, run tests, update PR text, push commits, or merge the PR. Those
actions stay outside the evaluator so the recorded evidence remains auditable.

Run the evaluator from the repository root after collecting all evidence for the
same exact head:

```bash
export NODE_OPTIONS=--max-old-space-size=32768
cargo run -q -p eatme-cli -- pr-readiness recovery-evaluate \
  --input pr-readiness-recovery.json \
  --json
```

The non-JSON form prepares a PR comment:

```bash
cargo run -q -p eatme-cli -- pr-readiness recovery-evaluate \
  --input pr-readiness-recovery.json
```

The command is fail-closed. It reports `NOT_MERGE_READY` when any required
evidence is missing, stale, collected for a different SHA, or broader than the
documented claim boundary. Green GitHub Actions and workflow completion are
necessary inputs, but they are not sufficient by themselves.

The recovery command must never accept timeout wrapper output as readiness
evidence. Run repository-supported commands directly, without `timeout`,
`gtimeout`, or a workflow wrapper that can terminate the command before it
finishes.

## Manual wrapper bypass

Default-workflow recovery supports a manual bypass when workflow wrapper tooling
cannot produce a useful result because of rate limits, a no-op guard, or another
wrapper-only failure. The bypass is the documented finished behavior, not an
exception to readiness.

The manual path is valid only when it repeats the repository gates directly for
the final candidate head:

1. Resolve the PR branch and `headRefOid`.
2. Confirm the local branch is the PR head branch and is not `master`.
3. Capture `git rev-parse HEAD`.
4. Run the asset validation, generated Gadugi freshness check, and repository
   quality gate from that exact checkout.
5. Run `mkdocs build --strict` from that exact checkout when documentation
   changed.
6. Re-query `headRefOid` after the final file change or no-op decision.
7. Publish evidence only when the final local HEAD still equals the final PR
   head.

Wrapper history is reported as workflow context. It is not readiness evidence,
and it is not a blocker once current-head validation passes manually. Evidence
from a wrapper run, manual run, or previous comment collected before the final
candidate head is stale/non-current and must not be cited as the final result.

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

Use `/tmp` for temporary files when running quality gates from deep worktrees:

```bash
export TMPDIR=/tmp
```

For GitHub checks, use authenticated `gh` access to the repository that owns the
PR. Do not place tokens, secrets, local credential paths, environment dumps, or
raw command output in readiness comments.

## External GitHub adapter

The recovery evaluator does not embed GitHub credentials or call GitHub APIs
directly. It uses the existing command-runner boundary to invoke authenticated
`gh` with a fixed `pr view` query, a 20-second timeout, and three attempts with a
short retry delay. Failed `gh` calls, malformed JSON, invalid SHAs, missing
required-check names, or missing required checks all fail closed.

Collect the external PR snapshot before building the full recovery evidence
file:

```bash
cargo run -q -p eatme-cli -- pr-readiness github-snapshot \
  --owner rysweet \
  --repo eatme \
  --pr-number 204 \
  --local-head-sha "$(git rev-parse HEAD)" \
  --required-check quality-gates \
  --json
```

Pass every branch-protection check that must be green with a separate
`--required-check` argument. The snapshot uses those names to annotate check
summaries, but the recovery evaluator still requires the trusted
`required_github_checks` list in the evidence envelope and gates by that list
rather than by caller-supplied `required` flags alone. Checks not named this way
remain optional; skipped optional checks may be reported as skipped but must not
be described as passed.

## GitHub metadata fields

The readiness gate consumes these `gh pr view` fields:

| Field | Required value |
| --- | --- |
| `headRefName` | Intended lane branch |
| `headRefOid` | Expected evidence SHA |
| `mergeStateStatus` | `CLEAN` |
| `mergeable` | `MERGEABLE` |
| `statusCheckRollup` | Required checks complete successfully for `headRefOid`; optional skipped checks are reported only as skipped. |

Fetch the PR head, merge state, mergeability, and check summary:

```bash
gh pr view <pr-number> \
  --json headRefName,headRefOid,mergeStateStatus,mergeable,statusCheckRollup
```

`statusCheckRollup` is acceptable only when every required check for `headRefOid`
has completed successfully. A required check blocks readiness when it is pending,
queued, in progress, requested, failing, errored, timed out, skipped when branch
protection requires it to run, cancelled, missing, or reported for a different
head. Optional checks may be skipped, but readiness evidence must call them
skipped rather than passed.

If the head changes during review, stop and restart the readiness verification
for the newly observed evidence SHA.

## Validation commands

Asset validation is the canonical repository check for scenario and persona
asset structure, including readiness and honest-boundary wording enforced by the
asset validators:

```bash
export NODE_OPTIONS=--max-old-space-size=32768
cargo run -q -p eatme-cli -- assets validate --json
```

Generated Gadugi freshness proves that generated adapters match the canonical
scenario assets. It is required whenever canonical scenario assets are affected
and whenever a recovery lane explicitly names adapter freshness as evidence:

```bash
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
```

When the lane requires whole-repository evidence, run the quality gate with a
short temporary directory path and preserve the repository Node heap option:

```bash
TMPDIR=/tmp NODE_OPTIONS=--max-old-space-size=32768 ./scripts/quality-gates.sh
```

Readiness evidence may name these commands and whether they passed for the exact
head. It should not paste secrets, raw environment dumps, or lengthy command
output into the PR.

When documentation changes, build the documentation site at the exact tested
head:

```bash
mkdocs build --strict
```

## Quality-audit SEEK/VALIDATE/FIX cycles

The recovery evaluator requires at least three quality-audit cycles. Each cycle
is a small evidence record with three named phases:

| Phase | Required evidence |
| --- | --- |
| `SEEK` | The audit target and the concrete risk being searched for, such as stale exact-head evidence, unsupported claims, missing local QA, wrong-head GitHub checks, unfocused diffs, or documentation impact. |
| `VALIDATE` | The command, query, file review, or PR metadata inspection used to prove whether the risk exists at the exact head SHA. |
| `FIX` | The minimal corrective action taken, or `No change required` when validation found no blocker. |

The first two cycles may find and fix blockers. The final cycle must be clean:
all required checks pass, no new repository change is needed, no stale evidence
remains current-facing, and the PR-facing record is ready to publish or already
matches the final head. A final cycle that only says "CI is green" is not clean
unless it also validates local QA, docs impact, diff scope, PR description
evidence, nonclaims, and stale-evidence handling.

The JSON input records cycles with contiguous, strictly increasing cycle numbers
starting at 1:

```json
{
  "quality_audit_cycles": [
    {
      "cycle_number": 1,
      "phases": ["Seek", "Validate", "Fix"],
      "outcome": "FixApplied",
      "head_sha": "<40-character PR head SHA>",
      "summary": "Found stale readiness comments for older SHAs, reviewed PR body and comments for 40-character SHA references, and labeled older evidence stale/non-current."
    },
    {
      "cycle_number": 2,
      "phases": ["Seek", "Validate", "Fix"],
      "outcome": "FixApplied",
      "head_sha": "<40-character PR head SHA>",
      "summary": "Found unsupported UI, grading, or completion claims, reviewed changed docs, scenarios, adapters, and PR evidence, and replaced overclaims with bounded nonclaims."
    },
    {
      "cycle_number": 3,
      "phases": ["Seek", "Validate", "Fix"],
      "outcome": "Clean",
      "head_sha": "<40-character PR head SHA>",
      "summary": "Rechecked exact-head evidence completeness and confirmed local QA, docs build, checks, diff scope, and PR evidence all name the final SHA; no change required."
    }
  ]
}
```

Fewer than three cycles, a missing phase, or a final cycle result other than
`Clean` produces `NOT_MERGE_READY`.

## Focused diff scope audit

The recovery gate accepts only changes that belong to the lane. For the wave7
nonclaim audit lane, focused scope includes:

| Scope | Examples |
| --- | --- |
| Readiness documentation | `docs/default-workflow-pr-readiness.md`, index links, and directly related usage text. |
| Readiness CLI/tests | `crates/eatme-cli/src/pr_readiness.rs`, `crates/eatme-cli/src/pr_readiness/recovery.rs`, `crates/eatme-cli/src/main.rs`, and targeted tests. |
| uvx recovery wrapper | `src/eatme_uvx/cli.py` when remote-branch `uvx --from git+... amplihack ...` execution is the recovery blocker. |
| uvx packaging | `pyproject.toml` when remote-branch packaging or entry-point behavior is part of the recovery blocker. |
| Quality-gate tooling | `.pre-commit-config.yaml` and `scripts/check-module-size.sh` when they enforce the readiness gate used by this lane. |
| Command runner support | `crates/eatme-core/src/command.rs` when GitHub snapshot collection needs bounded command execution or retry behavior. |
| Canonical scenario wording | Narrow nonclaim corrections in `assets/scenarios/eatme/*.yaml`. |
| Generated adapters | `assets/scenarios/gadugi/*.yaml` generated from canonical scenario changes. |
| PR-facing evidence | PR description or comment text that records exact-head readiness or blockers. |

Unrelated feature work, broad refactors, formatting-only churn outside touched
contracts, generated caches, build output, and local artifacts block readiness.
When the diff is unfocused, the report must be `NOT_MERGE_READY` and name the
unrelated path or change category.

Collect focused diff evidence with:

```bash
git diff --name-status origin/master...HEAD
git diff --stat origin/master...HEAD
```

Use those summaries as evidence only after verifying that the compared head is
the PR head SHA named in the recovery input.

## Documentation impact gate

Documentation impact is evaluated from the diff, not from intent. If any file
under `docs/`, `mkdocs.yml`, or documentation dependency files changes, the
recovery input must include a passing documentation build for the exact
validation SHA:

```bash
mkdocs build --strict
```

The Rust input always carries `documentation_build` evidence. If no
documentation files changed, the docs-impact evidence records
`docs_changed: false` and explains why `mkdocs build --strict` was not required.
The PR description may say "no documentation impact" only when the focused diff
audit shows no documentation changes.

Documentation must not contain point-in-time readiness claims that can become
stale. Exact-head status belongs in the PR body or a readiness comment. Docs may
describe the contract, input schema, examples, and historical stale/non-current
examples.

## PR description evidence

The recovery gate treats the PR description as part of readiness. It must be
updated or verified after the final evidence-changing commit and before
readiness is reported.

The description contains:

| Evidence | Required content |
| --- | --- |
| Exact head | The full 40-character PR head SHA that matches local `git rev-parse HEAD`. |
| Branch | The intended lane branch. |
| Local QA | Asset validation, generated Gadugi freshness, repository quality gate, and docs build when docs changed. |
| Quality audit | At least three SEEK/VALIDATE/FIX cycles with the final cycle clean. |
| GitHub Actions | Required checks complete successfully for the same SHA; skipped optional checks called skipped. |
| Diff scope | A focused-scope statement naming the categories changed. |
| Change outcome | Either `Files modified: ...` or `No-op justification: ...`. |
| Nonclaims | Explicitly states that evidence does not validate full Alice UI automation, full world execution, UI rendering correctness, visible rendering correctness, grading, creative assessment, Save completion, deployed sharing/platform success, first-lesson completion, full lesson completion, complete Alice coverage, or full Tweedle/player decode. |
| Blockers | `NOT_MERGE_READY` plus explicit blockers when any gate is missing or failing. |

Do not mark the PR ready from a comment alone when the PR description still
claims readiness for an older SHA or omits required evidence. A readiness comment
may summarize the same evidence, but the PR description remains the stable entry
point for reviewers.

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
| Full lesson completion | It is bounded recovery evidence only, not proof that a learner completed a lesson. |
| Grading or learner-world grading | It records evidence for review; it does not grade. |
| Creative assessment | It may name an editable change; it does not assess creativity. |
| Full UI automation | It records bounded launch/opened-project evidence and explicit gaps. |
| Visible rendering correctness | Screenshot or window evidence is observation evidence only. |
| Full Save completion | Save, reopen, and export remain readiness gaps until user-like evidence exists. |
| Complete Alice coverage | The scenario covers only the stated preflight contract. |
| Full Tweedle/player decode | Parser or launch evidence cannot prove complete source or player semantics. |

Use the generated adapter only as a consumer of this contract. Do not hand-edit
generated Gadugi YAML to change mission intent.

## Scenario nonclaim boundary

Canonical scenario language is user-facing specification text. Preserve it unless
the nonclaim/readiness gate requires a narrow correction. When a correction is
required, edit the canonical eatme scenario first and regenerate adapters from
that source.

Safe scenario and readiness wording can say that evidence is shown, not yet
shown, blocked, or available for review. It cannot convert partial evidence into
completion.

| Unsupported claim | Safe boundary |
| --- | --- |
| Full UI automation | Report only the specific launch, desktop, action, or artifact evidence that was observed. |
| Full world execution | Report only the bounded run or execution observation represented by evidence. |
| Visible rendering correctness | Treat screenshots and window observations as observation evidence only. |
| Grading or learner-world grading | Record evidence for review; do not assign grades. |
| Creative assessment | Surface available evidence or next steps without judging creative quality. |
| Save completion | Keep Save option, Save action, shortcut, and artifact availability separate from completion. |
| Deployed sharing or platform success | Do not infer service or platform readiness from local asset, adapter, or desktop evidence. |
| First-lesson completion | Treat first-lesson evidence as bounded scenario evidence only. |

Generated Gadugi adapters inherit this boundary. Do not repair generated wording
by editing `assets/scenarios/gadugi/*.yaml` directly.

## Generated Gadugi adapter freshness

Whenever a canonical scenario asset changes, the generated Gadugi adapter
freshness check is mandatory:

```bash
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
```

If the check reports stale or missing generated output after canonical scenario
YAML changes, regenerate adapters and run check mode again:

```bash
cargo run -q -p eatme-cli -- assets generate-gadugi --json
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
```

Commit the canonical scenario change and regenerated adapter change together.
When no scenario asset or generated adapter target is affected, adapter freshness
is part of the readiness decision only for lanes that explicitly require adapter
evidence. The wave7 nonclaim audit lane requires the check even for
documentation-only recovery; regeneration is required only if canonical scenario
YAML changes make generated output stale.

Validate committed scenario and persona assets:

```bash
cargo run -q -p eatme-cli -- assets validate --json
```

The validation gate passes only when the JSON report has `passed: true` and no
blocking errors.

## Evidence API

Default-workflow readiness evidence is a PR-facing envelope, not a new runtime
schema. A readiness comment or PR body should include these fields in prose or a
small table:

| Field | Required value |
| --- | --- |
| Branch | The intended lane branch. |
| Exact tested head | `git rev-parse HEAD` for the 40-character SHA that was validated. |
| PR head | `headRefOid` from `gh pr view`, matching the exact tested head SHA. |
| Merge state | `mergeStateStatus=CLEAN` for the exact PR head SHA. |
| Mergeability | `mergeable=MERGEABLE` for the exact PR head SHA. |
| GitHub checks | Required checks completed successfully for the exact PR head SHA; skipped optional checks may be reported as skipped, not passed. |
| Asset validation | `cargo run -q -p eatme-cli -- assets validate --json` passed at the exact tested head SHA. |
| Gadugi freshness | `cargo run -q -p eatme-cli -- assets generate-gadugi --check --json` passed at the exact tested head SHA when required by scenario changes or by the lane evidence contract. |
| Quality gate | `TMPDIR=/tmp NODE_OPTIONS=--max-old-space-size=32768 ./scripts/quality-gates.sh` passed at the exact tested head SHA, when required. |
| Documentation build | `mkdocs build --strict` passed at the exact tested head SHA, when docs changed. |
| Quality-audit cycles | At least three SEEK/VALIDATE/FIX cycles, with the final cycle clean. |
| Diff scope | Changed files are inside the focused recovery scope for the lane. |
| Docs impact | Documentation changes are either absent, or `mkdocs build --strict` passed for the exact head. |
| PR description | The PR description names the exact head and records local QA, quality cycles, checks, diff scope, change outcome, blockers or readiness, and nonclaims. |
| Stale evidence handling | Older tested-head evidence is removed, replaced, or labeled stale/non-current. |
| Claim boundary | No unsupported claims were added to scenarios, docs, PR text, or comments. |
| Change outcome | Either `Files modified: ...` lists the recovery files changed at the final head, or `No-op justification: ...` explains why the final head already satisfies readiness. |

The evidence envelope is valid only for the named head. If the branch receives a
new commit, rerun the gate and update the evidence to the new SHA.

## Recovery readiness input schema

The recovery evaluator reads JSON with schema version
`pr-readiness-recovery.v1`. Every SHA field uses a full 40-character lowercase
or uppercase hexadecimal Git object ID.

| Field | Required content |
| --- | --- |
| `schema_version` | Must be `pr-readiness-recovery.v1`. |
| `expected_remote_head_sha` | PR #204's expected remote head at validation time; `snapshot.pr_head_sha`, `snapshot.local_head_sha`, and `validation_sha` must all match it. |
| `snapshot` | PR number, branch, local head SHA, PR head SHA, merge metadata, and check summaries. |
| `validation_sha` | SHA named by every local validation item; it must equal `snapshot.local_head_sha` and `snapshot.pr_head_sha`. |
| `required_github_checks` | Trusted required check names. The evaluator fails closed when any named check is missing, skipped, pending, failing, cancelled, or reported for another head, regardless of the `required` flag in the snapshot. |
| `asset_validation`, `generated_gadugi_check`, `quality_gate`, `documentation_build` | Command evidence with the expected command, exact evidence SHA, exit status, summary, and pass/fail result. |
| `quality_audit_cycles` | At least three cycles numbered `1..n` without gaps or decreases, each with `Seek`, `Validate`, and `Fix` phases; the final cycle outcome must be `Clean`. |
| `diff_scope` | Changed paths and whether the diff is focused on the allowed recovery scope. |
| `docs_impact` | Whether docs changed and whether strict docs build evidence is required. |
| `pr_description_evidence` | Whether the PR description names the final SHA and includes readiness evidence plus bounded nonclaims. |
| `stale_evidence_handled` | `true` only after older exact-head evidence is removed, replaced, or labeled stale/non-current. |
| `wrapper_failures` | Historical wrapper failures such as `RATE_LIMIT`; these are context, not readiness evidence. |
| `change_outcome` | Either `{"FilesModified": [...]}` or `{"NoOp": {"justification": "..."}}`. |

Minimum PR #204 recovery input shape:

```json
{
  "schema_version": "pr-readiness-recovery.v1",
  "expected_remote_head_sha": "1111111111111111111111111111111111111111",
  "snapshot": {
    "pr_number": 204,
    "branch": "wave7-eatme-nonclaim-audit-1778303500",
    "local_head_sha": "1111111111111111111111111111111111111111",
    "pr_head_sha": "1111111111111111111111111111111111111111",
    "merge_state_status": "CLEAN",
    "mergeable": "MERGEABLE",
    "checks": [
      {
        "name": "quality-gates",
        "status": "Completed",
        "conclusion": "Success",
        "required": true,
        "head_sha": "1111111111111111111111111111111111111111"
      },
      {
        "name": "optional-preview",
        "status": "Completed",
        "conclusion": "Skipped",
        "required": false,
        "head_sha": "1111111111111111111111111111111111111111"
      }
    ]
  },
  "validation_sha": "1111111111111111111111111111111111111111",
  "required_github_checks": ["quality-gates"],
  "asset_validation": {
    "name": "asset validation",
    "command": "cargo run -q -p eatme-cli -- assets validate --json",
    "evidence_sha": "1111111111111111111111111111111111111111",
    "exit_status": 0,
    "summary": "Scenario and persona asset validation passed.",
    "passed": true
  },
  "generated_gadugi_check": {
    "name": "generated Gadugi freshness",
    "command": "cargo run -q -p eatme-cli -- assets generate-gadugi --check --json",
    "evidence_sha": "1111111111111111111111111111111111111111",
    "exit_status": 0,
    "summary": "Generated Gadugi adapters are fresh.",
    "passed": true
  },
  "quality_gate": {
    "name": "repository quality gates",
    "command": "TMPDIR=/tmp NODE_OPTIONS=--max-old-space-size=32768 ./scripts/quality-gates.sh",
    "evidence_sha": "1111111111111111111111111111111111111111",
    "exit_status": 0,
    "summary": "Repository quality gate passed.",
    "passed": true
  },
  "documentation_build": {
    "name": "documentation build",
    "command": "mkdocs build --strict",
    "evidence_sha": "1111111111111111111111111111111111111111",
    "exit_status": 0,
    "summary": "Documentation build passed because docs changed.",
    "passed": true
  },
  "quality_audit_cycles": [
    {
      "cycle_number": 1,
      "phases": ["Seek", "Validate", "Fix"],
      "outcome": "FixApplied",
      "head_sha": "1111111111111111111111111111111111111111",
      "summary": "Stale or wrong-head readiness evidence was found and fixed."
    },
    {
      "cycle_number": 2,
      "phases": ["Seek", "Validate", "Fix"],
      "outcome": "FixApplied",
      "head_sha": "1111111111111111111111111111111111111111",
      "summary": "Unsupported claims were reviewed and kept as explicit nonclaims."
    },
    {
      "cycle_number": 3,
      "phases": ["Seek", "Validate", "Fix"],
      "outcome": "Clean",
      "head_sha": "1111111111111111111111111111111111111111",
      "summary": "Final evidence was complete and no further fix was required."
    }
  ],
  "diff_scope": {
    "changed_files": ["docs/default-workflow-pr-readiness.md"],
    "focused": true
  },
  "docs_impact": {
    "docs_changed": true,
    "strict_build_required": true
  },
  "pr_description_evidence": {
    "head_sha": "1111111111111111111111111111111111111111",
    "contains_readiness_evidence": true,
    "contains_bounded_nonclaims": true
  },
  "stale_evidence_handled": true,
  "wrapper_failures": ["RATE_LIMIT"],
  "change_outcome": {
    "FilesModified": ["docs/default-workflow-pr-readiness.md"]
  }
}
```

## Recovery report output

The plain-text report starts with a status line:

```text
MERGE_READY
```

or:

```text
NOT_MERGE_READY
```

`MERGE_READY` output includes only evidence present in the report model: branch,
expected remote head, final head, validation status, exactly one change outcome,
trusted required-check names, GitHub check summaries, local QA summaries, quality
audit cycles, diff scope, docs impact, PR-description evidence flags, and
historical wrapper-failure context. The renderer normalizes control characters
from caller-controlled text and redacts obvious token/secret patterns before
printing.

`NOT_MERGE_READY` output will include explicit blockers. The report must not
hide missing criteria behind a partial success statement. Example:

```text
NOT_MERGE_READY

Blockers:
- quality_audit_cycles has 2 cycles; at least 3 are required
- final quality-audit cycle is not clean
- PR description does not name the exact final head SHA
```

JSON output uses the same status values:

```json
{
  "status": "NOT_MERGE_READY",
  "expected_remote_head_sha": "1111111111111111111111111111111111111111",
  "final_head_sha": "1111111111111111111111111111111111111111",
  "validation_status": "blocked for exact current HEAD 1111111111111111111111111111111111111111",
  "required_github_checks": ["quality-gates"],
  "github_checks": [],
  "qa_evidence": [],
  "quality_audit_cycles": [],
  "blockers": [
    "required GitHub Actions check quality-gates is missing or omitted at 1111111111111111111111111111111111111111"
  ]
}
```

Do not convert `NOT_MERGE_READY` to a successful exit in automation. A missing
or failing gate must block readiness until the evidence is collected again for
the final PR head.

## No-op and files-modified evidence

Every recovery report has an explicit change outcome.

No-op is preferred when the final PR head already validates without repository
changes. It keeps readiness recovery focused on evidence instead of creating a
commit solely to satisfy the wrapper.

Use `Files modified: ...` when the recovery changed documentation, canonical
scenario assets, generated adapters, PR text, or other readiness evidence files.
The list names only files changed for the recovery lane. It must not include
unrelated local files, generated build output, caches, or files changed by
another lane.

If a design artifact says `files_to_change: []`, treat that as no repository
change for the readiness recovery itself. A documentation-retcon commit is a
separate documentation change and must be reported with
`Files modified: docs/default-workflow-pr-readiness.md`.

Use `No-op justification: ...` only when the final PR head already satisfies the
readiness contract without additional repository changes. A valid no-op
justification names the final branch, the final 40-character HEAD, and the fresh
validation commands that passed after that HEAD was selected. It also states why
no scenario, generated adapter, documentation, or readiness-evidence file needed
to change.

No-op is not valid when it is based on:

| Invalid basis | Required response |
| --- | --- |
| Earlier validation for a different SHA | Rerun validation after resolving the final head. |
| Wrapper rate-limit, wrapper no-op guard, or missing wrapper output | Run the manual wrapper bypass and record current-head evidence. |
| Assumed generated adapter freshness | Run `assets generate-gadugi --check --json`. |
| Dirty worktree or unrelated edits | Separate or remove unrelated changes before reporting readiness. |
| Branch checked out from `master` instead of the PR head branch | Switch to the PR head branch and restart evidence collection. |

## Wave7 nonclaim audit example

This example describes the wave7 recovery lane behavior. Replace the PR number
and SHA with the values from the actual PR.

```bash
git fetch origin --prune
if git ls-remote --exit-code --heads origin \
  wave7-eatme-nonclaim-audit-1778303500 >/dev/null; then
  if git show-ref --verify --quiet \
    refs/heads/wave7-eatme-nonclaim-audit-1778303500; then
    git switch wave7-eatme-nonclaim-audit-1778303500
    git branch --set-upstream-to=origin/wave7-eatme-nonclaim-audit-1778303500
    git merge-base --is-ancestor HEAD \
      origin/wave7-eatme-nonclaim-audit-1778303500
    git merge-base --is-ancestor \
      origin/wave7-eatme-nonclaim-audit-1778303500 HEAD
  else
    git switch --track origin/wave7-eatme-nonclaim-audit-1778303500
  fi
elif git show-ref --verify --quiet \
  refs/heads/wave7-eatme-nonclaim-audit-1778303500; then
  git switch wave7-eatme-nonclaim-audit-1778303500
  git merge-base --is-ancestor origin/master HEAD
else
  git switch -c wave7-eatme-nonclaim-audit-1778303500 origin/master
fi

export NODE_OPTIONS=--max-old-space-size=32768
cargo run -q -p eatme-cli -- assets validate --json
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
TMPDIR=/tmp NODE_OPTIONS=--max-old-space-size=32768 ./scripts/quality-gates.sh
mkdocs build --strict
git rev-parse HEAD
```

After pushing the branch, open or update the PR for
`wave7-eatme-nonclaim-audit-1778303500`. The PR description and readiness
comment should say what was validated at the exact head and should keep these
nonclaims explicit:

```text
This readiness evidence is bounded to asset validation, generated Gadugi
freshness, repository quality gates, GitHub checks, mergeability, and exact-head
verification for <sha>.

It does not claim full Alice UI automation, full world execution, UI rendering
correctness, visible rendering correctness, grading, creative assessment, Save
completion, deployed sharing or platform success, first-lesson completion, full
lesson completion, or full Tweedle/player decode.
```

## PR #204 nonclaim audit readiness record

PR #204 uses the wave7 nonclaim audit lane:

```text
wave7-eatme-nonclaim-audit-1778303500
```

The finished readiness record for PR #204 is a PR-facing note, not a committed
claim that a documentation file can keep current by itself. The note is posted
after the final documentation/evidence commit, then the PR head is queried again
to confirm that the note's exact SHA still equals `headRefOid`.

The PR #204 readiness note contains this bounded evidence:

| Evidence item | Required wording |
| --- | --- |
| Branch | Name `wave7-eatme-nonclaim-audit-1778303500`. |
| Exact tested head | Name the full 40-character SHA from `git rev-parse HEAD`. |
| Exact PR head | Name the full 40-character SHA from `gh pr view 204 --json headRefOid`; it must match the exact tested head. |
| Local checks | Name each repository check that passed at that SHA: asset validation, generated Gadugi freshness, repository quality gates, and `mkdocs build --strict` when docs changed. |
| Quality audit | Name at least three SEEK/VALIDATE/FIX cycles and state that the final cycle was clean. |
| GitHub checks | Summarize `statusCheckRollup` for that same SHA; required successful checks may be called successful or green, skipped optional checks are called skipped. |
| Merge metadata | Name `mergeStateStatus=CLEAN` and `mergeable=MERGEABLE` only when reported for that same SHA. |
| Diff scope | State that every changed file is focused on readiness recovery, docs, tests, uvx packaging, quality-gate tooling, command runner support, canonical scenario wording, generated adapters, or PR evidence as applicable. Include the complete changed-file list from `git diff --name-status origin/master...HEAD`; do not summarize a subset as the full scope. |
| PR description | State that the PR description itself contains exact-head readiness evidence or explicit `NOT_MERGE_READY` blockers. |
| Stale evidence | State that older tested-head evidence was removed, replaced, or labeled stale/non-current and is not current validation. |
| Nonclaims | State that the evidence does not validate full Alice UI automation, full world execution, grading, creative assessment, UI rendering correctness, visible rendering correctness, Save completion, deployed sharing/platform success, first-lesson completion, full lesson completion, complete Alice coverage, or full Tweedle/player decode. |

Use this comment body shape after the last commit on PR #204:

```text
Default-workflow readiness recorded for PR #204.

Exact SHA: <40-character PR head SHA>
Branch: wave7-eatme-nonclaim-audit-1778303500

Verified for this exact SHA:
- local HEAD equals PR head
- asset validation passed
- generated Gadugi freshness passed
- repository quality gates passed
- documentation build passed
- at least three quality-audit SEEK/VALIDATE/FIX cycles completed, with the
  final cycle clean
- GitHub checks are complete for this SHA, with optional skipped checks reported
  as skipped
- mergeStateStatus=CLEAN and mergeable=MERGEABLE
- focused diff scope verified
- PR description evidence names this exact SHA and includes local QA, GitHub
  Actions, quality-audit cycles, diff scope, change outcome, and nonclaims
- older tested-head evidence is stale/non-current and is not presented as
  current validation
- Files modified: <complete changed-file list from git diff --name-status origin/master...HEAD>
  OR
  No-op justification: <why this exact head already satisfies readiness without
  repository changes>

Nonclaims: this does not validate full Alice UI automation, full world execution,
grading, creative assessment, UI rendering correctness, visible rendering
correctness, Save completion, deployed sharing/platform success,
first-lesson completion, full lesson completion, complete Alice coverage, or
full Tweedle/player decode.
```

If a new commit appears after this comment is posted, the comment is
stale/non-current and is superseded by the next exact-head readiness note. Rerun
the exact-head gate, post or replace the evidence for the new SHA, and perform
the final PR head query again.

## Historical stale/non-current PR #164 readiness example

This subsection is a historical stale/non-current example for the PR #164
finalization gate. It is not current validation for any later PR. Do not reuse
its PR number or SHA for future readiness decisions.

For PR #164, the exact accepted head is:

```text
eb0bb29b7cc1f8647e9a36c0bc8200fb3fdc5cba
```

The historical GitHub metadata gate passed only when `gh pr view 164 --json
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

The readiness decision for PR #164 was valid only for
`eb0bb29b7cc1f8647e9a36c0bc8200fb3fdc5cba` when those commands passed, the
required GitHub checks completed successfully for that exact SHA, optional
skipped checks were reported only as skipped, and the scenario wording stayed
within the starter-project evidence boundary above. It is stale/non-current for
PR #204 and for any later head.

## Readiness comment

Publish readiness only after all required gates pass for the exact head. The
comment should name the head and avoid broader product-readiness claims.

Example:

```text
Default-workflow readiness recorded for PR <pr-number> at exact head <sha>.

Verified gates: exact PR head, required GitHub checks completed successfully for that head with optional skipped checks reported only as skipped, mergeStateStatus=CLEAN, mergeable=MERGEABLE, bounded scenario/readiness wording, no unsupported claims for first-lesson completion/full lesson completion/grading/creative assessment/full Alice UI automation/full world execution/UI rendering correctness/visible rendering correctness/Save completion/deployed sharing or platform success/complete Alice coverage/full Tweedle/player decode, stale tested-head evidence replaced or labeled stale/non-current, asset validation, generated Gadugi adapter freshness when required by the lane, and either Files modified or No-op justification.

The prior non-zero wrapper exit is not treated as a blocker because direct verification passed at this exact head.
```

Post the comment with:

```bash
gh pr comment <pr-number> --body-file readiness-comment.txt
```

## Blocker handling

If any gate fails, do not publish readiness. Fix only the minimal issue that
caused the blocker, run the directly relevant validation before committing when
that helps confirm the fix, push the fix, and then rerun every final required
validation for the new PR head. A fix commit invalidates all earlier final
evidence, including checks that were unrelated to the blocker.

| Blocker | Minimal response |
| --- | --- |
| Head mismatch | Stop readiness for the old SHA and verify the newly observed PR head as the expected evidence SHA. |
| Failing, pending, cancelled, missing, or wrong-head checks | Fix the failing check, wait for completion, or rerun the missing check before readiness. |
| Dirty merge state | Resolve only the mergeability issue. |
| Overclaiming scenario language | Edit the canonical scenario wording and regenerate adapters if affected. |
| Stale generated adapter | Regenerate adapters from canonical sources. |
| Asset validation failure | Fix the invalid scenario or persona asset. |
| Unrelated changes | Remove the unrelated change from the readiness work. |
