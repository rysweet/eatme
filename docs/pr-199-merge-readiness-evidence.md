# PR #199 merge-readiness evidence

This page records the scoped recovery evidence for PR #199 only.

## Contents

- [Scope](#scope)
- [Configuration](#configuration)
- [Authoritative PR state](#authoritative-pr-state)
- [GitHub check state](#github-check-state)
- [Repository QA](#repository-qa)
- [Default-workflow evidence boundary](#default-workflow-evidence-boundary)
- [Structured blockers](#structured-blockers)
- [Evidence API](#evidence-api)
- [Usage](#usage)
- [Conclusion](#conclusion)

## Scope

This evidence is limited to PR #199 recovery and merge-readiness review. It does
not change product behavior, invent original Alice action evidence, or make
claims about unrelated pull requests.

The authoritative PR under review is:

| Field | Value |
| --- | --- |
| PR | <https://github.com/rysweet/eatme/pull/199> |
| Base branch | `master` |
| Head branch | `feat/issue-184-eatme-wave7-original-evidence-preservation-lane-fo` |
| Checked PR head | `6f815a58077a622685a10f3ac68d16b36dc5d332` |
| Checked at | `2026-05-09T18:40Z` |

## Configuration

Use the saved large-heap Node option before invoking Node-backed workflow
wrappers:

```bash
export NODE_OPTIONS=--max-old-space-size=32768
```

Rust validation, Gadugi generation, MkDocs, and the repository quality gate can
run with the same environment. Do not record tokens, secrets, credential paths,
raw environment dumps, or unnecessary raw logs in PR readiness evidence.

## Authoritative PR state

`gh pr view 199 --json headRefName,headRefOid,mergeStateStatus,mergeable,statusCheckRollup,url,title,baseRefName,isDraft`
reported this state for the checked PR head:

| Field | Value |
| --- | --- |
| `headRefOid` | `6f815a58077a622685a10f3ac68d16b36dc5d332` |
| `mergeStateStatus` | `CLEAN` |
| `mergeable` | `MERGEABLE` |
| `isDraft` | `false` |

If `headRefOid` changes, this evidence is stale. Re-run the PR metadata check,
all required QA, and blocker review for the new head before making a
merge-readiness claim.

## GitHub check state

The GitHub check rollup for `6f815a58077a622685a10f3ac68d16b36dc5d332` reported:

| Workflow | Check | Status | Conclusion |
| --- | --- | --- | --- |
| Documentation Site | Build MkDocs site | `COMPLETED` | `SUCCESS` |
| Documentation Site | Deploy to GitHub Pages | `COMPLETED` | `SKIPPED` |
| Quality Gates | detect changed files | `COMPLETED` | `SUCCESS` |
| Quality Gates | fmt, clippy, module size | `COMPLETED` | `SUCCESS` |
| Quality Gates | tests | `COMPLETED` | `SUCCESS` |
| Quality Gates | coverage | `COMPLETED` | `SUCCESS` |
| Quality Gates | fmt, clippy, tests, module size, coverage | `COMPLETED` | `SUCCESS` |
| Quality Gates | manual real Alice launch smoke | `COMPLETED` | `SKIPPED` |
| Security | GitGuardian Security Checks | `COMPLETED` | `SUCCESS` |

Skipped checks are not evidence that the skipped work ran. `Deploy to GitHub
Pages` being skipped is not deployment evidence, and `manual real Alice launch
smoke` being skipped does not provide original Alice action evidence.

## Repository QA

The required repository QA was run at PR head
`6f815a58077a622685a10f3ac68d16b36dc5d332`:

| Command | Result |
| --- | --- |
| `cargo test --workspace --all-features` | Pass |
| `cargo run -q -p eatme-cli -- assets validate --json` | Pass |
| `cargo run -q -p eatme-cli -- assets generate-gadugi --check --json` | Pass |
| `mkdocs build --strict` | Pass |
| `TMPDIR=/tmp ./scripts/quality-gates.sh` | Pass |

These commands prove the repository validation state for the checked head. They
do not prove original Alice action evidence that was not produced by a real
Alice run.

## Default-workflow evidence boundary

The prior `default-workflow-attempt.log` is not accepted as successful
default-workflow evidence. It must not be cited as proof that the real workflow
completed.

Valid PR #199 readiness evidence must come from current PR metadata, current
checks for the exact head, required repository QA, and real evidence artifacts.
Timeout/manual-fallback artifacts do not clear blockers and do not convert
missing Alice evidence into available evidence.

## Structured blockers

Missing original Alice action evidence remains structured blocker evidence until
real evidence exists. Preserve it with the stable blocker code
`missing_real_action_evidence`; do not infer, soften, or replace it with a
success state.

```json
{
  "code": "missing_real_action_evidence",
  "action": "save-project",
  "status": "missing",
  "summary": "Original Alice action evidence is missing.",
  "detail": "Original Alice action evidence was not found in the comparison target evidence."
}
```

This blocker means the recovery evidence must not claim Save completion, full
Alice UI automation, first-lesson completion, grading, creative assessment, or
visible rendering correctness. If a later real Alice run produces action
evidence, update the blocker only from that real artifact and bind the update to
the new PR head and check state.

## Evidence API

Automation consumers should preserve the blocker inside target-local evidence
and summarize it through `original_alice_action_evidence`:

```json
{
  "original_alice_action_evidence": {
    "status": "missing",
    "summary": "Original Alice action evidence is missing.",
    "detail": "Original Alice action evidence was not found in the comparison target evidence."
  },
  "target_evidence": [
    {
      "blockers": [
        {
          "code": "missing_real_action_evidence",
          "action": "save-project"
        }
      ]
    }
  ]
}
```

`available` only means no `missing_real_action_evidence` blocker was found in
the inspected evidence. It does not prove Save completion, full UI automation,
first-lesson completion, grading, creative assessment, or visible rendering
correctness.

## Usage

Collect current PR state:

```bash
gh pr view 199 \
  --json headRefName,headRefOid,mergeStateStatus,mergeable,statusCheckRollup,url,title,baseRefName,isDraft
```

Run required QA from the repository root at the checked PR head:

```bash
cargo test --workspace --all-features
cargo run -q -p eatme-cli -- assets validate --json
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
mkdocs build --strict
TMPDIR=/tmp ./scripts/quality-gates.sh
```

Record readiness only when the checked PR head, GitHub checks, merge state,
required QA, default-workflow proof boundary, and structured blocker state all
support the same conclusion.

## Conclusion

For PR #199 at `6f815a58077a622685a10f3ac68d16b36dc5d332`, GitHub reports a
clean merge state and successful completed checks, with `Deploy to GitHub Pages`
and `manual real Alice launch smoke` skipped. Those skipped checks must not be
treated as deployment evidence or original Alice action evidence. Local
repository QA passed at that head.

This evidence does not clear missing original Alice action evidence and does not
rehabilitate the invalid `default-workflow-attempt.log`. The merge-readiness
record remains scope-bound: PR #199 has clean metadata and QA for the checked
head, while missing real Alice action evidence must remain visible as
`missing_real_action_evidence` unless real evidence is produced.
