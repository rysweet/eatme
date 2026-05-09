# Starter project preflight evidence

The starter project preflight scenario records bounded real Alice evidence for
opening the bundled starter project before save, reopen, or export work is
reviewed.

Use this page when you need to run the scenario, edit its non-code wording,
inspect its evidence contract, or refresh the generated Gadugi adapter.

## Documentation contract

This page is scoped to the starter-project preflight evidence boundary defined
in [Default-workflow PR Readiness](default-workflow-pr-readiness.md). It may
describe evidence that the bundled starter project was launched and opened,
evidence that an editable starter-world change was named, attempted run or
observation evidence, generated adapter freshness, asset validation, and
readiness gaps that still require later proof.

Do not use this page to claim broader readiness. In particular, starter-project
preflight evidence is not pull request readiness, mergeability, production
suitability, complete lesson execution, user-like Alice UI coverage,
save/reopen/export completion, grading, creative assessment, visible rendering
correctness, or complete Alice coverage.

## Evidence boundary

`starter-project-open-save-export-preflight` records evidence that the eatme
harness launched real Alice with the bundled starter project and captured
inspectable evidence for that opened-project state.

The evidence boundary is intentionally narrow:

| Evidence | Meaning |
| --- | --- |
| Launch manifest | Identifies `starter-project-open-save-export-preflight` as the selected scenario. |
| Launch command | Shows that Alice was started with the bundled starter project, such as `africa.a3p`. |
| Assertions | Records deterministic harness assertions, including real Alice execution evidence. |
| Window or screenshot evidence | Shows that a smoke-ready Alice desktop session was observed. |
| Logs | Preserve Alice launch output for review and troubleshooting. |
| Starter-world change note | Names one small editable starter-world change for a later user-like Alice pass. |
| Run/observe readiness gaps | Treats log, window, screenshot, and report evidence as an attempted run or observation only, and names missing Run-window or observe-state evidence explicitly. |
| Starter-project readiness report | Gives instructor, student, or adapter reviewers a bounded handoff before later save, reopen, export, or classroom-readiness review. |

This scenario does not write `ui-action-contract.json`; that artifact belongs to
scenarios that explicitly exercise or specify user-like UI actions, such as
`first-lessons-real-ui-actions`. This scenario also does not provide full UI
automation, creative assessment, learner-world grading, or complete Alice
coverage. It also does not prove full world execution, visible rendering
correctness, deployed sharing/platform success, Save completion, or first-lesson
completion. It is preflight evidence for opening the starter project, not
evidence that a learner completed save, reopen, export, sharing, or
first-lesson work.

## Configuration

Set `ALICE_HOME` to the Alice checkout used by the smoke run:

```bash
export ALICE_HOME="${ALICE_HOME:-../alice3-modernization}"
```

Real Alice runs are explicit opt-in runs. Non-baseline scenarios require:

```bash
export EATME_REAL_ALICE=1
```

For Node-based wrappers or agent tooling only, a larger heap can avoid wrapper
runtime limits. The Rust CLI does not require this setting:

```bash
export NODE_OPTIONS=--max-old-space-size=32768
```

Keep local run artifacts outside commits. The default artifact root is:

```text
runs/
```

## Run the scenario

Run the preflight scenario with the bundled starter project contract:

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

The run writes artifacts under:

```text
runs/starter-project-open-save-export-preflight/local-starter-project-open-save-export-preflight/
```

Typical artifacts are:

```text
manifest.json
alice.log
xvfb.log
window-list.txt
screenshots/startup.png
starter-world-change-note.txt
run-observe-readiness-gaps.txt
starter-project-readiness-report.txt
```

Use the manifest, desktop artifacts, and readiness notes as bounded setup
evidence before asking an instructor, student, agent, or adapter to reason about
save, reopen, export, or classroom-readiness behavior.

## Manifest and artifact contract

The durable API for this scenario is the JSON output from
`alice launch-smoke` plus the manifest written to the run directory. Consumers
should use manifest fields and artifact paths, not private Alice internals.

| Field or artifact | Required use |
| --- | --- |
| `scenario_id` | Must equal `starter-project-open-save-export-preflight`. |
| `launch_command` | Must include the starter project passed to Alice. |
| `failure_category` | Must be `null` for a passing preflight run. |
| `assertions` | Must include passing deterministic launch and real Alice execution evidence. |
| `screenshot.path` or `window_list.path` | Must point to non-empty evidence for the opened desktop session. |
| `log.path` | Must point to a non-empty Alice log artifact. |
| `starter-world-change-note.txt` | Names a later small editable change without grading creativity or learner work. |
| `run-observe-readiness-gaps.txt` | Names the attempted run or observation, missing Run-window or observe-state evidence, and the save/reopen/export/sharing/readiness gaps. |
| `starter-project-readiness-report.txt` | Summarizes launch evidence, the starter-world change note, the attempted run or observation, and remaining gaps. |

