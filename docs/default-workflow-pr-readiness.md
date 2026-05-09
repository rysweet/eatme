# Default-workflow PR readiness

This page is the bounded readiness checklist for recovered PR 174. It is not
Alice classroom evidence and it is not proof that a user completed a
save/reopen/export journey.

## Exact-head inputs

| Field | Value |
| --- | --- |
| PR | PR: 174 |
| Branch | Branch: wave6-persona-gap-fill-1778302300 |
| Merge source | Merge source: origin/master |
| Exact head | Exact HEAD command: git rev-parse HEAD |
| Working tree | Working tree command: git status --short |
| External service | External service command: gh pr view 174 --json headRefOid,mergeStateStatus,mergeable,statusCheckRollup |

Run commands from the repository root reported by:

```bash
git rev-parse --show-toplevel
```

Use that repository root for linked worktree recovery checks, not the session directory.
This prevents a no-op guard from reading a non-Git path and treating missing
changes as proof that no implementation was needed.

## Scenario and adapter boundary

Canonical EatMe assets are the source of truth:

```text
assets/scenarios/eatme/starter-project-open-save-export-preflight.yaml
assets/scenarios/eatme/student-artifact-package-share-evidence.yaml
```

Generated Gadugi adapters must come from those canonical assets:

```text
assets/scenarios/gadugi/starter-project-open-save-export-preflight.yaml
assets/scenarios/gadugi/student-artifact-package-share-evidence.yaml
```

Use `student-artifact-package-share-evidence` for the student artifact sharing
packet boundary. Do not substitute the separate
`instructor-student-save-reopen-export-evidence-handoff` scenario for this PR
readiness example.

## Required default-workflow commands

Record pass or fail for the exact head under review:

```bash
cargo run -q -p eatme-cli -- assets validate --json
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
cargo test -q -p eatme-assets starter_project_preflight_boundary
cargo test -q -p eatme-assets gadugi
cargo test -q -p eatme-assets outside_in_alice_expansion_tests
gh pr view 174 --json headRefOid,mergeStateStatus,mergeable,statusCheckRollup
TMPDIR=/tmp NODE_OPTIONS=--max-old-space-size=32768 ./scripts/quality-gates.sh
```

If generated adapter check mode fails, regenerate adapters only from the
canonical EatMe scenario assets:

```bash
cargo run -q -p eatme-cli -- assets generate-gadugi --json
```

Then rerun check mode and the remaining commands.

## External service gate

Local validation is not enough to call the PR ready. Check GitHub before
publishing handoff evidence:

```bash
gh pr view 174 --json headRefOid,mergeStateStatus,mergeable,statusCheckRollup
```

The external service gate passes only when GitHub reports:

| Field | Required value |
| --- | --- |
| `headRefOid` | The same commit returned by `git rev-parse HEAD`. |
| `mergeStateStatus` | `CLEAN`. |
| `mergeable` | `MERGEABLE`. |
| `statusCheckRollup` | Required checks completed successfully for `headRefOid`. |

If GitHub reports a different `headRefOid`, `DIRTY`, `CONFLICTING`, pending
checks, failed checks, missing required checks, or checks for another commit,
block readiness even when local commands pass.

## Claim boundary

This evidence validates repository assets, generated adapters, and tests for the
exact head selected by `git rev-parse HEAD`. It does not claim full Save
completion, full UI automation, grading, creative assessment, visible rendering
correctness, deployed sharing or platform success, or first-lesson completion.

Readiness evidence is not Alice classroom evidence. It does not prove student
learning, student completion, visual correctness, successful sharing on a
platform, or a completed save/reopen/export journey.

## Manual fallback boundary

Do not use a manual fallback log as readiness evidence. A failed workflow log can
explain why recovery was needed, but it cannot replace exact-HEAD validation from
the repository root.

`default-workflow-attempt.log` is allowed to say why earlier recovery evidence is
invalid. It must not say that default-workflow evidence passed, that exact head
validation succeeded, or that the PR is ready for handoff.

## Handoff note shape

After all commands pass, GitHub reports the same head, and `git status --short`
prints no output, use this plain note shape:

```text
Default-workflow PR readiness

PR: 174
Branch: wave6-persona-gap-fill-1778302300
HEAD: <output of git rev-parse HEAD>
Merge source: origin/master

Canonical assets:
- assets/scenarios/eatme/starter-project-open-save-export-preflight.yaml
- assets/scenarios/eatme/student-artifact-package-share-evidence.yaml

Generated adapters:
- assets/scenarios/gadugi/starter-project-open-save-export-preflight.yaml
- assets/scenarios/gadugi/student-artifact-package-share-evidence.yaml

Commands:
- cargo run -q -p eatme-cli -- assets validate --json
- cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
- cargo test -q -p eatme-assets starter_project_preflight_boundary
- cargo test -q -p eatme-assets gadugi
- cargo test -q -p eatme-assets outside_in_alice_expansion_tests
- gh pr view 174 --json headRefOid,mergeStateStatus,mergeable,statusCheckRollup
- TMPDIR=/tmp NODE_OPTIONS=--max-old-space-size=32768 ./scripts/quality-gates.sh

External service: GitHub reports headRefOid equal to HEAD, mergeStateStatus=CLEAN,
mergeable=MERGEABLE, and required checks successful for that head.

Working tree: clean handoff requires no output from git status --short.

Boundary: this evidence validates repository assets, generated adapters, and
tests for the exact head. It does not claim full Save completion, full UI
automation, grading, creative assessment, visible rendering correctness,
deployed sharing or platform success, or first-lesson completion.
```

## Related documentation

- [Starter Project Preflight Evidence](starter-project-preflight-evidence.md)
- [Save, Reopen, and Export Evidence Handoff](save-reopen-export-evidence-handoff.md)
- [Generated Asset Consistency](generated-asset-consistency.md)
- [Validation and Quality Gates](validation-quality-gates.md)
