# CLI usage

The eatme command line is exposed through the `eatme-cli` Cargo package:

```bash
cargo run -q -p eatme-cli -- <command>
```

All current command results are printed as JSON. The `--json` flags are accepted
for explicit caller intent and compatibility with scripts and adapters.

## Command overview

| Command | Purpose |
| --- | --- |
| `assets validate` | Validate persona and scenario assets |
| `assets generate-gadugi` | Generate or check Gadugi adapter scenarios |
| `deps check` | Check host dependencies for real Alice smoke runs |
| `alice discover` | Inspect an Alice checkout |
| `alice package` | Package Alice through Maven |
| `alice launch-smoke` | Launch Alice and record deterministic evidence |
| `alice compare-launch-smoke` | Write or execute a two-target launch-smoke comparison manifest |
| `alice check-lesson-session` | Check that a comparison manifest carries a usable lesson-session contract |
| `alice check-lesson-readiness` | Check first-lesson comparison artifacts, including `ui-action-contract.json` |
| `alice run-first-lesson-readiness` | Run the first-lesson comparison plus readiness check sequence |

## Validate assets

Validate every committed asset:

```bash
cargo run -q -p eatme-cli -- assets validate --json
```

Validate one scenario:

```bash
cargo run -q -p eatme-cli -- assets validate \
  --path assets/scenarios/eatme/building-a-scene-first-world.yaml \
  --json
```

Validate one persona crew file:

```bash
cargo run -q -p eatme-cli -- assets validate \
  --path assets/personas/alice-user-crew.yaml \
  --json
```

## Generate or check Gadugi adapters

Check whether committed Gadugi adapters match the canonical eatme scenarios:

```bash
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
```

Generate adapters in place:

```bash
cargo run -q -p eatme-cli -- assets generate-gadugi --json
```

Use `--root <path>` when running from outside the repository root:

```bash
cargo run -q -p eatme-cli -- assets generate-gadugi \
  --root /path/to/eatme \
  --check \
  --json
```

`--check` exits with a failure when an expected generated adapter target is
stale or missing. That makes it the right command for CI and pre-PR validation.
It does not delete or report extra Gadugi YAML files, so remove obsolete
generated adapters manually when their canonical source is removed or renamed.

The adapter generator derives validation expectations from the actual scenario
asset inventory. See
[Generated Asset Consistency](generated-asset-consistency.md) for the
`scenario_asset_count` and exit-code contracts.

## Check dependencies

```bash
cargo run -q -p eatme-cli -- deps check --json
```

This command checks the host tools required by real Alice launch smoke runs,
including Java, Maven, virtual display tooling, screenshot support, and graphics
support. Use it before `alice package` and `alice launch-smoke`.

## Discover Alice

```bash
cargo run -q -p eatme-cli -- alice discover \
  --alice-home "${ALICE_HOME}" \
  --json
```

`--alice-home` points to the Alice checkout. It may also be supplied through the
`ALICE_HOME` environment variable.

## Package Alice

```bash
cargo run -q -p eatme-cli -- alice package \
  --alice-home "${ALICE_HOME}" \
  --offline \
  --json
```

Use `--offline` when the local Maven cache already has the dependencies needed
to package Alice.

## Run an Alice launch smoke

Baseline smoke:

```bash
cargo run -q -p eatme-cli -- alice launch-smoke \
  --alice-home "${ALICE_HOME}" \
  --run-id local-real-alice-launch-smoke \
  --runs-dir runs \
  --timeout 900 \
  --json \
  --no-memory
```

Lesson-labeled smoke:

```bash
EATME_REAL_ALICE=1 cargo run -q -p eatme-cli -- alice launch-smoke \
  --alice-home "${ALICE_HOME}" \
  --scenario building-a-scene-first-world \
  --run-id local-building-a-scene-first-world \
  --runs-dir runs \
  --timeout 900 \
  --json \
  --no-memory \
  --offline-package
```

### `alice launch-smoke` options