Adapters and review scripts should report failure when eatme reports failure.
They should not infer pass or failure by replaying Alice implementation details.

## Edit the scenario wording

The editable source is the canonical scenario YAML:

```text
assets/scenarios/eatme/starter-project-open-save-export-preflight.yaml
```

Non-coders can update scenario prose in these existing fields:

| Field | What to edit |
| --- | --- |
| `purpose` | A short statement of the evidence the scenario owns. |
| `smoke_ready.evidence` | Observable evidence names. |
| `acceptance_criteria` | Given/when/then expectations for reviewers. |
| `steps[*].evidence` | Manifest, log, screenshot, window, starter-world note, and readiness-gap evidence. |
| `agentic_follow_on.deterministic_gate` | How instructor, student, or agent reviews should use the preflight evidence. |
| `unsupported_policy` | Loud-failure behavior and unsupported claims. |

Use portable, public wording. Avoid internal shorthand and repository-local
planning vocabulary. Do not describe this scenario as completing a lesson,
clicking through all Alice UI actions, assessing creativity, grading a learner
world, or covering all Alice behavior.

Good wording:

```yaml
purpose: >-
  Record bounded evidence that the real Alice harness opens the bundled starter
  project, names one later starter-world edit, attempts a run or observation,
  and records manifest, log, screenshot or window evidence, missing run/observe
  states, and readiness-gap notes before save, reopen, export, sharing, or
  classroom-readiness work is trusted.
```

Good limitation wording:

```yaml
unsupported_policy: >-
  If host graphics, DISPLAY, Java 21, Maven prerequisites, or the explicit
  EATME_REAL_ALICE=1 gate are missing, fail loudly. This scenario does not
  provide full UI automation, creative assessment, learner-world grading, or
  complete Alice coverage; it also does not show full world execution, visible
  rendering correctness, first-lesson completion, Save completion, or
  deployed sharing/platform success.
```

## Refresh the generated Gadugi adapter

Gadugi adapters are generated from canonical eatme scenarios. After changing the
canonical YAML, check adapter freshness:

```bash
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
```

If the check reports stale generated output, regenerate adapters:

```bash
cargo run -q -p eatme-cli -- assets generate-gadugi --json
```

Then inspect and commit the canonical scenario change with the regenerated
adapter change:

```text
assets/scenarios/eatme/starter-project-open-save-export-preflight.yaml
assets/scenarios/gadugi/starter-project-open-save-export-preflight.yaml
```

Do not hand-edit the generated Gadugi adapter to change mission intent. Edit the
canonical eatme scenario and regenerate instead.

## Keep implementation surfaces consistent

The planned starter-project preflight contract is ready only when every committed
surface uses the same bounded language:

| Surface | What must match |
| --- | --- |
| Canonical scenario | `assets/scenarios/eatme/starter-project-open-save-export-preflight.yaml` names attempted run or observation, explicit missing Run-window or observe-state evidence, and all unsupported claims. |
| Generated adapter | `assets/scenarios/gadugi/starter-project-open-save-export-preflight.yaml` is regenerated from the canonical scenario and carries the same wording. |
| Rust boundary tests | Readiness and scenario-contract tests assert the same non-claims, including full world execution and deployed sharing/platform success. |
| Documentation | This page and the run/observe readiness pages describe only the same bounded evidence. |

Older Save-only shorthand is not specific enough for the new contract. Replace
it with the explicit unsupported claims: full UI automation, full world
execution, visible rendering correctness, grading, creative assessment, Save
completion, deployed sharing/platform success, and first-lesson completion.
Missing Run-window state and missing observe-state evidence must remain separate
gaps.

## Validate the boundary contract

The current starter-project/preflight boundary check is the focused Rust test in:

```text
crates/eatme-assets/src/starter_project_preflight_boundary_tests.rs
```

Run the boundary check directly:

```bash
cargo test -p eatme-assets starter_project_preflight_boundary
```

The test validates the canonical scenario YAML, generated Gadugi adapter
wording, this page, and
[Default-workflow PR Readiness](default-workflow-pr-readiness.md) against the
same bounded evidence contract.

The documentation check fails only on the narrow readiness overclaim phrases
listed by the source contract. It does not fail on negative boundary statements
such as this page's explanation that starter-project preflight evidence is not
pull request readiness. Failure output names the file, matched phrase, contract
source, and bounded replacement wording.

Validate the edited scenario:

```bash
cargo run -q -p eatme-cli -- assets validate \
  --path assets/scenarios/eatme/starter-project-open-save-export-preflight.yaml \
  --json
```

Validate all committed assets:

```bash
cargo run -q -p eatme-cli -- assets validate --json
```

Check generated adapter consistency:

```bash
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
```

Build the documentation site:

```bash
mkdocs build --strict
```
