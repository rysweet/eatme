# CLI usage

The eatme command line is exposed through the `eatme-cli` Cargo package:

```bash
cargo run -q -p eatme-cli -- <command>
```

Commands that accept `--json` print JSON when the flag is present. Without
`--json`, readiness commands print a plain bounded report with scenario-focused
`Shown`, `Not yet shown`, and `Unproven` sections. First-lesson readiness also
prints `Desktop proof` and emits proof details in this order: `Shown`, `Not yet
shown`, optional `Desktop next action`, optional `Original Alice action evidence`,
and `Unproven`.

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
| `alice check-lesson-readiness` | Report bounded readiness evidence for supported scenarios, including first-lesson readiness and `real-alice-launch-smoke` launch-smoke readiness |
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
  session steps, selected `ui-action-contract.json` evidence, and the
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
progress evidence. The report adds user-facing
`shown_evidence[]`, `not_yet_shown[]`, optional `desktop_next_action`,
`original_alice_action_evidence`, and `unproven_claims`, while preserving legacy
`evidence_boundaries[]` and `evidence_progress.items[]` states for automation
consumers. It reports original Alice and RabbitHole launch/action diagnostics in
`target_evidence[]` and summarizes original Alice action evidence in
`original_alice_action_evidence`. If
`original_alice_action_evidence.status` is `missing`, that state remains visible
as structured JSON; the `alice run-first-lesson-readiness` plain renderer also
shows it in the `Original Alice action evidence` section. The report then emits
one normalized boundary state per first-lesson scenario claim for Select Project,
procedure/edit, Save option/action evidence, visible rendering, grading,
creative assessment, and first-lesson completion. Missing, malformed, ambiguous,
unsafe, manifest-only, incomplete, out-of-order, or uncertain evidence remains
visible as `Not yet shown`. Boundary metadata may show that a boundary was
declared or observed, but it does not prove Save completion, rendering
correctness, grading, creative assessment, or first-lesson completion unless a
matching explicit completion/correctness evidence item exists.

When required evidence is missing, invalid, incomplete, or insufficient, JSON
includes `evidence_gap_message` and plain output prints the same line:

```text
Evidence gap: Missing evidence means this report cannot confirm first-lesson readiness, lesson completion, grading, or creative assessment.
```

Use that line as a stop sign for confirmation, not as a failed grade or a lesson
result. A pure known-blocker report does not need the line; if the report also
has missing, invalid, incomplete, or insufficient required evidence, it remains
`not_ready` and includes the line.

The `alice check-lesson-readiness --json` report includes `role_readiness` for
`instructor` and `student`, plus the legacy `lesson_session_readiness` student
envelope. The normalized `status` is `ready`, `not_ready`, or `blocked`. A
blocked report can identify a coherent explicit blocker, but `passed` remains
false until the bounded readiness status is `ready`; the blocker is not proof
that full Alice UI automation is complete.

The `alice run-first-lesson-readiness --json` sequence report repeats the same
gap field at the sequence top level and inside
`readiness_report.evidence_gap_message`.
When the missing proof is the next-action artifact, JSON
`evidence_progress.next_missing_real_desktop_proof`, progress details, and plain
CLI blockers use the display-safe phrase `desktop next-action evidence` instead
of exposing the internal artifact path.
Valid RabbitHole desktop next-action evidence that applies to the current run
emits top-level `desktop_next_action` in JSON and a
`Desktop next action` section after `Not yet shown` in plain output. Missing,
invalid, unsafe, stale, or non-applicable desktop next-action evidence omits that
top-level summary and remains represented through `Not yet shown`, `issues`, or
legacy progress and boundary fields. Display-safe wording uses
`desktop next-action evidence` instead of exposing the internal artifact path.

Check bounded launch-smoke readiness for the baseline real Alice smoke:

```bash
cargo run -q -p eatme-cli -- alice check-lesson-readiness \
  --manifest runs/comparisons/real-alice-launch-smoke/local-comparison/comparison-manifest.json \
  --json
```

For `scenario_id="real-alice-launch-smoke"`, the readiness path maps existing
launch-smoke manifest evidence into launch-smoke readiness only. A ready result
means the required launch-smoke manifests, assertions, failure-category state,
and manifest-level artifact metadata were present and coherent. Missing,
partial, malformed, manifest-only, failed, unsafe, or contradictory evidence is
reported as `not_ready`. The output must keep lesson completion, grading, creative
assessment, full UI automation, and visible correctness explicitly unproven.

