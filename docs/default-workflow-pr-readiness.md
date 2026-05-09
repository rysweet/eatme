# Default-workflow PR readiness

Default-workflow PR readiness is the exact-head recovery gate used when a pull
request needs a bounded merge-readiness decision and an owner-free wrapper exit,
including `NO_OP_GUARD`, did not produce usable evidence.

The workflow treats `NO_OP_GUARD` as a recovery trigger, not as success. It
fetches the PR branch or PR ref, checks out the exact GitHub `headRefOid` in
detached mode, runs mandatory repository evidence without timeout wrappers,
records at least three SEEK/VALIDATE/FIX audit cycles, and renders either
`MERGE_READY` or `NOT_MERGE_READY`.

GitHub Actions being green is necessary but not sufficient. A ready decision
also requires non-draft mergeable PR metadata, runnable repository evidence,
documentation-impact review, focused diff scope, PR description evidence, and a
clean final quality-audit cycle.

## Contents

- [Readiness contract](#readiness-contract)
- [Usage](#usage)
- [Configuration](#configuration)
- [Inputs and outputs](#inputs-and-outputs)
- [Head alignment gate](#head-alignment-gate)
- [PR metadata auditor](#pr-metadata-auditor)
- [GitHub Actions auditor](#github-actions-auditor)
- [Runnable evidence runner](#runnable-evidence-runner)
- [Quality audit cycles](#quality-audit-cycles)
- [Focused diff auditor](#focused-diff-auditor)
- [Docs impact auditor](#docs-impact-auditor)
- [PR description evidence auditor](#pr-description-evidence-auditor)
- [No-op justification](#no-op-justification)
- [Decision rendering](#decision-rendering)
- [PR #203 recovery example](#pr-203-recovery-example)
- [Claim boundaries](#claim-boundaries)
- [Troubleshooting blockers](#troubleshooting-blockers)

## Readiness contract

A pull request is default-workflow ready only when every gate passes for the
exact commit being reviewed.

| Gate | Required result |
| --- | --- |
| Exact head | Local `HEAD` equals the PR `headRefOid` fetched from GitHub. |
| Manual merge avoidance | The workflow fetches the PR branch or PR ref, then detaches at the exact `headRefOid` without merging into the base branch. |
| GitHub Actions | All required checks and relevant workflow runs are complete and green for `headRefOid`. |
| PR state | The PR is open, non-draft, mergeable, and has a clean merge state. |
| Runnable evidence | Required repository QA, scenario, generated-asset, and docs commands pass without timeout wrappers. |
| Quality audit | At least three SEEK/VALIDATE/FIX cycles are documented, and the final cycle is clean. |
| Diff scope | The PR diff is focused and contains no unrelated churn or accidental generated artifacts. |
| Docs impact | Documentation impact is reviewed, the mandatory docs build passes, and affected docs are updated or explicitly ruled out. |
| PR description | The PR body contains current, bounded evidence for the same head and does not overclaim readiness. |

If any gate is missing, stale, inconclusive, skipped when required, or tied to a
different commit, the decision is `NOT_MERGE_READY`.

## Usage

Run the recovery from the repository root with authenticated `gh` access. Do not
wrap any command in `timeout`, `gtimeout`, shell watchdogs, background-kill
helpers, or workflow timeout wrappers.

1. Export the repository workflow heap setting:

   ```bash
   export NODE_OPTIONS=--max-old-space-size=32768
   ```

2. Read the PR head from GitHub:

   ```bash
   gh pr view 203 --json headRefOid,headRefName,state,isDraft,mergeable,mergeStateStatus,statusCheckRollup,body
   ```

3. Fetch the PR ref or branch, then check out the exact PR head SHA without
   merging:

   ```bash
   HEAD_REF_OID="$(gh pr view 203 --json headRefOid --jq .headRefOid)"
   git fetch origin pull/203/head
   git switch --detach "$HEAD_REF_OID"
   git rev-parse HEAD
   ```

4. Compare `git rev-parse HEAD` with `headRefOid`. A mismatch stops the run.

5. Run all mandatory repository evidence commands:

   ```bash
   TMPDIR=/tmp ./scripts/quality-gates.sh
   cargo run -q -p eatme-cli -- assets validate --json
   cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
   mkdocs build --strict
   ```

6. Record at least three SEEK/VALIDATE/FIX cycles. The final cycle must find no
   remaining blocker.

7. Render `MERGE_READY` only when every gate passes. Otherwise render
   `NOT_MERGE_READY` with explicit blockers.

## Configuration

| Setting | Required value | Purpose |
| --- | --- | --- |
| `NODE_OPTIONS` | `--max-old-space-size=32768` | Keeps Node-based workflow wrappers on the repository's saved large-heap setting. |
| `TMPDIR` | `/tmp` for deep worktrees | Keeps temporary socket paths short for the full repository quality gate. |
| `gh` authentication | Authenticated to `rysweet/eatme` | Allows PR metadata, checks, diff, and body evidence to be audited. |

The workflow does not require a timeout wrapper. Long-running gates are allowed
to run to completion through their normal command behavior.

Do not print or persist tokens, credential paths, environment dumps, private
runner details, or other secrets in readiness evidence.

## Inputs and outputs

| Surface | Contract |
| --- | --- |
| PR number | The pull request being recovered, for example `203`. |
| Head branch | The current remote PR branch, not a local merge result. |
| Required evidence | PR metadata, exact-head checks, runnable command results, diff scope, docs impact, PR body evidence, and audit cycles. |
| Success output | `MERGE_READY` plus exact-head evidence and bounded gate list. |
| Failure output | `NOT_MERGE_READY` plus explicit blockers. |
| Repository changes | Focused fixes only. If files change, report `Files modified`. |
| No repository changes | Include a workflow-accepted no-op justification tied to current head, checks, and blockers or evidence. |

The CLI can render a structured decision from collected evidence:

```bash
cargo run -q -p eatme-cli -- default-workflow pr-readiness \
  --evidence readiness-evidence.json \
  --json
```

The evidence file is a JSON record of the documented command sequence. The CLI
does not run repository gates on its own; command evidence must be collected
first so every claim remains tied to the exact PR head.

GitHub and git evidence can be collected through the external-service adapter:

```bash
cargo run -q -p eatme-cli -- default-workflow collect-github-evidence \
  --pr 203 \
  --json
```

Add `--checkout` only when the worktree is ready to detach at the exact PR head.
The adapter calls `gh pr view`, `gh pr diff`, `gh run list`, `git fetch`,
`git rev-parse`, and `git status --short` with bounded retries and reports
external-call failures as hard errors instead of readiness success.

## Head alignment gate

The head alignment gate proves that local evidence is about the same commit that
GitHub will evaluate.

Required behavior:

1. Read the GitHub `headRefOid`.
2. Fetch the PR branch or PR ref from `origin` so the head object is available.
3. Check out the exact `headRefOid` in detached mode.
4. Compare `git rev-parse HEAD` with `headRefOid`.
5. Stop with `NOT_MERGE_READY` if the SHAs differ.

Detached checkout is acceptable. Manual merging, rebasing, force-pushing, and
history rewriting are not part of the readiness workflow.

## PR metadata auditor

The metadata auditor reads:

```bash
gh pr view 203 \
  --json state,isDraft,mergeable,mergeStateStatus,headRefOid,headRefName,body
```

The CLI service adapter uses the same metadata request with `statusCheckRollup`
included and records the returned body separately from the readiness decision.

The PR metadata gate passes only when:

| Field | Required value |
| --- | --- |
| `state` | `OPEN` |
| `isDraft` | `false` |
| `mergeable` | `MERGEABLE` |
| `mergeStateStatus` | `CLEAN` |
| `headRefOid` | Equal to local `HEAD` |

Any unknown, dirty, blocked, draft, closed, stale, or mismatched state is a
`NOT_MERGE_READY` blocker.

## GitHub Actions auditor

The Actions auditor verifies check conclusions for the exact PR head:

```bash
gh pr view 203 --json headRefOid,statusCheckRollup
```

The check gate is green only when every required check for `headRefOid` has
completed successfully. The gate blocks readiness when a required or relevant
check is pending, queued, in progress, requested, failing, errored, timed out,
cancelled, skipped when it is expected to run, missing, or reported for a
different SHA.

Known skipped status-rollup entries from optional conditional jobs are retained
in collected evidence but do not count as required checks. Other skipped entries
remain required-check blockers unless explicitly classified as optional
conditional jobs. Skipped optional entries also do not satisfy the required-check
evidence minimum; at least one required exact-head check must still complete
successfully.

When a workflow run is needed to distinguish stale from exact-head checks, query
workflow runs by branch and head SHA:

```bash
gh run list \
  --branch feat/issue-177-eatme-wave7-formalspec-contract-lane-follow-defaul \
  --json databaseId,headSha,status,conclusion,workflowName
```

The external adapter treats missing, malformed, or non-zero `gh` responses as
service-call failures. It does not convert unavailable GitHub evidence into a
passing check.

Green checks alone never produce `MERGE_READY`; they only satisfy the Actions
gate.

## Runnable evidence runner

The recovery runs all mandatory existing repository commands. It does not add new
tools or use timeout wrappers.

| Evidence | Command |
| --- | --- |
| Full repository quality gate | `TMPDIR=/tmp ./scripts/quality-gates.sh` |
| Persona and scenario asset validation | `cargo run -q -p eatme-cli -- assets validate --json` |
| Generated Gadugi adapter freshness | `cargo run -q -p eatme-cli -- assets generate-gadugi --check --json` |
| Documentation build | `mkdocs build --strict` |

All four commands are mandatory for this recovery lane, regardless of PR scope.
Scenario-specific manual gates, real-Alice smoke gates, and other focused checks
remain additional evidence when the PR scope requires them.

If a host cannot run an applicable real-Alice or scenario-specific manual gate,
the decision must say that the gate is unavailable or not run. Do not convert a
missing manual gate into a passing automated claim.

## Quality audit cycles

The quality audit recorder documents at least three cycles. Each cycle has three
parts:

| Step | Meaning |
| --- | --- |
| SEEK | Identify one evidence gap, risk, stale claim, or possible defect in the recovery surface. |
| VALIDATE | Prove the finding with a command, metadata query, diff review, docs review, or bounded inspection. |
| FIX | Apply a focused fix when a validated defect exists, or record `no repository change` when validation shows no fix is required. |

The final cycle must be clean: SEEK finds no remaining blocker after the
previous fixes or no-op validations, VALIDATE confirms all gates are still tied
to the exact head, and FIX records `no repository change`.

Example audit-cycle record:

```text
Cycle 1
SEEK: Check exact-head alignment after NO_OP_GUARD.
VALIDATE: local HEAD equals gh pr view headRefOid.
FIX: no repository change.

Cycle 2
SEEK: Check runnable QA and generated adapter evidence.
VALIDATE: quality-gates, assets validate, generate-gadugi --check, and mkdocs build pass.
FIX: no repository change.

Cycle 3
SEEK: Check final metadata, PR body evidence, and diff scope.
VALIDATE: PR is open, non-draft, mergeable, checks are green for headRefOid, diff is focused, body evidence is bounded.
FIX: no repository change; final cycle clean.
```

If a cycle validates a defect, make only the focused PR-scoped fix and rerun the
impacted evidence before the next cycle.

## Focused diff auditor

The diff auditor checks the PR scope, not just local status:

```bash
gh pr diff 203 --name-only
git status --short
git diff --stat
```

The gate passes when changed files match the PR purpose and no accidental local
files, generated artifacts, logs, reports, temporary files, or unrelated cleanup
are included.

Generated assets are acceptable only when they are the expected output of a
canonical asset change and the freshness check passes.

## Docs impact auditor

Docs impact is explicit:

| PR scope | Required docs result |
| --- | --- |
| Docs changed | `mkdocs build --strict` passes. |
| Scenario or adapter behavior changed | Affected docs are updated or the no-impact rationale explains why existing docs remain accurate. |
| PR body cites docs evidence | The cited docs exist and describe only proven behavior. |
| No docs impact | The decision records a bounded no-impact rationale. |

Docs should not contain point-in-time CI logs, status reports, progress notes, or
merge claims. Those belong in PR comments, PR descriptions, or workflow logs.

## PR description evidence auditor

The PR body must contain current, bounded evidence for the exact head before the
workflow can render `MERGE_READY`.

Required PR description evidence:

| Evidence | Required content |
| --- | --- |
| Exact head | The reviewed SHA or a clear statement that evidence is for the current `headRefOid`. |
| GitHub checks | Green exact-head checks or a blocker if checks are missing or incomplete. |
| Runnable QA | All mandatory commands run and pass, or each command that did not run or pass is listed as a blocker. |
| Scenario evidence | Asset, Gadugi, and scenario-specific evidence where applicable. |
| Docs impact | Docs changed and built, or no-impact rationale. |
| Audit cycles | At least three SEEK/VALIDATE/FIX cycles with a clean final cycle. |
| Claim boundaries | No unsupported claims about UI automation, rendering correctness, grading, creative assessment, full lesson completion, or full Tweedle/player decode. |

If the body is stale or overclaims, update it with bounded evidence before
readiness is rendered. If the body cannot be updated, render `NOT_MERGE_READY`
with a PR-description blocker.

## No-op justification

When recovery finds no repository defect, the workflow emits an explicit
workflow-accepted no-op justification instead of treating `NO_OP_GUARD` as
success.

The no-op justification includes:

```text
No-op justification: workflow accepted no repository changes because local HEAD
matches PR headRefOid, exact-head GitHub checks and runnable evidence satisfy the
readiness gates, diff scope is focused, docs impact is accounted for, PR body
evidence is current and bounded, and three SEEK/VALIDATE/FIX cycles completed
with a clean final cycle.
```

If any evidence is missing, the no-op justification is still allowed, but the
decision is `NOT_MERGE_READY` and the missing evidence is listed as a blocker.

## Decision rendering

Render one of two decisions.

Use `MERGE_READY` only when every required gate passes:

```text
MERGE_READY

PR: #203
Head: <headRefOid>
Evidence: exact-head checkout, green GitHub Actions for the same head,
non-draft mergeable PR metadata, runnable QA/scenario/docs evidence, focused
diff scope, current PR description evidence, and three SEEK/VALIDATE/FIX cycles
with a clean final cycle.
Files modified: <list, or "none">
```

Use `NOT_MERGE_READY` when any gate is missing or failed:

```text
NOT_MERGE_READY

PR: #203
Head: <headRefOid or "unverified">
Blockers:
1. <explicit blocker>
2. <explicit blocker>
No-op justification: <include when no repository files changed>
Files modified: <list, or "none">
```

Do not publish or claim readiness when the decision is `NOT_MERGE_READY`.

## PR #203 recovery example

This example shows how the workflow recovers PR #203 on branch
`feat/issue-177-eatme-wave7-formalspec-contract-lane-follow-defaul` without a
manual merge.

```bash
export NODE_OPTIONS=--max-old-space-size=32768

gh pr view 203 \
  --json headRefOid,headRefName,state,isDraft,mergeable,mergeStateStatus,statusCheckRollup,body

HEAD_REF_OID="$(gh pr view 203 --json headRefOid --jq .headRefOid)"
git fetch origin pull/203/head
git switch --detach "$HEAD_REF_OID"
git rev-parse HEAD

TMPDIR=/tmp ./scripts/quality-gates.sh
cargo run -q -p eatme-cli -- assets validate --json
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
mkdocs build --strict

gh pr diff 203 --name-only
gh run list \
  --branch feat/issue-177-eatme-wave7-formalspec-contract-lane-follow-defaul \
  --json databaseId,headSha,status,conclusion,workflowName
```

The example produces `MERGE_READY` only if the checked-out SHA equals
`headRefOid`, all exact-head checks are green, metadata is non-draft and
mergeable, command evidence passes, the diff is focused, docs impact is
accounted for, the PR description contains bounded evidence, and the third audit
cycle is clean.

## Claim boundaries

The recovery decision must stay inside the evidence collected. It must not claim:

| Unsupported claim | Required wording |
| --- | --- |
| Full UI automation | Say only which repository or scenario evidence ran. |
| Visible rendering correctness | Treat screenshots or windows as observation evidence only. |
| Grading correctness | Say evidence was recorded for review; do not say learner work was graded. |
| Creative assessment | Say prompts or artifacts are available for review; do not say creativity was assessed. |
| Full lesson completion | Say only the bounded lesson or readiness evidence that ran. |
| Full Tweedle/player decode | Say only which asset, adapter, or harness checks passed. |

### Starter-project evidence boundary

Starter-project preflight evidence is a bounded artifact trail, not complete PR
readiness. The executable starter-project boundary check scans
`docs/starter-project-preflight-evidence.md`, the source scenario, and the
generated Gadugi adapter against this contract in
`docs/default-workflow-pr-readiness.md`.

### Executable starter-project boundary check

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

## Troubleshooting blockers

| Blocker | Response |
| --- | --- |
| `NO_OP_GUARD` exit | Run recovery validation. Do not classify the exit as success. |
| Head mismatch | Fetch the current PR head and restart exact-head verification. |
| Draft PR | Render `NOT_MERGE_READY` until the PR is non-draft. |
| Unknown mergeability | Wait for GitHub to compute mergeability, then re-query metadata. |
| Dirty merge state | Render `NOT_MERGE_READY`; do not manually merge. |
| Pending, missing, skipped, stale, or failing checks | Render `NOT_MERGE_READY` until exact-head checks are complete and green. |
| Failed repository command | Make a focused fix if the failure is PR-scoped, then rerun impacted evidence. |
| Missing scenario evidence | Run the applicable asset, Gadugi, or scenario gate; otherwise list the missing evidence as a blocker. |
| Stale PR body | Update the body with current bounded evidence or block readiness. |
| Unrelated diff | Remove only the unrelated change or block readiness. |
