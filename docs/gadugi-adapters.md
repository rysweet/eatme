# Gadugi adapters

Gadugi adapters are generated scenarios that let a Gadugi runner exercise eatme
without taking ownership of Alice desktop internals.

The canonical source is:

```text
assets/scenarios/eatme/
```

Generated adapter output and hand-authored Gadugi regression scenarios live in:

```text
assets/scenarios/gadugi/
```

## Boundary

| eatme owns | Gadugi adapter owns |
| --- | --- |
| Alice dependency checks | Running eatme CLI commands |
| Alice discovery and packaging | Capturing command stdout and stderr |
| Xvfb/display setup | Inspecting JSON and manifest-level results |
| Java process lifecycle | Reporting adapter pass/fail |
| screenshots, logs, manifests | Presenting declared prompts and checking rubric fields for instructor flows |
| canonical scenario intent | Adapter command shape |

Adapters must not duplicate Xvfb setup, Java launch details, screenshot capture,
log scanning, or process cleanup. They call eatme and evaluate the resulting
JSON and artifacts.

## Generated description contract

Generated adapter descriptions are part of the scenario-link silver thread. The
description is generated from the canonical eatme scenario and must stay
reproducible; do not hand-edit the generated Gadugi YAML to change wording.
For the full scenario-link generated-runner usage, CLI, configuration, and
review contract, see
[Scenario-link Generated Runners](scenario-link-generated-runners.md).

The generator writes descriptions with this shape:

```text
Gadugi-compatible CLI scenario generated from <source-scenario>. Alice desktop launch behavior remains owned by eatme; <bounded evidence scope>.<boundary note>
```

Generated Gadugi YAML does not carry a top-level `schema_version`; use
`metadata.source_eatme_asset`, `metadata.generated_by`, and `metadata.tags` to
trace a runner back to its canonical eatme scenario and generator.

The evidence scope depends on the canonical scenario:

| Scenario | Generated wording boundary |
| --- | --- |
| `first-lessons-real-ui-actions` | Gadugi invokes eatme commands and checks first-lesson readiness evidence. The generated runner keeps honest limits: not full UI automation, not creative assessment, and not learner-world grading. |
| `starter-project-open-save-export-preflight` | Gadugi invokes eatme commands, records bounded starter-world and readiness-gap artifacts, and checks eatme launch-smoke evidence without claiming save/reopen/export coverage. |
| Other launch-smoke scenarios | Gadugi invokes eatme commands and checks manifest-level evidence only. |
| Other scenarios that declare the full honest boundary | The generated adapter preserves the source boundary without adding UI automation, creative assessment, or grading claims. |

For reader-facing first-lesson docs, prefer "generated runner" when describing
`first-lessons-real-ui-actions`. Use "adapter" when naming the file type,
command, or generated asset category.

## Check freshness

Use the check mode in CI and before opening a PR:

```bash
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
```

The command compares one expected generated adapter target per canonical eatme
scenario and fails when any expected target is stale or missing. It does not
prune extra Gadugi YAML files; remove obsolete generated adapters manually when
their canonical source is removed or renamed.
See [Generated Asset Consistency](generated-asset-consistency.md) for the
`scenario_asset_count` source of truth, generator freshness contract, and
validation exit-code behavior.

## Regenerate adapters

After changing canonical scenarios:

```bash
cargo run -q -p eatme-cli -- assets generate-gadugi --json
```

Then inspect and commit both the canonical scenario changes and the generated
adapter changes.

## Running from another directory

Use `--root` when the current working directory is not the repository root:

```bash
cargo run -q -p eatme-cli -- assets generate-gadugi \
  --root /path/to/eatme \
  --check \
  --json
```

## Adapter workflow

For a lesson smoke scenario, the adapter workflow is:

1. Validate assets.
2. Check host dependencies.
3. Run `alice launch-smoke` with the scenario id.
4. Inspect manifest fields such as `scenario_id`, `failure_category`, and
   `assertions`.
5. Report failure when eatme reports failure.

For an instructor agentic flow, the adapter workflow is:

1. Validate assets.
2. Present the canonical agentic prompt.
3. Collect instructor-facing outputs.
4. Check declared acceptance probes and rubric fields.
5. Keep the desktop launch boundary in eatme, not in Gadugi.

For outside-in evidence for instructor and student Alice lesson scenarios, use
the generated adapters as consumers of eatme's explicit contracts:

| Canonical scenario | Adapter expectation |
| --- | --- |
| `real-alice-launch-smoke` | Run the launch smoke and inspect manifest-level launch evidence. |
| `first-lessons-real-ui-actions` | Preserve the action-contract boundary and do not convert `ui_action_automation_unimplemented` into a full UI pass. |
| `instructor-lesson-materials-remix` | Evaluate instructor packet outputs and acceptance probes without launching Alice or grading learner worlds. |

Standard launch-smoke adapters expect command success and a `null`
`failure_category`. The `first-lessons-real-ui-actions` adapter intentionally
does not: it preserves `expect.exit_code: 1` and
`"failure_category": "ui_action_automation_unimplemented"` or
`"ui_action_remaining_steps_unimplemented"` while deterministic object
placement, procedure editing, world running, and project saving are incomplete.
Readiness consumers should inspect the normalized `status`,
`lesson_session_readiness`, and `no_go_contracts` fields documented in
[Lesson Session Readiness](lesson-session-readiness.md).

## Editing policy

Do not hand-edit generated Gadugi adapters to change mission intent. If a prompt,
rubric, artifact path, or expected evidence is wrong, edit the matching canonical
eatme scenario and regenerate.

Hand edits to generated adapters are only appropriate for generator development
itself, and those changes must be followed by a generator run that proves the
committed output is reproducible.

Hand-authored Gadugi regression scenarios may live beside generated adapters when
they test the eatme CLI or validation contract directly. They still count as
scenario assets and must pass `assets validate`.
