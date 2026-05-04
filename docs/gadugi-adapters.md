# Gadugi adapters

Gadugi adapters are generated scenarios that let a Gadugi runner exercise eatme
without taking ownership of Alice desktop internals.

The canonical source is:

```text
assets/scenarios/eatme/
```

The generated adapter output is:

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
| screenshots, logs, manifests | Agentic prompt and rubric execution for instructor flows |
| canonical scenario intent | Adapter command shape |

Adapters must not duplicate Xvfb setup, Java launch details, screenshot capture,
log scanning, or process cleanup. They call eatme and evaluate the resulting
JSON and artifacts.

## Check freshness

Use the check mode in CI and before opening a PR:

```bash
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
```

The command fails when generated files would differ from committed files.
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

For a lesson smoke lane, the adapter workflow is:

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
4. Evaluate acceptance probes and rubric fields.
5. Keep the desktop launch boundary in eatme, not in Gadugi.

## Editing policy

Do not hand-edit generated Gadugi adapters to change mission intent. If a prompt,
rubric, artifact path, or expected evidence is wrong, edit the matching canonical
eatme scenario and regenerate.

Hand edits are only appropriate for generator development itself, and those
changes must be followed by a generator run that proves the committed output is
reproducible.
