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
| Grading report | Rust quality gates; `assets validate`; `deps check`; docs build |
| Lesson-session readiness docs | `mkdocs build --strict`; asset validation and Gadugi freshness checks when scenario ids or adapter behavior are mentioned |
| Save/reopen contract code | `cargo test -p eatme-alice`; asset validation; docs build when evidence boundary wording changes |
| Path validation | `cargo test -p eatme-alice launch_path`; verify symlink and traversal rejection tests pass |
