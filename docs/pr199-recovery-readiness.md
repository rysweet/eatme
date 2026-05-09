# PR #199 recovery readiness [PLANNED]

This document describes the intended PR #199 recovery contract. The recovery
module and public Rust entrypoint are implementation-pending; this page is not a
claim that a `pr199-recovery` CLI command already exists.

PR #199 recovery readiness decides whether PR #199 has real merge-readiness
evidence after a recovery refactor. It accepts factual worktree-local proof for
the recovery workflow, QA commands, and Alice evidence. PR metadata is fetched as
supporting context only. Missing original Alice action evidence remains a blocker
until real evidence exists.

The recovery path is intentionally narrow. It does not repair unrelated CI
failures, regenerate assets for convenience, infer missing Alice actions, or
promote wrapper logs into proof.

## Contents

- [Quick start](#quick-start)
- [Readiness contract](#readiness-contract)
- [Workflow proof](#workflow-proof)
- [Alice action evidence](#alice-action-evidence)
- [Required QA proof](#required-qa-proof)
- [GitHub PR metadata](#github-pr-metadata)
- [Structured blocker API](#structured-blocker-api)
- [Configuration](#configuration)
- [Examples](#examples)
- [Troubleshooting](#troubleshooting)

## Quick start

PR #199 recovery is planned as an internal recovery module with a public Rust
entrypoint, not as a standalone CLI command. Build the entrypoint so callers pass
collected evidence as typed data and receive a structured readiness report:

```rust
// [PLANNED] Implement in the PR #199 recovery module.
let report = pr199_recovery::evaluate_pr199_recovery_readiness(evidence)?;
assert_eq!(report.pr, 199);
```

The entrypoint returns a structured readiness report. A ready report has no
blockers and includes passing workflow, Alice, QA, and PR metadata sections.
This ready report is a shape example:

```json
{
  "pr": 199,
  "status": "ready",
  "workflow": {
    "proof": "RealDefaultWorkflowNoTimeout"
  },
  "alice": {
    "original_actions": [
      "save-project",
      "place-object"
    ]
  },
  "qa": {
    "required_commands_passed": true
  },
  "pr_metadata": {
    "number": 199,
    "supporting_context_only": true
  },
  "blockers": []
}
```

A not-ready report keeps every missing or invalid fact visible. This blocked
report is an excerpt:

```json
{
  "pr": 199,
  "status": "not_ready",
  "workflow_proof": null,
  "blockers": [
    {
      "code": "invalid_workflow_proof",
      "field": "workflow_proof",
      "message": "Only RealDefaultWorkflowNoTimeout is accepted for PR #199 recovery readiness."
    },
    {
      "code": "missing_real_action_evidence",
      "field": "alice.original.actions.save-project",
      "action": "save-project",
      "message": "Original Alice save-project action evidence is missing and must remain blocked until real evidence exists."
    }
  ]
}
```

Do not rewrite, summarize, or remove blockers before posting recovery evidence.
They are part of the readiness result.

## Readiness contract

PR #199 is recovery-ready only when every required gate is satisfied by factual
worktree-local evidence.

| Gate | Accepted evidence | Rejected evidence |
| --- | --- | --- |
| Workflow path | `RealDefaultWorkflowNoTimeout` from the real `default-workflow` execution path | Timeout shortcuts, manual fallback attempts, fallback logs, substitute proof, or `default-workflow-attempt.log` |
| Alice originals | Real original Alice action evidence for each required action | Inferred, reconstructed, synthetic, copied, renamed, or placeholder Alice action evidence |
| Missing Alice evidence | Structured `missing_real_action_evidence` blockers | Silent omission, success-shaped defaults, or generic missing-evidence text |
| QA commands | The exact five scoped commands listed in [Required QA proof](#required-qa-proof), rerun in the current worktree | Stale logs, renamed commands, partial command sets, external-worktree logs, or substitute checks |
| PR metadata | Fixed `gh pr view 199` metadata fetched through `CommandRunner` as supporting context | Shell-interpolated PR commands, other PR numbers, or metadata used as a replacement for local proof |
| Scope | PR #199 recovery evidence handling and recovery code simplification only | Unrelated docs, assets, workflows, CI cleanup, or behavior changes |

PR metadata is supporting context. It can confirm that the GitHub PR state is
compatible with local evidence, but it cannot make missing local proof ready.

## Workflow proof

The workflow gate accepts exactly one workflow proof value:

```text
RealDefaultWorkflowNoTimeout
```

That value means the real `default-workflow` path ran without a timeout shortcut
and without a manual fallback substitute. It is the only workflow proof that can
advance readiness.

The planned recovery system converts all other workflow inputs into blockers:

| Input | Readiness result |
| --- | --- |
| Timeout shortcut | `invalid_workflow_proof` blocker |
| Manual fallback attempt | `invalid_workflow_proof` blocker |
| `default-workflow-attempt.log` | `invalid_workflow_proof` blocker |
| Substitute proof text | `invalid_workflow_proof` blocker |
| Missing workflow proof | `missing_workflow_proof` blocker |

Blocked workflow report excerpt:

```json
{
  "workflow_proof": "default-workflow-attempt.log",
  "blockers": [
    {
      "code": "invalid_workflow_proof",
      "field": "workflow_proof",
      "message": "default-workflow-attempt.log is not proof of the real default-workflow path."
    }
  ]
}
```

## Alice action evidence

Recovery evidence preserves the difference between real Alice action evidence and
known missing evidence. Required original Alice actions must be backed by real
action artifacts before they can be marked present.

When an original Alice action is missing, the planned recovery report emits a
structured blocker using the exact marker:

```text
missing_real_action_evidence
```

Consumers must keep that blocker intact. They must not infer action evidence from
modernized RabbitHole evidence, launch evidence, screenshots, scenario text,
human notes, or a PR comment.

Missing original Alice action blocker excerpt:

```json
{
  "code": "missing_real_action_evidence",
  "field": "alice.original.actions.place-object",
  "action": "place-object",
  "target": "original",
  "message": "Original Alice place-object action evidence is missing."
}
```

This blocker means the claim is not ready. It does not mean the recovery code
failed, and it must remain visible until real original Alice action evidence is
collected.

## Required QA proof

PR #199 recovery readiness requires this exact five-command scoped QA set to
have been rerun in the current worktree:

```bash
cargo test --workspace --all-features
cargo run -q -p eatme-cli -- assets validate --json
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
mkdocs build --strict
TMPDIR=/tmp ./scripts/quality-gates.sh
```

The command identity is exact. Argument order, package names, flags, and the
`TMPDIR=/tmp` prefix on the quality-gates command are part of the proof. The
separate `NODE_OPTIONS` export in [Configuration](#configuration) is an
environment precondition, not a sixth QA command. The quality-gates command uses
`TMPDIR=/tmp` so deep worktrees do not fail because of Unix socket path length
limits.

The QA gate rejects:

| Condition | Blocker |
| --- | --- |
| A required command is absent | `missing_qa_proof` |
| A command was run in another worktree | `stale_qa_proof` |
| A command was renamed or approximated | `invalid_qa_command` |
| Only a subset of commands was run | `incomplete_qa_proof` |
| A command failed | `failed_qa_command` |

QA proof does not replace workflow proof or Alice action evidence. A green QA
set can coexist with `missing_real_action_evidence` blockers.

## GitHub PR metadata

The planned recovery entrypoint fetches PR #199 metadata with fixed arguments
through `CommandRunner`:

```text
gh pr view 199 --json headRefOid,mergeStateStatus,mergeable,statusCheckRollup
```

The PR number is fixed to `199`. Callers do not pass a shell command, PR number,
or field list. The metadata supports the recovery report by showing the PR head,
merge state, mergeability, and check rollup for PR #199.

Accepted metadata does not override local blockers. For example, a clean merge
state cannot clear `missing_real_action_evidence`, and green GitHub checks cannot
turn `default-workflow-attempt.log` into workflow proof.

## Structured blocker API

Readiness state is authoritative through blockers. A report is ready only when
`blockers` is empty.

```json
{
  "pr": 199,
  "status": "not_ready",
  "blockers": [
    {
      "code": "missing_real_action_evidence",
      "field": "alice.original.actions.save-project",
      "action": "save-project",
      "target": "original",
      "message": "Original Alice save-project action evidence is missing."
    }
  ],
  "qa": {
    "required_commands": [
      "cargo test --workspace --all-features",
      "cargo run -q -p eatme-cli -- assets validate --json",
      "cargo run -q -p eatme-cli -- assets generate-gadugi --check --json",
      "mkdocs build --strict",
      "TMPDIR=/tmp ./scripts/quality-gates.sh"
    ]
  },
  "pr_metadata": {
    "number": 199,
    "supporting_context_only": true
  }
}
```

Use `code` for automation and `message` for reviewer-facing output. Consumers
should not parse human text to infer readiness.

Common blocker codes:

| Code | Meaning |
| --- | --- |
| `missing_workflow_proof` | No accepted real default-workflow proof was found. |
| `invalid_workflow_proof` | Workflow evidence was present but was a timeout, manual fallback, fallback log, or substitute. |
| `missing_real_action_evidence` | Required original Alice action evidence is absent and must remain blocked. |
| `invalid_alice_action_evidence` | Alice evidence was present but unsafe, malformed, synthetic, or scoped to the wrong target. |
| `missing_qa_proof` | A required scoped QA command has no worktree-local proof. |
| `invalid_qa_command` | A QA proof entry does not match the required command exactly. |
| `failed_qa_command` | A required scoped QA command ran and failed. |
| `wrong_pr_scope` | Evidence or metadata is not scoped to PR #199. |

## Configuration

Run recovery-related commands from the repository root.

Set the repository Node heap preference before workflow-related commands. This
is an environment precondition for reliable local execution, not an additional
QA proof command:

```bash
export NODE_OPTIONS=--max-old-space-size=32768
```

The preference is stored outside the repository in the Amplihack user config:

```text
$HOME/.amplihack/config
```

Do not put secrets, tokens, environment dumps, raw authenticated `gh` output, or
local credential paths into recovery evidence. The recovery report records only
the factual fields needed for PR #199 readiness.

## Examples

### Accept real workflow proof excerpt

```json
{
  "workflow_proof": "RealDefaultWorkflowNoTimeout",
  "blockers": []
}
```

The workflow gate passes. Other gates still decide the final readiness result.

### Reject a timeout shortcut excerpt

```json
{
  "workflow_proof": "TimeoutShortcut",
  "blockers": [
    {
      "code": "invalid_workflow_proof",
      "field": "workflow_proof",
      "message": "TimeoutShortcut is not accepted for PR #199 recovery readiness."
    }
  ]
}
```

The workflow gate remains blocked even if the timeout shortcut produced useful
notes.

### Preserve missing Alice originals excerpt

```json
{
  "alice": {
    "original": {
      "actions": {
        "save-project": null
      }
    }
  },
  "blockers": [
    {
      "code": "missing_real_action_evidence",
      "field": "alice.original.actions.save-project",
      "action": "save-project",
      "target": "original",
      "message": "Original Alice save-project action evidence is missing."
    }
  ]
}
```

The missing original action remains explicit. A consumer may display this blocker
or fail a readiness gate, but it must not replace it with synthetic evidence.

### Reject substitute QA proof excerpt

```json
{
  "qa": {
    "observed_commands": [
      "cargo test -p eatme-core",
      "mkdocs build"
    ]
  },
  "blockers": [
    {
      "code": "incomplete_qa_proof",
      "field": "qa.observed_commands",
      "message": "PR #199 recovery readiness requires the exact five scoped QA commands."
    }
  ]
}
```

Partial checks are useful debugging information, but they are not merge-readiness
proof for PR #199 recovery.

## Troubleshooting

| Symptom | Cause | Fix |
| --- | --- | --- |
| `default-workflow-attempt.log` appears in the report | A fallback log was supplied as proof | Rerun the real `default-workflow` path and record `RealDefaultWorkflowNoTimeout`. |
| `missing_real_action_evidence` blocks readiness | Required original Alice action evidence is absent | Collect real original Alice action evidence or keep the blocker visible. |
| QA proof is rejected as stale | The command proof came from another worktree or older head | Rerun the exact required command in the current worktree. |
| PR metadata is clean but status is `not_ready` | Local workflow, QA, or Alice evidence still has blockers | Fix the local blocker. PR metadata cannot override it. |
| A readiness comment would need caveats | The recovery report still has blockers | Do not publish merge-readiness. Publish or preserve the blockers instead. |
