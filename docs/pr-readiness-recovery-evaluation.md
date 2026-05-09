# PR readiness recovery evaluation

The recovery evaluation system validates whether a pull request is merge-ready by
collecting structured evidence, running it through a deterministic blocker
evaluator, and producing a typed `MERGE_READY` or `NOT_MERGE_READY` verdict. It
enforces exact-SHA binding, input safety, required GitHub checks, quality-audit
cycles, diff scope focus, and bounded nonclaims.

This system does not validate full Alice UI automation, visible rendering
correctness, grading, creative assessment, Save completion, first-lesson
completion, or full Tweedle/player decode. Those remain explicit nonclaims.

## Contents

- [CLI commands](#cli-commands)
- [GitHub snapshot](#github-snapshot)
- [Recovery evaluation](#recovery-evaluation)
- [Input schema](#input-schema)
- [Evidence fields](#evidence-fields)
- [Quality-audit cycles](#quality-audit-cycles)
- [Diff scope and docs impact](#diff-scope-and-docs-impact)
- [Change outcome](#change-outcome)
- [Output report](#output-report)
- [Blocker categories](#blocker-categories)
- [Safety and sanitization](#safety-and-sanitization)
- [Forbidden claims](#forbidden-claims)
- [End-to-end workflow](#end-to-end-workflow)
- [Configuration reference](#configuration-reference)
- [Nonclaims](#nonclaims)

## CLI commands

The `pr-readiness` subcommand group exposes two commands:

| Command | Purpose |
| --- | --- |
| `pr-readiness github-snapshot` | Fetch current PR state, mergeability, and check rollup from GitHub |
| `pr-readiness recovery-evaluate` | Evaluate structured recovery input and produce a merge-readiness verdict |

Both commands support `--json` for machine-readable output.

## GitHub snapshot

Fetch a live snapshot of PR state, mergeability, and CI check rollup:

```bash
cargo run -q -p eatme-cli -- pr-readiness github-snapshot \
  --owner rysweet \
  --repo eatme \
  --pr-number 204 \
  --local-head-sha "$(git rev-parse HEAD)" \
  --required-check quality-gates
```

### `github-snapshot` options

| Option | Required | Description |
| --- | --- | --- |
| `--owner <owner>` | yes | GitHub repository owner |
| `--repo <repo>` | yes | GitHub repository name |
| `--pr-number <n>` | yes | Pull request number (must be > 0) |
| `--local-head-sha <sha>` | yes | Full 40-character local HEAD SHA |
| `--required-check <name>` | yes (repeatable) | Name of a required GitHub Actions check; at least one must be specified |
| `--json` | no | Emit JSON output |

The command shells out to `gh pr view` with bounded retries (3 attempts, 500 ms
delay, 20 s timeout). It parses the GitHub status check rollup and normalizes
each check into a `CheckSummary` with status, conclusion, required flag, and
head SHA.

Missing required checks are synthesized with `status=Missing` and
`conclusion=Missing` so the evaluator can surface them as blockers. Skipped
checks are reported as `conclusion=Skipped`, not success.

### GitHub snapshot output

Without `--json`, the command prints a one-line summary:

```text
PR #204 wave7-eatme-nonclaim-audit-1778303500 at a5cfaa841856d5f4e5f2e1ef55be96628cf7d4b5 with 3 checks
```

With `--json`, the command prints a `PrReadinessSnapshot` object:

```json
{
  "pr_number": 204,
  "branch": "wave7-eatme-nonclaim-audit-1778303500",
  "local_head_sha": "a5cfaa841856d5f4e5f2e1ef55be96628cf7d4b5",
  "pr_head_sha": "a5cfaa841856d5f4e5f2e1ef55be96628cf7d4b5",
  "merge_state_status": "CLEAN",
  "mergeable": "MERGEABLE",
  "checks": [
    {
      "name": "quality-gates",
      "status": "Completed",
      "conclusion": "Success",
      "required": true,
      "head_sha": "a5cfaa841856d5f4e5f2e1ef55be96628cf7d4b5"
    }
  ]
}
```

## Recovery evaluation

Evaluate a JSON input file against the recovery readiness rules:

```bash
cargo run -q -p eatme-cli -- pr-readiness recovery-evaluate \
  --input /path/to/recovery-input.json \
  --json
```

### `recovery-evaluate` options

| Option | Required | Description |
| --- | --- | --- |
| `--input <path>` | yes | Path to a JSON file conforming to `RecoveryReadinessInput` |
| `--json` | no | Emit JSON output; without this, a plain-text report is printed |

The command exits with a non-zero status when the verdict is `NOT_MERGE_READY`.

## Input schema

The input file must conform to `RecoveryReadinessInput` with schema version
`pr-readiness-recovery.v1`:

```json
{
  "schema_version": "pr-readiness-recovery.v1",
  "expected_remote_head_sha": "<full-40-char-sha>",
  "snapshot": { "...PrReadinessSnapshot..." },
  "validation_sha": "<full-40-char-sha>",
  "required_github_checks": ["quality-gates"],
  "asset_validation": { "...RecoveryValidationEvidence..." },
  "generated_gadugi_check": { "...RecoveryValidationEvidence..." },
  "quality_gate": { "...RecoveryValidationEvidence..." },
  "documentation_build": { "...RecoveryValidationEvidence..." },
  "quality_audit_cycles": [ "...QualityAuditCycle[]..." ],
  "diff_scope": { "changed_files": ["..."], "focused": true },
  "docs_impact": { "docs_changed": true, "strict_build_required": true },
  "pr_description_evidence": {
    "head_sha": "<sha>",
    "contains_readiness_evidence": true,
    "contains_bounded_nonclaims": true
  },
  "stale_evidence_handled": true,
  "wrapper_failures": [],
  "change_outcome": { "NoOp": { "justification": "..." } }
}
```

### Baseline constraints

The evaluator enforces these baseline rules for the current PR:

- `schema_version` must be exactly `pr-readiness-recovery.v1`
- `snapshot.pr_number` must be `204`
- `snapshot.branch` must match the expected PR branch
- `expected_remote_head_sha` is required and must be a valid 40-character hex SHA
- GitHub PR head, local HEAD, and validation SHA must all equal the expected
  remote head

## Evidence fields

Each of the four required evidence entries is a `RecoveryValidationEvidence`:

```json
{
  "name": "asset validation",
  "command": "cargo run -q -p eatme-cli -- assets validate --json",
  "evidence_sha": "<full-40-char-sha>",
  "exit_status": 0,
  "summary": "All 7 assets valid",
  "passed": true
}
```

| Field | Type | Rule |
| --- | --- | --- |
| `name` | string | Must match the expected evidence name exactly |
| `command` | string | Must match the expected command exactly |
| `evidence_sha` | string | Must be a valid 40-char SHA matching `validation_sha` |
| `exit_status` | integer | Must be `0` for a passing result |
| `summary` | string | Must be non-empty; max 400 characters; no control chars |
| `passed` | boolean | Must be `true` |

### Required evidence entries

| Evidence name | Required command |
| --- | --- |
| `asset validation` | `cargo run -q -p eatme-cli -- assets validate --json` |
| `generated Gadugi freshness` | `cargo run -q -p eatme-cli -- assets generate-gadugi --check --json` |
| `repository quality gates` | `TMPDIR=/tmp NODE_OPTIONS=--max-old-space-size=32768 ./scripts/quality-gates.sh` |
| `documentation build` | `mkdocs build --strict` |

Each evidence entry must name the exact current HEAD SHA. Stale evidence (naming
a different SHA) is a blocker.

## Quality-audit cycles

At least three quality-audit cycles are required. Each cycle must include all
three phases (SEEK, VALIDATE, FIX) and name the exact current HEAD SHA:

```json
{
  "cycle_number": 1,
  "phases": ["Seek", "Validate", "Fix"],
  "outcome": "Clean",
  "head_sha": "<full-40-char-sha>",
  "summary": "All checks pass at current HEAD"
}
```

| Field | Type | Rule |
| --- | --- | --- |
| `cycle_number` | integer | Must be contiguous and strictly increasing from 1 |
| `phases` | array | Must contain `Seek`, `Validate`, and `Fix` |
| `outcome` | enum | `Clean` or `FixApplied` |
| `head_sha` | string | Must match `validation_sha` exactly |
| `summary` | string | Must be non-empty; max 400 characters; no control chars |

The final cycle must have outcome `Clean`. Earlier cycles may have outcome
`FixApplied` if issues were found and corrected.

## Diff scope and docs impact

### Diff scope

The `diff_scope` field declares which files changed in the PR:

```json
{
  "changed_files": [
    "crates/eatme-cli/src/pr_readiness.rs",
    "docs/default-workflow-pr-readiness.md"
  ],
  "focused": true
}
```

`focused` must be `true`. Every file in `changed_files` must be within the
allowed recovery scope:

| Allowed path pattern | Example |
| --- | --- |
| `crates/eatme-cli/src/pr_readiness*` | `crates/eatme-cli/src/pr_readiness/recovery.rs` |
| `crates/eatme-cli/src/main.rs` | exact path |
| `crates/eatme-core/src/command.rs` | exact path |
| `src/eatme_uvx/cli.py` | exact path |
| `docs/default-workflow-pr-readiness.md` | exact path |
| `docs/index.md` | exact path |
| `.pre-commit-config.yaml` | exact path |
| `pyproject.toml` | exact path |
| `scripts/check-module-size.sh` | exact path |
| `assets/scenarios/eatme/*.yaml` | scenario YAML files |
| `assets/scenarios/gadugi/*.yaml` | Gadugi adapter YAML files |

Files outside this scope produce a blocker.

### Docs impact

```json
{
  "docs_changed": true,
  "strict_build_required": true
}
```

When `docs_changed` is `true`, `strict_build_required` must also be `true`, and
the `documentation_build` evidence must pass. Otherwise the evaluator adds a
blocker.

## Change outcome

The `change_outcome` field declares whether the recovery required code changes:

**No-op** (no repository modifications):

```json
{ "NoOp": { "justification": "All audit cycles found codebase clean at <sha>" } }
```

The justification must be non-empty, max 400 characters, and contain no control
characters.

**Files modified** (code changes were made):

```json
{ "FilesModified": ["crates/eatme-cli/src/pr_readiness/recovery.rs"] }
```

When `FilesModified` is used, the validation SHA must appear in the file list
context. Each path must be max 240 characters with no control characters.

## Output report

The evaluator produces a `RecoveryReadinessReport`:

```json
{
  "status": "MERGE_READY",
  "branch": "wave7-eatme-nonclaim-audit-1778303500",
  "expected_remote_head_sha": "a5cfaa841856d5f4e5f2e1ef55be96628cf7d4b5",
  "final_head_sha": "a5cfaa841856d5f4e5f2e1ef55be96628cf7d4b5",
  "validation_status": "passed for exact current HEAD a5cfaa841856d5f4e5f2e1ef55be96628cf7d4b5",
  "change_outcome": { "NoOp": { "justification": "..." } },
  "required_github_checks": ["quality-gates"],
  "github_checks": [ "...CheckSummary[]..." ],
  "qa_evidence": [ "...RecoveryValidationEvidence[]..." ],
  "quality_audit_cycles": [ "...QualityAuditCycle[]..." ],
  "diff_scope": { "changed_files": ["..."], "focused": true },
  "docs_impact": { "docs_changed": true, "strict_build_required": true },
  "pr_description_evidence": { "..." },
  "wrapper_failures": [],
  "blockers": []
}
```

| Field | Description |
| --- | --- |
| `status` | `MERGE_READY` when `blockers` is empty; `NOT_MERGE_READY` otherwise |
| `branch` | Sanitized branch name |
| `expected_remote_head_sha` | The SHA that all evidence must target |
| `final_head_sha` | The local HEAD SHA at evaluation time |
| `validation_status` | Human-readable status string naming the exact HEAD |
| `change_outcome` | Sanitized echo of the input change outcome |
| `required_github_checks` | Sanitized list of required check names |
| `github_checks` | Sanitized check summaries from the snapshot |
| `qa_evidence` | Sanitized evidence for all four required QA gates |
| `quality_audit_cycles` | Sanitized audit cycle records |
| `diff_scope` | Sanitized diff scope |
| `docs_impact` | Echo of docs impact |
| `pr_description_evidence` | Echo of PR description evidence |
| `wrapper_failures` | Historical wrapper failures (context only, not readiness evidence) |
| `blockers` | List of human-readable blocker strings; empty means merge-ready |

### Plain-text report

Without `--json`, the report renders as structured plain text:

```text
MERGE_READY
Branch: wave7-eatme-nonclaim-audit-1778303500
Expected remote HEAD: a5cfaa841856d5f4e5f2e1ef55be96628cf7d4b5
Final HEAD: a5cfaa841856d5f4e5f2e1ef55be96628cf7d4b5
Validation status: passed for exact current HEAD a5cfaa841856d5f4e5f2e1ef55be96628cf7d4b5
No-op justification: All audit cycles found codebase clean at a5cfaa8...
Required GitHub checks: quality-gates
GitHub checks:
- quality-gates: status=completed conclusion=success head=a5cfaa8... required_flag=true
QA evidence:
- asset validation: passed=true exit=0 head=a5cfaa8... command=`cargo run -q -p eatme-cli -- assets validate --json` summary=All 7 assets valid
...
Quality-audit evidence:
- cycle 1: outcome=Clean head=a5cfaa8... phases=SEEK,VALIDATE,FIX summary=...
...
```

When blockers exist, they appear at the end:

```text
NOT_MERGE_READY
...
Blockers:
- validation evidence for abc123... is stale for current HEAD def456...
- required GitHub Actions check quality-gates is not green at def456...
```

## Blocker categories

The evaluator checks these categories in order. Any failure adds a blocker
string to the report:

| Category | Example blocker |
| --- | --- |
| Schema version | `schema_version must be pr-readiness-recovery.v1, got v2` |
| PR baseline | `recovery baseline is PR #204, got PR #199` |
| Branch match | `PR #204 must be on branch wave7-..., but local branch is main` |
| SHA alignment | `PR #204 baseline requires GitHub PR head, local HEAD, and validation SHA to equal expected remote head` |
| Validation freshness | `validation evidence for <old> is stale for current HEAD <new>` |
| Input safety | `<field> contains control characters or newlines` |
| Input length | `<field> exceeds <limit> characters` |
| Required check names | `required_github_checks must name trusted required GitHub checks` |
| Check head evidence | `GitHub Actions check <name> is for <old>, not exact head <sha>` |
| Check results | `required GitHub Actions check <name> is not green at <sha>` |
| Mergeability | `PR #204 is not merge-ready: mergeStateStatus=BLOCKED mergeable=CONFLICTING` |
| Evidence name | `expected evidence named asset validation, got assets` |
| Evidence command | `asset validation must use command <expected> at <sha>, got <actual>` |
| Evidence SHA | `asset validation evidence names <old>, but final validation requires <new>` |
| Evidence result | `asset validation did not pass at <sha>` |
| Audit cycle count | `three SEEK/VALIDATE/FIX quality-audit cycles are required` |
| Audit cycle phases | `quality-audit cycle 1 must include SEEK, VALIDATE, and FIX phases` |
| Audit final clean | `final cycle clean quality-audit outcome is required` |
| Diff scope focus | `focused diff scope evidence is required` |
| Diff scope path | `focused diff scope excludes unrelated path <path>` |
| Docs impact | `docs impact requires strict documentation build evidence` |
| PR description | `pr_description_evidence must name the validation SHA` |
| Stale evidence | `stale_evidence_handled must be true` |
| Change outcome | No-op justification empty or FilesModified validation |

## Safety and sanitization

All report output is sanitized before rendering:

- **Control characters** are normalized to spaces; consecutive whitespace is
  collapsed
- **Secret markers** are redacted to `[REDACTED]`: GitHub PATs (`ghp_`,
  `github_pat_`), OpenAI keys (`sk-`), Slack tokens (`xoxb-`, `xoxp-`), AWS
  access keys (`AKIA...`), and query-string credentials (`token=`, `api_key=`,
  `apikey=`, `secret=`)
- **Long strings** are truncated at 512 characters with `...` appended

Input safety is checked before evaluation:

| Field | Limit |
| --- | --- |
| Evidence summaries | 400 characters, no control chars |
| Quality-audit cycle summaries | 400 characters, no control chars |
| Diff scope changed paths | 240 characters, no control chars |
| No-op justification | 400 characters, no control chars |
| FilesModified paths | 240 characters, no control chars |

Violations produce blockers, not silent truncation.

## Forbidden claims

The following claims are forbidden in PR documentation unless they appear in an
explicit nonclaim context (paragraph containing `nonclaims`, `does not validate`,
`does not claim`, `do not claim`, `must not imply`, `cannot convert`,
`unsupported claim`, or `not validate`):

- full Alice UI automation
- full UI automation
- full world execution
- UI rendering correctness
- visible rendering correctness
- grading
- creative assessment
- Save completion
- deployed sharing/platform success
- first-lesson completion
- full lesson completion
- complete Alice coverage
- full Tweedle/player decode

The `validate_pr_204_documentation` function enforces this: any forbidden claim
found outside a nonclaim paragraph is a validation error.

## End-to-end workflow

### Step 1: Collect the GitHub snapshot

```bash
SHA=$(git rev-parse HEAD)

cargo run -q -p eatme-cli -- pr-readiness github-snapshot \
  --owner rysweet \
  --repo eatme \
  --pr-number 204 \
  --local-head-sha "$SHA" \
  --required-check quality-gates \
  --json > /tmp/snapshot.json
```

### Step 2: Run local quality gates

```bash
cargo run -q -p eatme-cli -- assets validate --json
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
mkdocs build --strict
TMPDIR=/tmp NODE_OPTIONS=--max-old-space-size=32768 ./scripts/quality-gates.sh
```

### Step 3: Compose the recovery input JSON

Build a `RecoveryReadinessInput` JSON file incorporating the snapshot from step 1,
evidence from step 2, at least three quality-audit cycles, the diff scope, docs
impact, PR description evidence, and the change outcome. Every SHA field must
name the exact current HEAD.

### Step 4: Evaluate

```bash
cargo run -q -p eatme-cli -- pr-readiness recovery-evaluate \
  --input /tmp/recovery-input.json \
  --json
```

The command prints the report and exits 0 for `MERGE_READY` or non-zero for
`NOT_MERGE_READY`.

### Step 5: Update the PR description

Use `render_review_note` output or the JSON report to update the PR description
with current-head evidence, check rollup summary, and bounded nonclaims. Label
any older tested-head evidence as stale/non-current.

### Step 6: Final gate verification

Before merge, verify that:

- The evidence SHA still matches the PR head (`verify_final_gate`)
- The latest review note names the exact SHA
- The latest review note labels older evidence stale/non-current
- The PR head has not changed since evidence collection

## Configuration reference

### Schema version

The only accepted schema version is `pr-readiness-recovery.v1`.

### Required GitHub checks

At least one required check must be named. The evaluator verifies that each
named check has `status=Completed` and `conclusion=Success` for the exact
validation SHA.

### Mergeability requirements

The PR must have `mergeStateStatus=CLEAN` and `mergeable=MERGEABLE`. Any other
combination is a blocker.

### Text limits

| Context | Character limit |
| --- | --- |
| Evidence summary | 400 |
| Audit cycle summary | 400 |
| No-op justification | 400 |
| Changed file path | 240 |
| Report render value | 512 (truncated, not blocked) |

### Retry configuration

The GitHub snapshot fetcher uses:

| Parameter | Value |
| --- | --- |
| Timeout | 20 seconds |
| Retry attempts | 3 |
| Retry delay | 500 ms |

## Nonclaims

The recovery evaluation system does not validate:

- full Alice UI automation
- full world execution
- UI rendering correctness or visible rendering correctness
- grading or creative assessment
- Save completion or first-lesson completion
- deployed sharing/platform success
- complete Alice coverage
- full Tweedle/player decode

These are explicit nonclaims. The system evaluates merge-readiness evidence for
a focused recovery scope only.
