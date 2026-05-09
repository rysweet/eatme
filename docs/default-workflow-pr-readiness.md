# PR 174 persona/scenario gap-fill readiness

This page defines the readiness contract for recovered PR 174. The PR is
review-ready for the persona/scenario gap-fill scope only when final handoff
evidence is collected at the exact PR head and the worktree is clean.

The feature scope is editable persona assets, canonical EatMe scenario assets,
generated Gadugi adapter freshness, repository-local asset validation, and
exact-head GitHub metadata.

This is not Alice classroom evidence. It does not prove Alice UI automation,
grading, creative assessment, save/reopen/export completion, deployed sharing,
or lesson completion.

## Exact-head evidence model

Do not treat a checked-in commit SHA as current readiness evidence. Any
documentation commit changes `HEAD`, so exact-head evidence belongs in the
final PR handoff note or CI logs after the last commit has been pushed.

Collect exact-head evidence only after syncing to PR #174's actual head and
confirming a clean worktree:

```bash
git fetch origin pull/174/head:pr-174-persona-gap-fill
git switch pr-174-persona-gap-fill
git status --short
git rev-parse HEAD
gh pr view 174 --json headRefOid,mergeStateStatus,mergeable,statusCheckRollup
```

Record the starting HEAD with `git rev-parse HEAD` before making recovery
changes. The final handoff must record these fields for the same commit:

| Field | Required value |
| --- | --- |
| PR | `174` |
| Branch | `wave6-persona-gap-fill-1778302300` |
| Local HEAD | The final commit SHA from `git rev-parse HEAD`. |
| PR `headRefOid` | The same SHA as Local HEAD. |
| GitHub merge state | `CLEAN`. |
| GitHub mergeability | `MERGEABLE`. |
| GitHub check rollup | No failed or pending checks for the same head. Skipped checks are acceptable only when they are outside the persona/scenario asset scope and do not expand the readiness claim. |

Repository-local asset checks for the same head must pass:

| Check | Evidence |
| --- | --- |
| `cargo run -q -p eatme-cli -- assets validate --json` | Passes with no errors or warnings. Expected PR 174 asset counts are `instructor_count: 11`, `student_count: 13`, and `scenario_asset_count: 95`. |
| `cargo run -q -p eatme-cli -- assets generate-gadugi --check --json` | Passes with no changed adapters and no errors. Expected PR 174 adapter counts are `generated_count: 47` and `checked_count: 47`. |

If the branch moves, refresh the handoff evidence after syncing to the new PR
head and rerunning the repository-local checks.

## Review evidence

Review evidence must be collected for the same exact head as readiness evidence:

```bash
gh pr view 174 --json headRefOid,reviewDecision,reviews,comments
```

The review gate passes only when:

| Field | Required value |
| --- | --- |
| `headRefOid` | `headRefOid` must match `git rev-parse HEAD`. |
| `reviewDecision` | Record the current `reviewDecision` for the same exact head. |
| `reviews` | Review entries must apply to the same exact head or be treated as historical context only. |
| `comments` | Review comments must be bounded to editable persona/scenario assets, generated adapter freshness, or repository-local asset checks. |

Do not use stale review comments, skipped checks, or local-only validation as
review evidence for a moved branch.

## Source-of-truth assets

Editable assets are the source of truth:

```text
assets/personas/alice-user-crew.yaml
assets/scenarios/eatme/*.yaml
```

Generated Gadugi adapters are review artifacts, not authoritative content:

```text
assets/scenarios/gadugi/*.yaml
```

Refresh generated adapters only with the existing generator when canonical
EatMe scenario assets change:

```bash
cargo run -q -p eatme-cli -- assets generate-gadugi --json
```

Then verify adapter freshness:

```bash
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
```

## Persona/scenario coverage contract

PR 174 is ready when these asset-level statements are true:

| Area | Required state |
| --- | --- |
| Persona crew | `assets/personas/alice-user-crew.yaml` defines instructor and student personas that cover setup, teaching, debugging, assessment-boundary, classroom logistics, creative, accessibility, VR/player, sharing, and reflection needs. |
| Constituencies | `constituency_coverage` links non-coder-editable constituency records to persona IDs and scenario IDs. |
| Scenario references | Canonical EatMe scenarios reference persona IDs that resolve through the persona crew asset. |
| Scenario wording | Scenario text stays at the editable asset level and does not claim full Alice UI automation, grading, creative assessment validation, save/reopen/export completion, or lesson completion. |
| Gadugi adapters | Generated adapters are reproducible from canonical EatMe scenario assets and pass check mode with no changes. |

## Refreshing readiness evidence

Run these commands from the repository root:

```bash
git rev-parse --show-toplevel
```

