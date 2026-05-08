# CLI usage

The eatme command line is exposed through the `eatme-cli` Cargo package:

```bash
cargo run -q -p eatme-cli -- <command>
```

Commands that accept `--json` print JSON when the flag is present. Without
`--json`, `alice run-first-lesson-readiness` prints a plain readiness report for
humans, including plain blockers for first-lesson automation scenario evidence.

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

This consumes the embedded target launch manifests and first-lesson readiness
progress evidence. JSON reports explicit `evidence_boundaries[]` and
`evidence_progress.items[]` states, including Save Project and Select Project
proof-artifact entries. It reports original Alice and RabbitHole launch/action
diagnostics in `target_evidence[]`, then reports one normalized boundary state
per first-lesson scenario claim for Select Project, procedure/edit, Save, visible
rendering, grading, creative assessment, and first-lesson completion. Missing,
malformed, ambiguous, unsafe, manifest-only, incomplete, out-of-order, or
uncertain evidence remains visible as a blocker. Boundary metadata may show that
a boundary was declared or observed, but it does not prove bounded Save
completion, rendering correctness, grading, creative assessment, or first-lesson
completion unless the matching boundary evidence exists.

The report includes `role_readiness` for `instructor` and `student`, plus the
legacy `lesson_session_readiness` student envelope. The normalized `status` is
`ready`, `not_ready`, or `blocked`. A blocked report can still be structurally
valid; that means the report found coherent evidence plus an explicit blocker,
not that full Alice UI automation is complete.
When the missing proof is the next-action artifact, JSON
`evidence_progress.next_missing_real_desktop_proof`, progress details, and plain
CLI blockers use the display-safe phrase `desktop next-action evidence` instead
of exposing the internal artifact path.

Run the first-lesson comparison and readiness check as one bounded sequence:

```bash
export NODE_OPTIONS=--max-old-space-size=32768

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
`--execute` it still writes a manifest and returns `status=not_ready` with a
detail `readiness_status=incomplete` because target launch evidence is missing.
Its `desktop_proof_contract` reports `status="skipped"` and
`reason_code="execute_not_requested"` so scripts can distinguish a deliberate
manual smoke skip from a failed desktop proof run. Boundary reporting keeps
plain-output blockers scenario-focused:

```text
First-lesson automation scenario readiness: not ready

Evidence present:
- Alice launch scenario evidence is present.
- Visible rendering scenario evidence is present.

Blockers:
- Select Project scenario evidence is missing.
- Procedure/edit scenario evidence is missing.
- Save scenario evidence is missing.
- Grading scenario evidence is missing.
- Creative assessment scenario evidence is missing.
- First-lesson completion scenario evidence is missing.
```

With `--execute`, non-baseline Alice scenarios still require
`EATME_REAL_ALICE=1`. The command preserves the same conservative scope: it does
not create a complete instructor assignment, consume a complete student lesson,
perform creative assessment, grade learner worlds, or claim broad Alice
compatibility.

For the conservative boundary schema, see
[First-Lesson Evidence Readiness](first-lesson-evidence-readiness.md). For
instructor/student usage recipes, see
[Lesson Session Readiness](lesson-session-readiness.md).

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

Use the student automation scenario when the claim includes first-lesson
scenario evidence for object placement, code/procedure editing, running the
world, and saving a project:

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

The automation scenario writes launch, log, window, screenshot, action-contract,
and first-lesson readiness progress evidence. Readiness reports Save Project
and Select Project proof-artifact states from desktop next-action evidence
through `evidence_progress.items[]`, and
reports Select Project, procedure/edit, Save, visible rendering, grading,
creative assessment, and first-lesson completion independently as `present`,
`missing`, `invalid`, `not_observed`, or `blocked`. The report treats each result as
boundary-specific evidence only, not full UI coverage, rendering correctness,
grading, creative assessment, or completed lesson proof.

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
