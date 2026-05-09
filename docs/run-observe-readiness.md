# Run/observe readiness evidence

Run/observe readiness records bounded evidence for the Alice automation scenario
when it reaches a Run or observe step. It reports what the selected scenario and
run have shown, what remains not yet shown, and which claims remain explicitly
unproven.

Use this page when you need to review starter-project preflight evidence,
first-lesson run/observe evidence, generated Gadugi adapter behavior, or the
git-linked no-op guard used by TDD recovery checks.

## Scope

Run/observe readiness is an evidence contract, not a completion claim.

| Evidence | Meaning |
| --- | --- |
| Setup evidence | The selected scenario, run id, and required repository assets were found. |
| Run dispatch evidence | The automation scenario reached the bounded Run lane and recorded the dispatch or attempted dispatch it owns. |
| Observe-step evidence | The automation scenario reached the observe boundary and recorded available log, window, screenshot, or report evidence. |
| Gap evidence | The report names missing user-facing Run-window state, missing observe-state evidence, and other not-yet-shown evidence. |
| Unproven claims | The report keeps unsupported completion, world-execution, grading, rendering, Save, deployment/sharing, and automation claims visible. |

The contract does not claim full world execution, visible rendering correctness,
grading, creative assessment, Save completion, full UI automation, deployed
sharing/platform success, or first-lesson completion.

## Configuration

Real Alice desktop evidence is opt-in:

```bash
export EATME_REAL_ALICE=1
export ALICE_HOME=/path/to/alice-checkout
```

Node-backed wrappers and agent runners can use the saved heap preference:

```bash
export NODE_OPTIONS=--max-old-space-size=32768
```

The Rust CLI does not require `NODE_OPTIONS`, but wrappers that invoke Gadugi or
other Node-based tooling should preserve it when running the same workflow.

## Git-linked no-op guard

The TDD no-op guard resolves the active repository with Git before deciding
whether a recovery run has changed anything:

```bash
git rev-parse --show-toplevel
```

Run the guard from the linked worktree that owns the PR branch. The guard uses
that resolved root for all later diff checks. If `git rev-parse --show-toplevel`
fails, the guard fails closed instead of checking an unrelated directory or
treating the run as a clean no-op.

For PR recovery, use the published PR head branch as the source of truth. If only
the remote branch exists locally, create a local tracking branch from that remote
instead of creating a replacement branch. Fetch current `origin/master` before
recording readiness. If `origin/master` is already an ancestor of the PR head,
record that no rebase or merge was needed. If it is not an ancestor, merge
`origin/master` into the PR branch and keep the published PR history intact; do
not rebase the PR branch for recovery evidence.

## Starter-project preflight usage

Run the starter-project preflight when you need bounded open/start evidence
before later save, reopen, export, or classroom-readiness review:

```bash
EATME_REAL_ALICE=1 cargo run -q -p eatme-cli -- alice launch-smoke \
  --alice-home "${ALICE_HOME}" \
  --scenario starter-project-open-save-export-preflight \
  --run-id local-starter-project-open-save-export-preflight \
  --runs-dir runs \
  --timeout 900 \
  --json \
  --no-memory \
  --offline-package
```

The scenario writes run artifacts under:

```text
runs/starter-project-open-save-export-preflight/local-starter-project-open-save-export-preflight/
```

The durable starter-project evidence files are:

```text
manifest.json
alice.log
window-list.txt
screenshots/startup.png
starter-world-change-note.txt
run-observe-readiness-gaps.txt
starter-project-readiness-report.txt
```

Treat `starter-project-readiness-report.txt` as a plain handoff. It summarizes
launch evidence, the starter-world change note, the attempted run or observation,
and the save/reopen/export/readiness gaps. It does not upgrade any gap into a
completed journey.

## First-lesson readiness usage

Check an existing first-lesson comparison manifest:

```bash
cargo run -q -p eatme-cli -- alice check-lesson-readiness \
  --manifest runs/comparisons/first-lessons-real-ui-actions/local/comparison-manifest.json
```

Run the bounded comparison and readiness sequence when both Alice targets are
available:

```bash
export NODE_OPTIONS=--max-old-space-size=32768
export ALICE_BASELINE_HOME=/path/to/original-alice
export ALICE_MODERNIZED_HOME=/path/to/rabbithole-alice

EATME_REAL_ALICE=1 cargo run -q -p eatme-cli -- alice run-first-lesson-readiness \
  --run-id local-first-lesson-readiness \
  --json \
  --no-memory \
  --offline-package \
  --execute
```

