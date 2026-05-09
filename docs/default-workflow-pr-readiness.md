# Default-workflow PR readiness

Default-workflow PR readiness is the exact-head gate used when a pull request
needs a final readiness decision without a manual merge.

For scenario-link recovery work, the gate establishes only that the current
branch head is reviewable, generated assets are reproducible, documentation
builds, and the first-lesson evidence path stays bounded. It does not establish
full UI automation, rendering correctness, grading, creative assessment, Save
completion, lesson completion, or broad Alice compatibility.

## Contents

- [Readiness contract](#readiness-contract)
- [Generic readiness procedure](#generic-readiness-procedure)
- [Configuration](#configuration)
- [GitHub metadata fields](#github-metadata-fields)
- [Scenario-link evidence boundary](#scenario-link-evidence-boundary)
- [Generated Gadugi adapter freshness](#generated-gadugi-adapter-freshness)
- [Scenario-link recovery procedure](#scenario-link-recovery-procedure)
- [Review and finalization packet](#review-and-finalization-packet)
- [Readiness note](#readiness-note)
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
| Scenario-link wording | Canonical scenarios use plain, bounded, user-facing prerequisite, evidence, and follow-on language. |
| Overclaim boundary | Scenario, generated runner, and docs wording do not claim first-lesson completion, grading, creative assessment, full UI automation, visible rendering correctness, full Save completion, or complete Alice coverage. |
| Gadugi adapters | Generated adapters are fresh whenever canonical scenario assets or generator output are affected. |
| Documentation | MkDocs builds in strict mode when documentation is changed. |
| Quality gate | The repository quality gate passes for the current head before finalization. |
| Scope | No unrelated files or behavior are changed. |

A previous wrapper failure is not a blocker when direct verification confirms the
same head, green checks, clean mergeability, bounded wording, and fresh
generated adapters. The workflow records readiness; it does not merge the pull
request.

## Generic readiness procedure

Run the gate in this order:

1. Verify the PR head equals the exact requested SHA.
2. Verify GitHub checks are green for that same SHA.
3. Verify `mergeStateStatus=CLEAN` and `mergeable=MERGEABLE`.
4. Inspect scenario-link wording if the PR touches canonical scenarios,
   generated adapters, or docs that describe the first-lesson evidence path.
5. Run the generated Gadugi adapter freshness check if any canonical scenario
   asset or generator output is affected.
6. Validate assets.
7. Build docs in strict mode when docs are changed.
8. Run the repository quality gate.
9. Prepare the readiness note only when every required gate passed.

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

Deep worktrees can exceed Unix socket path limits during the full repository
quality gate. Run that gate with a short temporary directory:

```bash
TMPDIR=/tmp ./scripts/quality-gates.sh
```

For GitHub checks, use authenticated `gh` access to the repository that owns the
PR. Do not place tokens, secrets, local credential paths, environment dumps, or
raw command output in readiness notes.

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
PR_NUMBER="${PR_NUMBER:?set PR_NUMBER to the pull request number}"
gh pr view "$PR_NUMBER" \
  --json headRefOid,mergeStateStatus,mergeable,statusCheckRollup
```

`statusCheckRollup` is green only when every required check for `headRefOid` has
completed successfully. A required check blocks readiness when it is pending,
queued, in progress, requested, failing, errored, timed out, skipped when the
branch protection requires it to run, cancelled, missing, or reported for a
different head.

If the head changes during review, stop and restart the readiness verification
for the newly requested SHA.

## Scenario-link evidence boundary

Review canonical source scenarios when scenario-link, prerequisite, evidence, or
follow-on wording is part of the PR:

```text
assets/scenarios/eatme/
```

The canonical scenario YAML is the source of truth. Generated Gadugi YAML is a
consumer of that source and must not add independent behavior. Reader-facing
scenario links may connect prerequisites, first-lesson evidence, instructor
handoff, student next action, and follow-on paths only when those links are
represented by the editable scenario assets or generated docs.

Use plain wording for the first-lesson silver thread:

| Link surface | Acceptable claim |
| --- | --- |
| Prerequisites | Required tools, Alice homes, real-Alice gate, and scenario assets are named before execution. |
| Evidence | The run records only the manifest, log, window, screenshot, action-contract, and readiness fields named by the scenario or report. |
| Follow-on path | The next step is a bounded action or handoff, not a declaration that the lesson is complete. |
| Generated runner | Gadugi invokes eatme commands and checks emitted evidence; eatme owns Alice desktop launch behavior. |
| Documentation | Docs route readers from scenario authoring to generated runners, lesson readiness, and evidence interpretation without broad product claims. |

The wording must not say or imply that the scenario, generated adapter, or docs
claim:

| Unsupported claim | Required boundary |
| --- | --- |
| First-lesson completion | It is first-lesson readiness or first-action evidence only. |
| Grading or learner-world grading | It records evidence for review; it does not grade. |
| Creative assessment | It may name an editable change; it does not assess creativity. |
| Full UI automation | It records bounded launch evidence, action-contract evidence, and explicit gaps. |
| Visible rendering correctness | Screenshot or window evidence is observation evidence only. |
| Full Save completion | Save, reopen, and export remain readiness gaps until user-like evidence exists. |
| Complete Alice coverage | The scenario covers only its stated readiness contract. |

For starter-project preflight wording, keep the same boundary: opened starter
project, manifest/log/window/screenshot evidence, bounded starter-world notes,
and readiness-gap artifacts only.

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
When no scenario asset or generated adapter target is affected, adapter
freshness is not part of the readiness decision.

Validate committed scenario and persona assets:

```bash
cargo run -q -p eatme-cli -- assets validate --json
```

The validation gate passes only when the JSON report has `passed: true` and no
blocking errors.

## Scenario-link recovery procedure

Use this procedure when a pull request touches scenario links, generated runner
wording, or first-lesson evidence-path documentation. The recovery gate treats
the existing branch as the review surface. It records readiness for the checked
head and does not merge the pull request manually.

Use this bounded command set for the branch:

```bash
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
cargo run -q -p eatme-cli -- assets validate --json
mkdocs build --strict
TMPDIR=/tmp ./scripts/quality-gates.sh
```

The scenario-link recovery is ready only when:

1. The PR head matches the branch head being reviewed.
2. Canonical scenario links, prerequisites, learner-facing boundaries, and
   follow-on paths stay in `assets/scenarios/eatme/`.
3. Generated Gadugi adapters are reproducible from the generator.
4. Generated adapter descriptions use bounded evidence-source wording.
5. Scenario-link silver-thread tests cover reader routing without implying UI
   rendering, grading, creative assessment, or lesson completion.
6. Documentation states only executable checked-head evidence boundaries.

If the working tree already contains exactly the required generator, generated
adapter, test, and documentation changes, keep them and complete the checks. If
the current branch already satisfies every gate without additional repository
changes, record a no-op finalization instead of editing unrelated files.

## Review and finalization packet

The finalization packet is the evidence summary used by reviewers. It should be
short and tied to current-head checks, not to expected future behavior.

Include:

| Field | Content |
| --- | --- |
| Branch | The reviewed branch name. |
| Head | The exact commit SHA checked by GitHub metadata and local commands. |
| Scope | Scenario-link silver thread, generated Gadugi adapter wording, tests, and docs. |
| Commands | The repository-native commands that passed for that head. |
| Boundaries | Explicit non-claims for full UI automation, rendering correctness, grading, creative assessment, Save completion, lesson completion, and broad Alice coverage. |
| Merge handling | State that the workflow recorded readiness and did not manually merge. |
| Implementation output | Include `Files modified` when repository files changed, or `No-op justification:` when no further edit is needed. |

A no-op finalization is acceptable only when the branch is already clean or the
only dirty files are intentionally preserved generated/test/doc changes required
by the recovery, all required current checks pass for the current head, and no
additional repository edits would change the readiness result.

## Readiness note

Prepare readiness only after all required gates pass for the exact head. The
note should name the head and avoid broader product-readiness claims.

Example:

```text
Default-workflow readiness recorded for exact head <head-sha>.

Verified gates: exact PR head, green GitHub checks for that head, mergeStateStatus=CLEAN, mergeable=MERGEABLE, bounded scenario-link wording, generated Gadugi adapter freshness, asset validation, strict documentation build, and repository quality gate.

Scope: scenario-link silver-thread asset/docs/generator validation only. This does not claim full UI automation, rendering correctness, grading, creative assessment, Save completion, lesson completion, or broad Alice compatibility.

Files modified: <changed files, or `No-op justification:` with the checked-head reason>
```

## Blocker handling

If any gate fails, do not record readiness. Fix only the minimal issue that
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
| Documentation build failure | Fix the broken doc link, heading, nav entry, or markdown issue. |
| Quality gate failure | Fix the failing repository-native check without broadening readiness claims. |
| Unrelated changes | Remove the unrelated change from the readiness work. |