| Option | Description |
| --- | --- |
| `--alice-home <path>` | Alice checkout. Can also come from `ALICE_HOME`. |
| `--run-id <id>` | Required run identifier. Use stable, descriptive values. |
| `--runs-dir <path>` | Root directory for run artifacts. Defaults to `runs`. |
| `--timeout <seconds>` | Maximum launch wait. Defaults to 120 seconds. |
| `--scenario <id>` | Scenario id to record in the manifest. Defaults to `real-alice-launch-smoke`. |
| `--starter-project <path>` | Starter project to open. Relative paths resolve from `--alice-home`. |
| `--json` | Explicit JSON output flag. |
| `--no-memory` | Disable memory writes for the run. |
| `--offline-package` | Package Alice in offline mode before launching. |

Non-baseline scenarios fail fast unless `EATME_REAL_ALICE=1` is present.

## Compare two Alice targets

The comparison harness reads editable target definitions from
`assets/alice-comparison-targets.yaml`. The first milestone can write a bounded
manifest without invoking Alice:

```bash
cargo run -q -p eatme-cli -- alice compare-launch-smoke \
  --run-id local-comparison \
  --json
```

Use `--execute` only when both target homes are configured:

```bash
ALICE_BASELINE_HOME=/path/to/alice-reference \
ALICE_MODERNIZED_HOME=/path/to/alice-candidate \
cargo run -q -p eatme-cli -- alice compare-launch-smoke \
  --run-id local-comparison \
  --timeout 900 \
  --json \
  --no-memory \
  --offline-package \
  --execute
```

Target definitions can list `required_paths` that must exist under the resolved
Alice home before launch smoke runs. The default modernized target checks the
RabbitHole `tweedle-lang` grammar files so a missing submodule is reported as a
target-preparation problem instead of a Maven package failure.

The output is written under
`runs/comparisons/<scenario-id>/<run-id>/comparison-manifest.json` and includes
the comparison contract, target metadata, a scorecard summary, timing fields,
per-target artifacts when execution is requested, and assertion/status
differences. In manifest-only mode, the scorecard marks functionality and timing
as not measured. With `--execute`, it reports whether both targets produced
matching launch-smoke functionality evidence and compares target durations only
when both targets pass.

Every comparison manifest carries `comparison_contract`, which defines:

- required inputs: target registry, baseline and modernized targets, scenario id,
  run id, resolved homes for execution, and declared target-required paths;
- expected outputs: comparison manifest, target statuses and durations, target
  launch-smoke manifests when execution runs, scorecard, and diff;
- pass/fail semantics: `matched`, `different`, `incomplete`, and
  `not_measured`;
- timing rules: duration scope and the requirement for repeated same-machine
  samples before making speed claims;
- non-claims: the launch-smoke comparison does not automate full Alice lesson
  creation and consumption, perform creative assessment, grade student worlds,
  prove broad Alice compatibility, or prove modernization quality.

Every comparison manifest also carries `lesson_session_contract`, which makes the
selected scenario's instructor/student boundary explicit:

- `real-alice-launch-smoke` is launch-readiness evidence only;
- lesson-labeled launch smoke records the same startup evidence under the chosen
  scenario id;
- `first-lessons-real-ui-actions` records the required instructor/student
  session steps, the current `ui-action-contract.json` evidence, and the
  `action_contract_blocked_until_ui_automation` boundary until deterministic
  Alice desktop actions are implemented.

Check a comparison manifest before treating it as lesson-session evidence:

```bash
cargo run -q -p eatme-cli -- alice check-lesson-session \
  --manifest runs/comparisons/first-lessons-real-ui-actions/local-comparison/comparison-manifest.json \
  --json
```

The check fails when `lesson_session_contract` is missing, when its scenario does
not match the comparison manifest, or when the first-lesson contract omits the
open/change/run/save steps, `ui-action-contract.json` evidence, or required
non-claims.

Check executable first-lesson readiness evidence after running a comparison with
`--execute`:

