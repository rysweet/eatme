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
- [Authoritative quality gate](#authoritative-quality-gate)
- [Documentation strict build](#documentation-strict-build)
- [Exact-head evidence](#exact-head-evidence)
- [PR update contract](#pr-update-contract)
- [API and output boundaries](#api-and-output-boundaries)

## What this workflow proves

The workflow proves only that the final branch `HEAD` has:

- been updated from current `master` through a clean merge or rebase;
- resolved only task-scoped conflicts in readiness docs, scenario assets,
  generated Gadugi adapters, readiness tests, or the no-op guard;
- passed the authoritative repository quality gate from the actual Git worktree
  root; and
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

Record the branch, current `HEAD`, and dirty status before changing anything:

```bash
git branch --show-current
git rev-parse HEAD
git status --short
```

Preserve unrelated dirty files. They are not part of the readiness claim unless
they directly conflict with task-scoped recovery files.

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

## Authoritative quality gate

`scripts/quality-gates.sh` is the authoritative local validation entrypoint for
default-workflow recovery. It owns the combined fmt, clippy, test, module-size,
and coverage/quality expectations:

```bash
export NODE_OPTIONS=--max-old-space-size=32768
TMPDIR=/tmp ./scripts/quality-gates.sh
```

Use `TMPDIR=/tmp` in deep linked worktrees so test sockets and temporary paths do
not fail because of checkout path length. Do not weaken, bypass, or replace this
script when producing readiness evidence.

The gate covers:

| Check | Evidence boundary |
| --- | --- |
| `cargo fmt --check` | Rust formatting is current. |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | Workspace clippy warnings are repaired instead of suppressed through broad allowances. |
| `cargo test --workspace --all-features` | Existing Rust tests pass without requiring real Alice UI automation. |
| Rust module-size check | Source modules under `crates/` stay within the repository line-count contract. |
| Coverage/quality check | The repository coverage/quality gate passes at the same commit. |

Targeted commands are useful for diagnosis, but they are not a substitute for the
authoritative gate. Run targeted fmt, clippy, tests, asset validation, or Gadugi
freshness checks only to locate and repair failures before rerunning the full
gate.

## Documentation strict build

`mkdocs build --strict` is the authoritative documentation check. It is a
separate docs-site validation command, not part of `scripts/quality-gates.sh`.
Run it when readiness documentation changes:

```bash
mkdocs build --strict
```

Do not describe a passing `scripts/quality-gates.sh` run as docs-site evidence.
When both Rust readiness and documentation changed, record both commands against
the same final commit SHA.

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
export NODE_OPTIONS=--max-old-space-size=32768
TMPDIR=/tmp ./scripts/quality-gates.sh
```

If readiness documentation changed, also run the docs strict build from the same
final `HEAD`:

```bash
mkdocs build --strict
```

If canonical scenario assets changed, also verify the source assets and generated
adapters from the same final `HEAD`:

```bash
cargo run -q -p eatme-cli -- assets validate --json
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
```

Do not hand-edit generated Gadugi assets. Regenerate them through the existing
generator when canonical scenario assets change, then rerun the authoritative
quality gate.

Document unrelated pre-existing blockers plainly. Do not turn a blocked, skipped,
or failed command into readiness evidence.

## PR update contract

Update the existing PR for the recovery branch when the working tree is clean.
Create a new PR only when no suitable PR exists for that branch.

The PR body includes:

- final `HEAD` SHA;
- the authoritative quality-gate command that passed on that SHA;
- docs strict-build result when readiness documentation changed;
- asset-validation and generated-adapter freshness results when canonical
  scenario assets changed;
- bounded real Alice launch-smoke/readiness claim; and
- explicit non-claims.

Safe bounded claim:

```text
This PR preserves bounded real Alice launch-smoke/readiness evidence for the
final HEAD only. It reports repository quality-gate, docs strict-build,
asset/readiness, and launch-smoke wording evidence only.
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

Unsafe PR wording is any statement that treats the bounded readiness evidence as
proof of a completed lesson, full UI automation, full world execution, visible
rendering correctness, grading, creative assessment, Save completion, or deployed
sharing/platform success. Do not include those statements in docs, generated
adapters, validation summaries, or PR text.

PR #188 uses the same bounded evidence shape. Its final evidence belongs in the
PR body or review handoff after the final passing gate exists, not as a stale
point-in-time result in this reference document. The recovery record names the
exact final commit SHA, the passing `NODE_OPTIONS=--max-old-space-size=32768
TMPDIR=/tmp ./scripts/quality-gates.sh` result for that commit, and the
`mkdocs build --strict` result when docs changed. If another commit is added
after the PR body or review comment is updated, replace the evidence with the new
commit SHA and rerun results before requesting review.

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

## Scope

This page also preserves the PR #175 publication-head evidence contract that was
already on `master` when this recovery branch was brought current.

| Field | Observed value |
| --- | --- |
| Validated evidence head | `a951f34a0a187adfa24cfe0555ca00da6a04197d` |
| Artifact publication head | not embedded in this committed artifact; committing a documentation refinement changes the PR head. |

The validated evidence head is the PR head whose metadata and check rollup were
captured for PR #175. The Artifact publication head is intentionally not
embedded in this committed artifact, because committing a documentation
refinement changes the PR head. This page therefore does not claim that its own
eventual publication commit has checked itself.

## Readiness evidence

### Local Git observations

The local Git evidence for PR #175 was captured before this refinement changed
the artifact/test files. It is a pre-refinement observation for the validated
evidence head and not a claim about the post-edit worktree or the eventual
publication head.

This refinement intentionally changes only the readiness artifact and the
contract tests that guard it:

```text
docs/default-workflow-pr-readiness.md
crates/eatme-assets/src/default_workflow_pr_readiness_contract_tests.rs
```

### GitHub PR #175 observations

PR #175 metadata was recorded as GitHub metadata for the validated evidence
head. Those observations are evidence for that head only and do not approve,
merge, or broaden this recovery branch.

### Validated evidence-head executable evidence

Validated evidence-head commands record `NODE_OPTIONS=--max-old-space-size=32768`
and use no timeout wrapper.

```bash
cargo run -q -p eatme-cli -- assets validate --json
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
mkdocs build --strict
TMPDIR=/tmp ./scripts/quality-gates.sh
```

These commands are executable evidence for the recorded PR #175 validated
evidence head only. They do not prove manual real Alice desktop launch, full UI
automation, visible rendering correctness, grading, creative assessment, Save
completion, or lesson completion.

### Historical same-head outside-in testing evidence

Historical `uvx` checks used a branch ref as resolved at execution time, not an
immutable SHA-pinned install reference. Any same-head claim depends on the
recorded execution context, not on the mutable branch name alone.

## Review evidence

Review evidence keeps PR metadata, local Git observations, and executable
validation results separate. Skipped, not-measured, no-execute, and historical
states remain nonclaims instead of success evidence.

## Finalization evidence

PR #175 remains unmerged. No manual merge was performed. The PR #175 record is
workflow readiness/review/finalization evidence, not proof of approval or broad
product readiness.

Finalization status: `merge-ready-after-publication-head-checks` for the PR #175
evidence-contract recovery. The post-push publication head/check rollup recorded
outside this file is required before using it as final merge evidence.

### External publication-head evidence record

The exact publication-head evidence must be recorded outside this committed
artifact after push. A final no-op or merge-ready statement must be a literal
no-op justification tied to the publication head, check rollup, and focused
artifact-contract scope. Owner-free finalization does not require owner
intervention.

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
- No manual merge is claimed.
- No blanket CI success is claimed beyond the listed evidence-head or
  publication-head check rollup.
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