See [Real Alice Launch-Smoke Readiness](real-alice-launch-smoke-readiness.md)
for the plain output and JSON API contract.

Run the first-lesson comparison and readiness check as one bounded sequence:

```bash
export NODE_OPTIONS=--max-old-space-size=32768

EATME_REAL_ALICE=1 \
ALICE_BASELINE_HOME=/path/to/alice-reference \
ALICE_MODERNIZED_HOME=/path/to/alice-candidate \
cargo run -q -p eatme-cli -- alice run-first-lesson-readiness \
  --run-id local-first-lesson-readiness \
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
First-lesson/grading gap report: not ready
Gap report scope: missing/incomplete evidence, unsupported claims, and next actions only.
Evidence gap: Missing evidence means this report cannot confirm first-lesson readiness, lesson completion, grading, or creative assessment.
Desktop proof: skipped (execute_not_requested) - execution was not requested; rerun with --execute on a machine with Alice desktop access to collect real desktop proof

Shown:
- Nothing yet.

Not yet shown:
- Select Project is not yet shown.
- Procedure/edit is not yet shown.
- Save option/action evidence is not yet shown.
- Visible rendering is not yet shown.
- Grading is not yet shown.
- Creative assessment is not yet shown.
- First-lesson completion is not yet shown.
- Screenshot, log, and window evidence is not yet shown.

Original Alice action evidence:
- Original Alice action evidence is missing.
- Original Alice action evidence was not found in the comparison target evidence.

Unproven:
- Full Alice UI automation is not proven.
- Grading is not proven.
- Creative assessment is not proven.
- Visible rendering correctness is not proven.
- Save completion is not proven.
- First-lesson completion is not proven.
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
  --json \
  --no-memory \
  --offline-package
```

The automation scenario writes launch, log, window, screenshot, action-contract,
and first-lesson readiness progress evidence. Readiness keeps Save Project and
Select Project proof-artifact states in `evidence_progress.items[]`.
The report additionally summarizes Select Project, procedure/edit, Save
option/action evidence, visible rendering, grading, creative assessment, and
first-lesson completion independently as shown or not yet shown while preserving
the legacy `present`, `missing`, `invalid`, `not_observed`, or `blocked` machine
states. The report treats each result as boundary-specific evidence only, not
full UI coverage, rendering correctness, grading, creative assessment, Save
completion, or completed lesson proof.

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

## Save/reopen evidence in first-lesson readiness

When Alice hooks are available, the first-lesson readiness sequence includes
save and reopen probes after run-world proof is accepted. The sequence is:

1. Run-world proof is established through the Alice launch and procedure-edit
   flow.
2. `tools/eatme-save-project` is invoked to save the edited project under
   `project-save/`.
3. `tools/eatme-reopen-project` is invoked to reopen the saved artifact under
   `project-reopen/`.
4. Save and reopen evidence appears in the readiness report alongside the
   existing launch, action-contract, and first-lesson evidence.

The save/reopen probes are gated: save requires run-world proof, reopen requires
accepted save proof. When a hook is missing or a precondition is not met, the
probe reports `blocked` and the readiness report shows the boundary explicitly.

Save proof may appear in `ui-action-contract.json`. Reopen proof appears only in
the `project-reopen/` evidence directory and is never inferred from save proof
alone. For the full hook API, artifact layout, and validation rules, see
[Save/reopen Readiness](save-reopen-readiness.md).

## Cache isolation

When eatme is installed through `uvx` or `pip`, the Python CLI wrapper
(`src/eatme_uvx/cli.py`) uses a SHA-256 hash of the source root to isolate the
Cargo target directory. This prevents target-dir conflicts when multiple
worktrees, clones, or `uvx`-installed copies of the repository build
concurrently.

The isolated target directory is:

```text
${XDG_CACHE_HOME:-~/.cache}/eatme-uvx/targets/<sha256-prefix-16>/
```

The hash uses the first 16 hex characters of `sha256(str(source_root))`. Each
distinct source root gets its own target directory so incremental builds do not
interfere across worktrees.

The `CARGO_TARGET_DIR` environment variable is set automatically when not already
present. To override the default isolation, set `CARGO_TARGET_DIR` explicitly
before invoking the wrapper.