Sync to the current PR head before collecting evidence:

```bash
git fetch origin pull/174/head:pr-174-persona-gap-fill
git switch pr-174-persona-gap-fill
```

Record the local head and compare it to GitHub:

```bash
HEAD="$(git rev-parse HEAD)"
gh pr view 174 --json headRefOid,mergeStateStatus,mergeable,statusCheckRollup
```

The GitHub gate passes only when:

| Field | Required value |
| --- | --- |
| `headRefOid` | Same value as `git rev-parse HEAD`. |
| `mergeStateStatus` | `CLEAN`. |
| `mergeable` | `MERGEABLE`. |
| `statusCheckRollup` | No failed or pending checks for the same head. Skipped checks are acceptable only when they are out of scope for persona/scenario asset readiness. A skipped manual Alice smoke check, for example, must not be cited as Alice UI evidence. |

Run the asset-level validation commands:

```bash
cargo run -q -p eatme-cli -- assets validate --json
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
```

Use the repository quality gate only as an additional repository health check,
not as Alice UI, grading, creative assessment, or lesson-completion evidence:

```bash
TMPDIR=/tmp NODE_OPTIONS=--max-old-space-size=32768 ./scripts/quality-gates.sh
```

Block readiness if `git status --short` reports local changes, or if GitHub
reports a different `headRefOid`, a non-clean merge state, failed checks,
pending checks, or checks for another commit.

## Supported and unsupported claims

Supported review claims:

- PR 174 fills persona/scenario gaps in editable assets.
- Persona IDs used by canonical EatMe scenarios resolve through the persona crew
  asset.
- Generated Gadugi adapters are fresh relative to canonical EatMe scenarios.
- Repository-local asset validation passes for the exact PR head.
- GitHub metadata names the same exact head and reports the PR mergeable.

Unsupported review claims:

- Alice UI automation completed a full user journey.
- A learner completed a lesson.
- A student world was graded or creatively assessed automatically.
- Save/reopen/export was completed in a live Alice session.
- Visual rendering, deployed sharing, or classroom success was verified.

## Handoff note

Use this template after exact-head evidence is refreshed at the final PR head:

```text
PR 174 persona/scenario gap-fill readiness

Branch: wave6-persona-gap-fill-1778302300
HEAD: <git rev-parse HEAD>
PR headRefOid: <gh pr view 174 --json headRefOid>
Merge state: CLEAN
Mergeable: MERGEABLE
Worktree: clean
Check rollup: no failed or pending checks for the same head; skipped checks are
out of scope and are not used as Alice UI, grading, creative assessment, or
lesson-completion evidence.

Editable source assets:
- assets/personas/alice-user-crew.yaml
- assets/scenarios/eatme/*.yaml

Generated review artifacts:
- assets/scenarios/gadugi/*.yaml

Repository-local checks:
- cargo run -q -p eatme-cli -- assets validate --json
- cargo run -q -p eatme-cli -- assets generate-gadugi --check --json

Boundary: this evidence supports editable persona/scenario asset readiness and
generated adapter freshness for the exact head. It does not claim Alice UI
automation, grading, creative assessment, save/reopen/export completion,
deployed sharing, visual correctness, classroom success, or lesson completion.
```

## Publishing bounded PR evidence

Publish only asset-scoped evidence to PR 174 after the final commit is pushed
and the exact-head checks above have passed:

```bash
gh pr comment 174 --body-file /path/to/pr-174-evidence.md
```

The body file must use this bounded template:

```markdown
Persona/scenario gap-fill readiness refreshed for HEAD `<commit-sha>`.

Asset-scoped evidence:
- `cargo run -q -p eatme-cli -- assets validate --json` succeeded.
- `cargo run -q -p eatme-cli -- assets generate-gadugi --check --json` succeeded.
- Committed changes are limited to canonical persona assets, canonical EatMe scenario assets, and generator-produced Gadugi adapters under `assets/scenarios/gadugi/*.yaml`.

Scope note: this evidence covers persona/scenario asset completeness, validation success, and generated-adapter freshness only. It does not claim Alice UI automation, grading correctness, creative assessment quality, completed lessons, or full lesson-flow coverage.
```

If GitHub publishing fails because of authentication, API errors, or rate
limiting, preserve the intended PR text unchanged outside the repository and
record the exact `gh` failure beside it. Do not commit fallback logs or local
publish-attempt files to the PR branch.

## Related documentation

- [Persona Assets](persona-assets.md)
- [Scenario Authoring](scenario-authoring.md)
- [Gadugi Adapters](gadugi-adapters.md)
- [Generated Asset Consistency](generated-asset-consistency.md)
- [Validation and Quality Gates](validation-quality-gates.md)
