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

`--check` exits with a failure when an adapter would change. That makes it the
right command for CI and pre-PR validation.

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

## Output contract

Command output is JSON intended for humans, CI, and adapter runners. For smoke
runs, the manifest is the durable artifact. Consumers should use
`failure_category` and `assertions` as the source of truth rather than scraping
terminal text.
