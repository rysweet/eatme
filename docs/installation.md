# Installation

Eatme is a Rust workspace with a small Python documentation toolchain. Runtime
commands are driven through Cargo. The documentation site is built with MkDocs.

## Prerequisites

Install:

- Git
- Rust and Cargo
- Python 3 with virtual environment support
- Java 21 for real Alice packaging and launch smoke runs
- Maven for real Alice packaging
- Xvfb, `xdpyinfo`, `wmctrl`, a screenshot tool, and Mesa/OpenGL support for real
  desktop smoke runs

The asset validation and Gadugi generation workflows do not require Alice to be
installed. Real Alice launch smoke workflows do.

## Clone and build

```bash
git clone https://github.com/rysweet/eatme.git
cd eatme
cargo build --workspace
```

Run the CLI help:

```bash
cargo run -q -p eatme-cli -- --help
```

## Configure Alice

Set `ALICE_HOME` to the Alice checkout used for discovery, packaging, and launch
smoke runs:

```bash
export ALICE_HOME="${ALICE_HOME:-../alice3-modernization}"
```

Check host dependencies before attempting a launch:

```bash
cargo run -q -p eatme-cli -- deps check --json
```

The dependency check fails loudly with actionable messages. Do not replace a
failed dependency check with a mocked launch result.

## Install documentation tooling

The documentation toolchain is intentionally minimal:

```bash
python3 -m venv .venv
. .venv/bin/activate
python -m pip install -r requirements-docs.txt
```

Build the site:

```bash
mkdocs build --strict
```

The output is written to `site/`. That generated directory is not the source of
truth; edit Markdown files in `docs/` and rebuild.

## Environment preferences

`NODE_OPTIONS=--max-old-space-size=32768` is preserved for Node-based agent or
wrapper tooling. Eatme itself is Rust-based, and the documentation site uses
Python/MkDocs.

## First successful local check

After installation, run:

```bash
cargo run -q -p eatme-cli -- assets validate --json
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
python3 -m venv .venv
. .venv/bin/activate
python -m pip install -r requirements-docs.txt
mkdocs build --strict
```

Those commands validate the editable assets, confirm generated adapter
freshness, and prove the documentation site can be built.
