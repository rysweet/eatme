# Default-workflow PR readiness

Default-workflow PR readiness is the exact-head gate used when a pull request
needs a clear final readiness decision and the wrapper workflow did not produce
useful output.

This page specifies the readiness behavior the workflow should enforce: verify
the exact PR head, inspect GitHub metadata for that same head, preserve the
bounded starter-project evidence contract, require fresh generated Gadugi
adapters when scenario assets are involved, and publish a narrowly scoped
readiness comment only after every gate passes.

Repository-local command results are evidence for this workflow, not a complete
PR readiness decision by themselves. Full default-workflow readiness also
requires exact-head GitHub checks, merge state, mergeability, and scope gates.

## Contents

- [Readiness contract](#readiness-contract)
- [Generic readiness procedure](#generic-readiness-procedure)
- [Configuration](#configuration)
- [GitHub metadata fields](#github-metadata-fields)
- [Starter-project evidence boundary](#starter-project-evidence-boundary)
- [Executable starter-project boundary check](#executable-starter-project-boundary-check)
- [Generated Gadugi adapter freshness](#generated-gadugi-adapter-freshness)
- [PR #164 readiness example](#pr-164-readiness-example)
- [Readiness comment](#readiness-comment)
- [Blocker handling](#blocker-handling)

## Readiness contract

A PR is default-workflow ready only when every gate passes for the exact commit
being reviewed.

| Gate | Required result |
| --- | --- |
| Exact head | The PR head SHA equals the requested SHA. A mismatch blocks readiness. |
| GitHub checks | Required checks are green for that same SHA. |
| Merge state | `mergeStateStatus` is `CLEAN`. |
| Mergeability | `mergeable` is `MERGEABLE`. |
| Starter-project wording | The canonical scenario uses plain, bounded, user-facing language. |
| Overclaim boundary | The scenario does not claim first-lesson completion, grading, creative assessment, full UI automation, visible rendering correctness, full Save completion, or complete Alice coverage. |
| Gadugi adapters | Generated adapters are fresh whenever canonical scenario assets are affected. |
| Scope | No unrelated files or behavior are changed. |

A previous wrapper failure is not a blocker when direct verification proves the
same head, green checks, clean mergeability, bounded wording, and fresh
generated adapters.

If only local executable gates were run, describe the result as
repository-local exact-head evidence. Do not make a final PR-readiness or
mergeability claim until the GitHub and mergeability gates above also pass for
the same commit.

## Generic readiness procedure

Run the gate in this order:

1. Verify the PR head equals the exact requested SHA.
2. Verify GitHub checks are green for that same SHA.
3. Verify `mergeStateStatus=CLEAN` and `mergeable=MERGEABLE`.
4. Inspect the starter-project preflight scenario wording if the PR touches that
   evidence contract.
5. Run the generated Gadugi adapter freshness check if any canonical scenario
   asset is affected.
6. Validate assets.
7. Publish the readiness comment only when every required gate passed.

## Configuration

Run commands from the repository root.

If running Node-based workflow wrappers, set the repository's large-heap Node
option before invoking the wrapper:

```bash
export NODE_OPTIONS=--max-old-space-size=32768
```

The Rust asset validation and Gadugi generator commands do not require Node, but
the environment variable is safe to keep exported for repository-wide workflow
commands.

For GitHub checks, use authenticated `gh` access to the repository that owns the
PR. Do not place tokens, secrets, local credential paths, environment dumps, or
raw command output in readiness comments.

## GitHub metadata fields

The readiness gate consumes these `gh pr view` fields:

| Field | Required value |
| --- | --- |
| `headRefOid` | Exact requested SHA |
| `mergeStateStatus` | `CLEAN` |
| `mergeable` | `MERGEABLE` |
| `statusCheckRollup` | Required checks green for `headRefOid` |

Fetch the PR head, merge state, mergeability, and check summary:

```bash
gh pr view 164 \
  --json headRefOid,mergeStateStatus,mergeable,statusCheckRollup
```

`statusCheckRollup` is green only when every required check for `headRefOid` has
completed successfully. A required check blocks readiness when it is pending,
queued, in progress, requested, failing, errored, timed out, skipped when the
branch protection requires it to run, cancelled, missing, or reported for a
different head.

If the head changes during review, stop and restart the readiness verification
for the newly requested SHA.

## Starter-project evidence boundary

Review the canonical source scenario when starter-project preflight wording is
part of the PR:

```text
assets/scenarios/eatme/starter-project-open-save-export-preflight.yaml
```

The wording must stay plain and bounded. It may say that the scenario records
real Alice launch/opened-project evidence for the bundled starter project, an
editable starter-world change note, an attempted run or observation, and
readiness-gap notes.

When older wording or generated output uses action-oriented evidence shorthand,
read it only as bounded launch/opened-project evidence. It does not mean
user-like UI automation, save/reopen/export completion, learner-world grading,
or creative assessment.

The wording must not say or imply that the scenario proves:

| Unsupported claim | Required boundary |
| --- | --- |
| First-lesson completion | It is starter-project preflight evidence only. |
| Grading or learner-world grading | It records evidence for review; it does not grade. |
| Creative assessment | It may name an editable change; it does not assess creativity. |
| Full UI automation | It records bounded launch/opened-project evidence and explicit gaps. |
| Visible rendering correctness | Screenshot or window evidence is observation evidence only. |
| Full Save completion | Save, reopen, and export remain readiness gaps until user-like evidence exists. |
| Complete Alice coverage | The scenario covers only the stated preflight contract. |

Use the generated adapter only as a consumer of this contract. Do not hand-edit
generated Gadugi YAML to change mission intent.

## Executable starter-project boundary check

The current focused Rust test surface for starter-project preflight boundary
wording is:

```text
crates/eatme-assets/src/starter_project_preflight_boundary_tests.rs
```

Run the contract check directly with:

```bash
cargo test -p eatme-assets starter_project_preflight_boundary
```

That test validates the canonical scenario YAML, generated Gadugi adapter
wording, and scoped starter-project/preflight evidence documentation. It checks
that those executable assets and scoped docs use bounded, user-facing language
for launch/opened-project evidence, editable starter-world change notes,
attempted run or observation evidence, generated adapter freshness, asset
validation, and explicit readiness gaps.

The documentation-overclaim check inspects this source contract and the scoped
starter-project evidence page:

```text
docs/default-workflow-pr-readiness.md
docs/starter-project-preflight-evidence.md
```

The documentation-overclaim check fails only on narrow readiness or evidence
overclaim phrases, not broad negative statements that explain what the scenario
does not prove. Prohibited phrases are:

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

Failure output names the violating file, the matched phrase, this source
contract, and the bounded replacement wording.

The check is intentionally narrow. It does not prove pull request readiness,
mergeability, production suitability, complete lesson execution, user-like Alice
UI coverage, save/reopen/export completion, grading, creative assessment,
visible rendering correctness, or complete Alice coverage. Those claims require
their own evidence and gates.

## Generated Gadugi adapter freshness

Whenever a canonical scenario asset changes, the generated Gadugi adapter
freshness check is mandatory:

```bash
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
```

If the check reports stale or missing generated output, regenerate adapters and
run check mode again:

```bash
cargo run -q -p eatme-cli -- assets generate-gadugi --json
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
```

Commit the canonical scenario change and regenerated adapter change together.
When no scenario asset or generated adapter target is affected, adapter freshness
is not part of the readiness decision.

Validate committed scenario and persona assets:

```bash
cargo run -q -p eatme-cli -- assets validate --json
```

The validation gate passes only when the JSON report has `passed: true` and no
blocking errors.

## PR #164 readiness example

This subsection is a concrete example for the PR #164 finalization gate. Do not
reuse its PR number or SHA for future readiness decisions.

For PR #164, the exact accepted head is:

```text
eb0bb29b7cc1f8647e9a36c0bc8200fb3fdc5cba
```

The GitHub metadata gate passes only when `gh pr view 164 --json
headRefOid,mergeStateStatus,mergeable,statusCheckRollup` reports:

```json
{
  "headRefOid": "eb0bb29b7cc1f8647e9a36c0bc8200fb3fdc5cba",
  "mergeStateStatus": "CLEAN",
  "mergeable": "MERGEABLE"
}
```

Because PR #164 changes starter-project scenario wording and generated Gadugi
output, these gates are mandatory for that PR:

```bash
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
cargo run -q -p eatme-cli -- assets validate --json
```

The readiness decision for PR #164 is valid only if those commands pass, the
GitHub checks are green for
`eb0bb29b7cc1f8647e9a36c0bc8200fb3fdc5cba`, and the scenario wording stays
within the starter-project evidence boundary above.

## Readiness comment

Publish readiness only after all required gates pass for the exact head. The
comment should name the head and avoid broader product-readiness claims.

Example:

```text
Default-workflow readiness recorded for PR #164 at exact head eb0bb29b7cc1f8647e9a36c0bc8200fb3fdc5cba.

Verified gates: exact PR head, green GitHub checks for that head, mergeStateStatus=CLEAN, mergeable=MERGEABLE, bounded starter-project preflight wording, no unsupported claims for first-lesson completion/grading/creative assessment/full UI automation/visible rendering correctness/full Save completion, generated Gadugi adapter freshness, and asset validation.

The prior non-zero wrapper exit is not treated as a blocker because direct verification passed at this exact head.
```

Post the comment with:

```bash
gh pr comment 164 --body-file readiness-comment.txt
```

## Blocker handling

If any gate fails, do not publish readiness. Fix only the minimal issue that
caused the blocker, run the relevant validation again, push the fix, and repeat
exact-head verification against the new PR head.

| Blocker | Minimal response |
| --- | --- |
| Head mismatch | Stop readiness for the old SHA and verify the requested new head. |
| Failing, pending, cancelled, missing, or wrong-head checks | Fix the failing check, wait for completion, or rerun the missing check before readiness. |
| Dirty merge state | Resolve only the mergeability issue. |
| Overclaiming scenario language | Edit the canonical scenario wording and regenerate adapters if affected. |
| Stale generated adapter | Regenerate adapters from canonical sources. |
| Asset validation failure | Fix the invalid scenario or persona asset. |
| Unrelated changes | Remove the unrelated change from the readiness work. |