```bash
cargo run -q -p eatme-cli -- alice check-lesson-readiness \
  --manifest runs/comparisons/first-lessons-real-ui-actions/local-comparison/comparison-manifest.json \
  --json
```

This consumes the embedded target launch manifests and each
`ui-action-contract.json`. It passes only when both comparison targets include
real Alice execution evidence, specific Alice window evidence, the required
place/edit/run/save action assertions, and a readable action contract with the
matching required action ids. A passing readiness report can still have
`readiness_status=blocked_until_ui_automation`; that means the concrete
artifacts are present and the only accepted blocker is the current lack of
deterministic UI actions. It is not full UI automation, not creative assessment,
and not learner-world grading.

Run the first-lesson comparison and readiness check as one bounded sequence:

```bash
EATME_REAL_ALICE=1 \
ALICE_BASELINE_HOME=/path/to/alice-reference \
ALICE_MODERNIZED_HOME=/path/to/alice-candidate \
cargo run -q -p eatme-cli -- alice run-first-lesson-readiness \
  --run-id local-first-lesson-readiness \
  --timeout 900 \
  --json \
  --no-memory \
  --offline-package \
  --execute
```

The command fixes the scenario to `first-lessons-real-ui-actions`, writes
`runs/comparisons/first-lessons-real-ui-actions/<run-id>/comparison-manifest.json`,
then immediately runs the same readiness check against that manifest. Without
`--execute` it still writes a manifest and returns an incomplete report that says
target launch evidence is missing. With `--execute`, non-baseline Alice scenarios
still require `EATME_REAL_ALICE=1`. The command preserves the same boundaries:
it does not create a complete instructor assignment, consume a complete student
lesson, perform creative assessment, grade learner worlds, or claim broad Alice
compatibility.

### Outside-in evidence recipes for Alice lesson scenarios

Use the baseline when the only claim is that the real Alice launcher works:

```bash
cargo run -q -p eatme-cli -- alice launch-smoke \
  --alice-home "${ALICE_HOME}" \
  --run-id local-real-alice-launch-smoke \
  --runs-dir runs \
  --timeout 900 \
  --json \
  --no-memory
```

Use the student action-contract scenario when the claim includes first-lesson
evidence for object placement, code/procedure editing, running the world, and
saving a project:

```bash
export NODE_OPTIONS=--max-old-space-size=32768

EATME_REAL_ALICE=1 cargo run -q -p eatme-cli -- alice launch-smoke \
  --alice-home "${ALICE_HOME}" \
  --scenario first-lessons-real-ui-actions \
  --run-id local-first-lessons-real-ui-actions \
  --runs-dir runs \
  --timeout 900 \
  --json \
  --no-memory \
  --offline-package
```

The action-contract scenario writes manifest/log/window/screenshot evidence and
`ui-action-contract.json`. Without an Alice-side placement hook it reports
`ui_action_automation_unimplemented`. With a valid placement hook it advances to
`ui_action_remaining_steps_unimplemented`, because editing a procedure, running
the world, and saving from the desktop are still not automated. Neither result
is full UI coverage.

Use the instructor remix scenario through asset validation and generated adapters,
not through `alice launch-smoke`, because it is an instructor agentic-flow
scenario:

```bash
cargo run -q -p eatme-cli -- assets validate \
  --path assets/scenarios/eatme/instructor-lesson-materials-remix.yaml \
  --json

cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
```

Instructor remix evidence is a teacher plan, student handout, exit ticket, and
review/remix probe set. It may cite launch evidence, but it does not grade
learner worlds or assess creativity automatically.

## Output contract

Command output is JSON intended for humans, CI, and adapter runners. For smoke
runs, the manifest is the durable artifact. Consumers should use
`failure_category` and `assertions` as the source of truth rather than scraping
terminal text.

For retcon or specification documentation, document only fields and artifacts
that the scenario contract owns. Do not describe launch smoke as full UI
automation, creative assessment, or learner-world grading.