Read the plain report in this order:

1. `Desktop proof`
2. `Shown`
3. `Not yet shown`
4. `Desktop next action`, when present
5. `Unproven`

Do not treat `Shown` entries as evidence for any claim outside the entry's own
bounded wording.

## Missing Run/observe states

Run/observe readiness keeps missing Run and observe states visible as separate
user-facing facts. A report that reaches the observe boundary can still be
`not_ready` when either state is missing:

| Missing state | Required wording |
| --- | --- |
| Run-window state | `User-facing Run-window state is not yet shown for the automation scenario observe step.` |
| Observe-state evidence | `User-facing observe-state evidence is not yet shown for the selected scenario and run.` |

Do not collapse either gap into a generic failure, and do not use the presence of
one state as proof of the other.

## JSON API

`alice check-lesson-readiness --json` emits
`eatme.alice-lesson-session-readiness/v1`. Run/observe readiness consumers use
these fields:

| Field | Type | Meaning |
| --- | --- | --- |
| `status` | string | `ready`, `not_ready`, or `blocked` for the bounded readiness check. |
| `desktop_proof_contract` | object | Modernized desktop proof status rendered as the plain `Desktop proof` line. |
| `shown_evidence` | array | User-facing evidence facts accepted for the selected scenario and run. |
| `not_yet_shown` | array | Missing, invalid, not-observed, or blocked evidence states. |
| `desktop_next_action` | object or omitted | Safe RabbitHole next-action summary when current and applicable. |
| `unproven_claims` | array | Required non-claims that remain visible in plain output and JSON. |
| `evidence_boundaries` | array | Boundary-specific evidence states and safe artifact metadata. |
| `issues` | array | Structural problems that prevent trusting the evidence. |

Every `shown_evidence[]` and `not_yet_shown[]` item has this shape:

| Field | Type | Meaning |
| --- | --- | --- |
| `id` | string | Stable evidence id. |
| `state` | string | `present`, `missing`, `invalid`, `not_observed`, or `blocked`. |
| `summary` | string | Plain user-facing summary. |
| `detail` | string | Display-safe detail for automation and debugging. |
| `does_not_prove` | array | Claims that remain unsupported by this evidence item. |

Example bounded item:

```json
{
  "id": "run_observe_gap",
  "state": "missing",
  "summary": "User-facing Run-window state is not yet shown for the automation scenario observe step.",
  "detail": "The selected run reached the observe boundary, but no accepted Run-window state was linked to the scenario and run.",
  "does_not_prove": [
    "full world execution",
    "visible rendering correctness",
    "deployed sharing/platform success",
    "first-lesson completion"
  ]
}
```

## Generated Gadugi adapter contract

The canonical source stays under:

```text
assets/scenarios/eatme/
```

Generated Gadugi adapters stay under:

```text
assets/scenarios/gadugi/
```

For run/observe readiness, adapters invoke eatme commands, capture stdout and
stderr, and inspect manifest-level or report-level evidence. They do not take
ownership of Xvfb setup, Java process lifecycle, screenshot capture, Alice UI
internals, grading, creative review, Save completion, full world execution, or
deployed sharing/platform success.

After editing canonical scenario wording, refresh generated adapters:

```bash
cargo run -q -p eatme-cli -- assets generate-gadugi --json
```

Before review, check that generated adapters are current:

```bash
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
```

## Validation

Validate assets:

```bash
cargo run -q -p eatme-cli -- assets validate --json
```

Check generated Gadugi adapters:

```bash
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
```

Build the documentation site:

```bash
mkdocs build --strict
```

Run the repository quality gate when Rust code, scenario contracts, or generated
adapter behavior changed:

```bash
TMPDIR=/tmp ./scripts/quality-gates.sh
```

## Writing conservative readiness text

Use plain wording that names the evidence boundary:

| Say | Avoid |
| --- | --- |
| `Run dispatch evidence is shown for the selected scenario and run.` | `The world fully ran.` |
| `User-facing Run-window state is not yet shown.` | `Rendering is correct.` |
| `User-facing observe-state evidence is not yet shown.` | `The observation proves execution succeeded.` |
| `Save option/action evidence is an observation only.` | `Save completed.` |
| `Deployed sharing/platform success is not shown.` | `The project was shared successfully.` |
| `Creative assessment is not shown.` | `The project is creative enough.` |
| `First-lesson completion is not shown.` | `The first lesson is done.` |

The durable rule is to report only what the accepted evidence names, then keep
missing states and unproven claims visible.
