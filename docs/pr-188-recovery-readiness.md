# PR #188 recovery readiness

PR #188 recovery readiness is a PR-specific specialization of
[Default Workflow PR Readiness](default-workflow-pr-readiness.md) for the
`wave6-real-alice-smoke-report-1778302300` branch. It documents the intended
review-handoff behavior for recovery work: evidence must be produced from the
current branch `HEAD`, the pull request remains unmerged until normal review
accepts it, and claims stay within the silver-thread/e2e launch-smoke boundary.

Here, silver-thread/e2e launch-smoke means the narrow end-to-end path that proves
Alice can be packaged, launched, observed, and reported through deterministic
launch-smoke artifacts. It does not mean complete UI-driven lesson execution.

Use this guide when PR #188 needs recovery after an interrupted owner session,
rate limit, or no-op readiness handoff.

## Contents

- [Scope](#scope)
- [Configuration](#configuration)
- [Usage](#usage)
- [No-op acceptance](#no-op-acceptance)
- [Review and finalization evidence](#review-and-finalization-evidence)
- [Output boundary](#output-boundary)
- [Recovery command sequence](#recovery-command-sequence)
- [Canonical non-claims](#canonical-non-claims)

## Scope

The recovery scope is deliberately narrow:

| In scope | Out of scope |
| --- | --- |
| Confirm the active branch is `wave6-real-alice-smoke-report-1778302300`. | Manually merge PR #188. |
| Run executable checks against the current branch `HEAD`. | Use stale CI output, older local results, or results from another branch. |
| Repair only directly failing readiness docs, scenario assets, generated Gadugi adapters, tests, or bounded output wording. | Refactor unrelated code or broaden Alice evidence claims. |
| Preserve silver-thread/e2e launch-smoke readiness wording. | Claim full UI automation, visible rendering correctness, grading, Save completion, lesson completion, or complete end-to-end lesson execution. |
| Accept a no-op only when branch/root status is clean and readiness checks pass. | Treat a dirty worktree, skipped command, or blocked command as readiness evidence. |

The recovery path describes readiness for review. It does not merge the pull
request and does not replace the repository's normal PR review controls.

## Configuration

Set the saved local Node heap preference before running workflow commands:

```bash
export NODE_OPTIONS=--max-old-space-size=32768
```

Use `/tmp` for the quality gate temporary directory in deep worktrees and keep
the saved Node heap preference in the same executable evidence command:

```bash
NODE_OPTIONS=--max-old-space-size=32768 TMPDIR=/tmp ./scripts/quality-gates.sh
```

Do not wrap commands with external command-duration tools such as `timeout` or
`gtimeout`. Let the repository commands run normally and use their own exit
statuses as evidence.

## Usage

Start from the repository root:

```bash
repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"
```

Confirm the recovery branch and current commit:

```bash
git branch --show-current
git rev-parse HEAD
git status --short
```

The branch must be:

```text
wave6-real-alice-smoke-report-1778302300
```

Run the readiness checks from the same `HEAD`:

```bash
NODE_OPTIONS=--max-old-space-size=32768 TMPDIR=/tmp ./scripts/quality-gates.sh
cargo run -q -p eatme-cli -- assets validate --json
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
```

When documentation changes are part of the recovery, also build the docs site:

```bash
mkdocs build --strict
```

If any command fails, repair only the directly failing PR #188 readiness surface,
then rerun the failing command and the authoritative quality gate against the new
`HEAD`.

## No-op acceptance

A no-op recovery is accepted only when no repository change is required. The
accepted no-op state has all of these properties:

| Requirement | Command evidence |
| --- | --- |
| Active branch is PR #188 recovery branch | `git branch --show-current` prints `wave6-real-alice-smoke-report-1778302300`. |
| Current commit is known | `git rev-parse HEAD` prints the commit used for all checks. |
| Root worktree is clean | `git status --short` prints no entries. |
| Repository quality gate passes | `NODE_OPTIONS=--max-old-space-size=32768 TMPDIR=/tmp ./scripts/quality-gates.sh` exits `0`. |
| Canonical assets validate | `cargo run -q -p eatme-cli -- assets validate --json` exits `0` with `passed: true`. |
| Generated Gadugi adapters are fresh | `cargo run -q -p eatme-cli -- assets generate-gadugi --check --json` exits `0` with `passed: true`. |

Do not accept a no-op when untracked files, staged files, unstaged files, failing
checks, skipped checks, or results from a different commit are present.

## Review and finalization evidence

The PR #188 review handoff names the exact branch and final commit, then lists
only executable current-`HEAD` checks. A safe handoff uses this shape:

```text
Branch: wave6-real-alice-smoke-report-1778302300
HEAD: <git rev-parse HEAD>
Worktree: clean at repository root

Evidence:
- NODE_OPTIONS=--max-old-space-size=32768 TMPDIR=/tmp ./scripts/quality-gates.sh
- cargo run -q -p eatme-cli -- assets validate --json
- cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
- mkdocs build --strict (include only when documentation changed)

Scope:
- Silver-thread/e2e launch-smoke readiness only.
- No manual PR merge was performed.
- Full UI automation, visible rendering correctness, grading, Save completion,
  lesson completion, and complete end-to-end lesson execution are not proven.
```

If the recovery is a no-op, add the no-op justification:

```text
No repository changes were required because the PR #188 recovery branch was
already clean at the repository root and all current-HEAD readiness checks
passed.
```

If a repair commit is required, replace the no-op sentence with a bounded change summary
and rerun the checks against the repair commit.

## Output boundary

PR #188 recovery uses the existing readiness and launch-smoke surfaces. It does
not introduce a new public API.

The readiness output remains bounded by the schema documented in
[Real Alice Launch-Smoke Readiness](real-alice-launch-smoke-readiness.md):

```text
eatme.alice-lesson-session-readiness/v1
```

Consumers may summarize:

- current branch and commit evidence;
- repository quality-gate evidence;
- canonical asset validation evidence;
- generated Gadugi freshness evidence;
- documentation strict-build evidence when documentation changed; and
- silver-thread/e2e launch-smoke readiness wording.

Consumers must not summarize PR #188 recovery as proof of full UI automation,
rendering correctness, grading, Save completion, lesson completion, complete
end-to-end lesson execution, learner-world correctness, or deployed
sharing/platform success.

## Recovery command sequence

Use this command sequence for a local recovery run:

```bash
repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

git switch wave6-real-alice-smoke-report-1778302300
git branch --show-current
git rev-parse HEAD
git status --short

export NODE_OPTIONS=--max-old-space-size=32768

NODE_OPTIONS=--max-old-space-size=32768 TMPDIR=/tmp ./scripts/quality-gates.sh
cargo run -q -p eatme-cli -- assets validate --json
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
mkdocs build --strict

git status --short
```

When the final status is clean and every command exits `0`, PR #188 is ready for
normal review handoff with bounded silver-thread/e2e launch-smoke evidence. Leave
merge completion to the normal pull request path.

## Canonical non-claims

PR #188 recovery preserves the default-workflow non-claim boundary:

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
