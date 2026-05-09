# PR #199 recovery workflow

PR #199 recovery restores merge-readiness confidence after a prior
manual-fallback process violation. It uses the real `default-workflow` path with
no timeout shortcut, records auditable proof of that execution, preserves missing
Alice action evidence as explicit structured blockers, reruns scoped QA, and
either updates only stale recovery evidence or records a literal no-op
justification tied to the current PR facts.

This workflow is intentionally narrow. It does not add features, refactor code,
fabricate missing evidence, infer unavailable Alice evidence, or treat skipped
checks as successful.

## Required recovery flow

Run recovery only for PR #199 merge-readiness validation:

1. Execute the real `default-workflow` path with no timeout-based fallback.
2. Record auditable proof of that execution: outcome, timestamp or run
   identifier, and a log reference sufficient for later review.
3. Collect current PR state: branch, head SHA, changed file list, mergeability,
   and GitHub check rollup for the same head.
4. Report GitHub check rollup by category: `success`, `failure`, `pending`,
   `cancelled`, and `skipped`. Skipped checks are not passed checks.
5. Verify missing Alice action evidence remains represented as structured
   blockers using `missing_real_action_evidence`.
6. Rerun applicable QA:

   ```bash
   cargo test --workspace --all-features
   cargo run -q -p eatme-cli -- assets validate --json
   cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
   mkdocs build --strict
   TMPDIR=/tmp ./scripts/quality-gates.sh
   ```

7. Compare current facts against the existing PR #199 merge-readiness evidence.
8. If evidence is stale or incomplete, update only the existing PR #199
   merge-readiness evidence file with focused recovery facts.
9. If nothing changed, produce a literal no-op justification tied to the current
   branch, file list, head SHA, checks, QA results, blockers, and recovery-only
   scope.

## Incidental configuration

The saved Node heap preference may be present in the local environment:

```bash
NODE_OPTIONS=--max-old-space-size=32768
```

It is an incidental agent/tooling preference, not a PR #199 recovery
requirement. Do not record local machine-specific configuration paths in
repository evidence.

No credentials, tokens, branch-protection overrides, or local credential paths
are introduced by this workflow.

## GitHub check rollup semantics

The recovery record must summarize checks for the current PR #199 head only.
Separate the rollup into:

| Category | Meaning |
| --- | --- |
| `success` | Check completed successfully for the current head. |
| `failure` | Check failed, errored, timed out, or otherwise completed unsuccessfully. |
| `pending` | Check is queued, requested, waiting, expected, in progress, or missing a final conclusion. |
| `cancelled` | Check was cancelled. |
| `skipped` | Check reported skipped. This is not a successful result. |

Do not collapse skipped, cancelled, pending, missing, wrong-head, or failed
checks into success. If branch protection or the recovery evidence requires a
check to run, skipped remains a blocker until the check produces an acceptable
current-head result.

## Structured blockers

Missing original Alice action evidence is never reconstructed or inferred. It
remains an explicit blocker.

Canonical blocker code:

```text
missing_real_action_evidence
```

Example blocker entry:

```json
{
  "code": "missing_real_action_evidence",
  "status": "blocked",
  "subject": "original_alice_action_evidence",
  "reason": "Original Alice action evidence is unavailable and must not be fabricated or reconstructed.",
  "resolution": "Preserve as an explicit merge-readiness blocker until real evidence is provided."
}
```

Preserve existing blocker wording unless the recorded blocker facts differ.
Avoid wording-only churn.

## Merge-ready evidence rules

The existing PR #199 merge-readiness evidence is updated only when current
recovery facts differ from the recorded state. If an update is needed, the only
repository file that may change is the existing PR #199 merge-readiness evidence
file.

Update evidence when any of these change:

- PR head SHA
- branch or changed file list
- GitHub check rollup, including separate skipped, cancelled, pending, failure,
  and success states
- QA command outcomes
- blocker status, blocker facts, or blocker wording required by changed facts
- real `default-workflow` recovery execution status, outcome, or log reference

Do not update evidence when the current branch, file list, head, checks, QA
outcomes, blocker state, and default-workflow proof already match the existing
record.

## No-op justification format

When no repository change is needed, emit this literal no-op justification:

```text
No-op: PR #199 recovery required no repository modification.

Current PR branch: <branch>
Current changed files: <file list or count with file-list reference>
Current PR head: <sha>
Current checks: success=<n/details>; failure=<n/details>; pending=<n/details>; cancelled=<n/details>; skipped=<n/details>
Default-workflow proof: <real default-workflow outcome and log/run reference>
Scoped QA rerun: <command outcomes>
Blockers preserved: missing_real_action_evidence remains explicit
Scope decision: existing PR #199 merge-ready evidence already matches current branch/files/head/checks/QA/default-workflow/blocker state
```

## Tutorial: recovering PR #199

Start from the PR branch and confirm the local checkout matches the PR head. Run
the real `default-workflow` path without any timeout fallback. Save the execution
outcome and log or run reference so the recovery proof is auditable.

Collect current PR metadata and check rollup with `git` and `gh`. Record the
branch, head SHA, changed files, mergeability, and check states for the same
head. Keep skipped checks separate from passed checks.

Use the recovery service adapter for GitHub PR state collection when code needs
this data. The adapter shells out to `gh pr view 199`, applies bounded retries
for transient CLI/API failures, treats malformed JSON or non-zero exits as
structured recovery errors, and preserves skipped, cancelled, pending, failed,
and successful checks as separate categories.

Run the scoped QA commands listed above. Treat environmental failures as
failures, not as merge-ready success. Inspect the existing PR #199 merge-ready
evidence and confirm that `missing_real_action_evidence` blockers remain present.
Only change blocker wording when the recorded blocker facts themselves changed.

If the evidence is stale, update only the existing PR #199 merge-readiness
evidence file and push that focused change. If the evidence is current, do not
modify the repository; emit the no-op justification with the current branch,
changed files, head, checks, default-workflow proof, QA results, blocker state,
and recovery-only scope.
