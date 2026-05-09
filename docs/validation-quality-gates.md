# Validation and quality gates

Eatme uses explicit gates so assets, generated adapters, Rust code, and docs do
not drift independently.

## Required documentation gate

Build the site:

```bash
python3 -m venv .venv
. .venv/bin/activate
python -m pip install -r requirements-docs.txt
mkdocs build --strict
```

Strict mode treats broken navigation, invalid configuration, and warnings as
failures.

## Required asset gates

Validate persona and scenario assets:

```bash
cargo run -q -p eatme-cli -- assets validate --json
```

Check generated Gadugi adapter freshness:

```bash
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
```

Use these checks for documentation-only changes when the docs mention assets,
scenario ids, or adapter workflow. They prove the documented command examples
still map to committed assets.

The generated adapter count contract is documented in
[Generated Asset Consistency](generated-asset-consistency.md). It defines how
`scenario_asset_count` is discovered, when validation exits non-zero, and why
stale adapters must be regenerated instead of hand-edited.

## Rust quality gates

The local quality script is:

```bash
scripts/quality-gates.sh
```

It runs:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
find crates -name '*.rs' -not -path '*/target/*' -exec wc -l {} +
cargo llvm-cov --workspace --all-features --fail-under-lines 70 --summary-only
```

## Cargo target optimization

The workspace Cargo profiles keep local and agent build artifacts small without
weakening the gates. The `dev` and `test` profiles use line-table debug
information, so normal builds, tests, clippy, and coverage still run while Cargo
avoids writing full debug symbols for every worktree build. When a deep
debugger session needs full symbols, override Cargo for that one command instead
of changing the repository default:

```bash
CARGO_PROFILE_DEV_DEBUG=2 CARGO_PROFILE_TEST_DEBUG=2 cargo test --workspace --all-features
```

### Shared Cargo target cache

Parallel worktrees should reuse one Cargo target cache instead of rebuilding a
full `target/` directory under every checkout. Eatme uses this precedence:

1. `EATME_CARGO_TARGET_DIR`
2. `CARGO_TARGET_DIR`
3. `$XDG_CACHE_HOME/eatme/cargo-target`, or `~/.cache/eatme/cargo-target` when
   `XDG_CACHE_HOME` is not set or is empty
4. The checkout-local `.cargo-target` directory only when neither cache-home
   variable is available

`scripts/quality-gates.sh` exports the selected path as `CARGO_TARGET_DIR`.
Existing `CARGO_TARGET_DIR` users keep standard Cargo behavior, and developers
who do not configure either variable still get a shared cache across worktrees
without hard-coding a host-specific path.

Configure a shared cache with a path that belongs to the current user and is on
a volume with enough free space:

```bash
export EATME_CARGO_TARGET_DIR="$HOME/.cache/eatme/cargo-target"
scripts/quality-gates.sh
```

Agents and local runners may choose a larger mounted volume when one exists:

```bash
export EATME_CARGO_TARGET_DIR="/data/$USER/eatme/cargo-target"
scripts/quality-gates.sh
```

The `/data` path is only an example. Do not commit host-specific target
directories, and do not share a writable target directory between unrelated
users. Build outputs can contain source-derived metadata and local paths, so
they should stay in a private cache and should not be published as CI artifacts.

The `uvx` entry point follows the same convention before using its package cache
target directory:

```bash
EATME_CARGO_TARGET_DIR="$HOME/.cache/eatme/cargo-target" \
  uvx --from git+https://github.com/rysweet/eatme.git@master amplihack --help
```

For `uvx`, the target-dir selection is:

1. `EATME_CARGO_TARGET_DIR`
2. `CARGO_TARGET_DIR`
3. `$XDG_CACHE_HOME/eatme-uvx/target`, or `~/.cache/eatme-uvx/target` when
   `XDG_CACHE_HOME` is not set or is empty

CI remains portable. The GitHub Actions quality-gates workflow uses GitHub's
cache action and direct Cargo commands with the runner-local `target/`
directory; it does not require local-only paths such as `/data`.

The module-size gate enforces the repository convention that Rust source modules
stay at or below 500 lines.
For the split outside-in Alice expansion contract tests, see
[Outside-in Alice Test Modules](outside-in-alice-test-modules.md).

## Real Alice launch gate

Real Alice execution is not implicit. Lesson-labeled launch smokes require:

```bash
export ALICE_HOME="${ALICE_HOME:-../alice3-modernization}"
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

If the host cannot run Alice, the correct result is a visible failure or an
explicitly skipped manual gate, not a passing mocked smoke.

## CI behavior

The quality-gates workflow runs Rust gates on pull requests and pushes to
`master`. The real Alice launch smoke job is manual and requires a self-hosted
runner with Alice desktop dependencies.

The documentation Pages workflow builds MkDocs on pull requests. It deploys only
from `master` pushes or manual dispatch, never from pull requests.

## Change checklist

| Change type | Required checks |
| --- | --- |
| Docs only | `mkdocs build --strict`; asset checks when docs mention asset or adapter behavior |
| Scenario assets | `assets validate`; `assets generate-gadugi --check` or regenerate adapters |
| Gadugi generator | Regenerate adapters; asset validation; Rust tests |
| Alice harness | Rust quality gates; real Alice smoke where environment permits |
| CLI command surface | Rust quality gates; update CLI usage docs; docs build |
| Cargo profile or target-dir behavior | Rust quality gates; focused precedence tests; docs build |
| Lesson-session readiness docs | `mkdocs build --strict`; asset validation and Gadugi freshness checks when scenario ids or adapter behavior are mentioned |
