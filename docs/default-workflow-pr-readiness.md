# Default workflow PR readiness

Default workflow PR readiness is the recovery contract for a branch that must be
brought current, validated, and handed off through a pull request without
overstating Alice evidence. It is a documentation and PR-readiness workflow: it
ties validation evidence to the final Git commit and keeps launch-smoke readiness
bounded to the evidence the repository actually inspects.

Use this workflow for any recovery branch or readiness PR that updates Alice
readiness docs, scenario assets, generated adapters, readiness tests, or a
worktree-root no-op guard.

## Contents

- [What this workflow proves](#what-this-workflow-proves)
- [Worktree-root guard contract](#worktree-root-guard-contract)
- [Recovery usage](#recovery-usage)
- [Exact-head evidence](#exact-head-evidence)
- [PR update contract](#pr-update-contract)
- [API and output boundaries](#api-and-output-boundaries)

## What this workflow proves

The workflow proves only that the final branch `HEAD` has:

- been updated from current `master` through a clean merge or rebase;
- resolved only task-scoped conflicts in readiness docs, scenario assets,
  generated Gadugi adapters, readiness tests, or the no-op guard;
- run the documented validation commands from the actual Git worktree root; and
- preserved bounded real Alice launch-smoke readiness wording.

It keeps the canonical non-claims visible:

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

## Worktree-root guard contract

Every no-op or TDD guard in this workflow resolves the active repository root at
runtime:

```bash
repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"
```

If `git rev-parse --show-toplevel` fails, the guard fails clearly and exits
non-zero. It must not treat a non-Git directory as a clean no-op, and it must not
use a hard-coded linked-worktree path from an earlier recovery run.

All Git checks that decide whether a change exists run from that resolved root:

```bash
git diff --quiet
git diff --cached --quiet
test -z "$(git status --porcelain)"
```

The guard may report "no task-scoped changes" only after those commands have run
inside the active Git worktree and `git status --porcelain` is empty. Untracked
files are still changes. A stale path, missing `.git` link, detached session
directory, or non-empty porcelain status is a guard failure, not success.

## Recovery usage

Start from the actual repository root:

```bash
repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"
export NODE_OPTIONS=--max-old-space-size=32768
```

Use the existing recovery branch:

```bash
git switch <recovery-branch>
git fetch origin master
git merge origin/master
```

Prefer merge when it preserves the branch's recovery history and avoids risky
conflict churn. Use rebase only when it is clearly the lower-risk clean path.

Resolve conflicts only in task-scoped files:

- readiness documentation;
- canonical scenario YAML under `assets/scenarios/eatme/`;
- generated Gadugi adapters under `assets/scenarios/gadugi/`;
- readiness/reporting Rust tests or code directly touched by the merge; and
- the no-op/TDD guard that resolves the Git worktree root.

Do not refactor unrelated code, rewrite unrelated history, or broaden the
readiness claim while resolving conflicts.

## Exact-head evidence

Capture the final SHA only after the merge, conflict resolution, generated
outputs, documentation updates, and any recovery commits are complete and the
worktree is clean:

```bash
test -z "$(git status --porcelain)"
final_head="$(git rev-parse HEAD)"
printf 'Final HEAD: %s\n' "$final_head"
```

Run validation on that exact `HEAD`. If a command is rerun after another commit,
replace the old SHA and old command results with the new exact-head evidence.

Required validation evidence:

```bash
mkdocs build --strict
cargo run -q -p eatme-cli -- assets validate --json
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
cargo test -p eatme-alice --test first_lesson_readiness_sequence
cargo test -p eatme-cli --test alice_first_lesson_readiness
cargo test -p eatme-cli --test alice_first_lesson_readiness_reporting
```

When targeted validation is clean and practical, run the full quality gate from
the same Git root:

```bash
TMPDIR=/tmp ./scripts/quality-gates.sh
```

Document unrelated pre-existing blockers plainly. Do not turn a blocked or
failed command into readiness evidence.

## PR update contract

Update the existing PR for the recovery branch when the working tree is clean.
Create a new PR only when no suitable PR exists for that branch.

The PR body includes:

- final `HEAD` SHA;
- validation commands that were actually run on that SHA;
- pass/fail/blocker result for each command;
- bounded real Alice launch-smoke/readiness claim; and
- explicit non-claims.

Safe bounded claim:

```text
This PR preserves bounded real Alice launch-smoke/readiness evidence for the
final HEAD only. It reports repository/docs/assets/generated-adapter/readiness
validation and launch-smoke readiness wording only.
```

Required non-claims:

```text
Canonical non-claims:
- First-lesson completion is not proven.
- Full world execution is not proven.
- Grading is not proven.
- Creative assessment is not proven.
- Full Alice UI automation is not proven.
- Visible rendering correctness is not proven.
- Save completion is not proven.
- Deployed sharing/platform success is not proven.
```

Unsafe PR wording:

```text
Alice first lesson is complete.
The UI automation passes end to end.
The world runs correctly.
The project renders correctly.
The work is graded or creatively assessed.
Save is complete.
Sharing or platform deployment succeeded.
```

## API and output boundaries

No new public API is introduced by this workflow. It uses the existing readiness
schema:

```text
eatme.alice-lesson-session-readiness/v1
```

PR automation and reviewers consume the same fields documented in
[Real Alice Launch-Smoke Readiness](real-alice-launch-smoke-readiness.md),
[First-Lesson Evidence Readiness](first-lesson-evidence-readiness.md), and
[Lesson Session Readiness](lesson-session-readiness.md). New or updated output
must keep `unproven_claims` visible and must not reinterpret launch-smoke
evidence as first-lesson completion, assessment, rendering correctness, Save
completion, full world execution, Full Alice UI automation, or deployed
sharing/platform success.
