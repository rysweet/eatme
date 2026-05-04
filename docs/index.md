# eatme documentation

`eatme` is the documentation, asset, and launch-smoke harness for agentic Alice
quality assurance. It describes classroom missions as editable assets, validates
those assets before they are trusted, generates Gadugi adapter scenarios from the
canonical eatme scenarios, and records deterministic evidence when real Alice is
launched.

The finished system has three layers:

| Layer | Purpose |
| --- | --- |
| Canonical mission assets | Persona crews and eatme scenario YAML owned by this repository |
| Harness commands | Rust CLI commands for validation, Gadugi generation, Alice discovery, packaging, and launch smoke |
| Published docs | This MkDocs site, built locally and deployed through GitHub Pages |

## Audience routes

| If you are... | Start here |
| --- | --- |
| Installing eatme | [Installation](installation.md) |
| Running commands | [CLI Usage](cli-usage.md) |
| Writing scenarios | [Scenario Authoring](scenario-authoring.md) |
| Using Gadugi | [Gadugi Adapters](gadugi-adapters.md) |
| Keeping generated assets in sync | [Generated Asset Consistency](generated-asset-consistency.md) |
| Checking a change | [Validation and Quality Gates](validation-quality-gates.md) |
| Running real Alice | [Alice Integration](alice-integration.md) |
| Planning class activity | [Instructor Missions](instructor-missions.md) |
| Completing a learner journey | [Student Missions](student-missions.md) |
| Publishing docs | [GitHub Pages](github-pages.md) |

## What eatme proves

Eatme proves that Alice-facing assets and adapter scenarios are coherent before
they are used by people or agents. For real Alice smoke lanes, it also proves
that the desktop application can be packaged, launched, observed, and reported
through deterministic artifacts.

A passing launch smoke records:

- dependency-check results
- Alice package command and exit status
- virtual display readiness
- Java launch command
- Alice process status
- startup screenshot or window evidence
- Alice log artifact
- fatal log scan
- manifest assertions
- failure category, or `null` when the smoke passed

## What eatme does not pretend to prove

The current real Alice lesson lanes are launch-smoke lanes. They prove
smoke-ready evidence for a scenario-labeled Alice run. They do not claim full
in-lesson UI automation, grade learner creativity, or inspect private Alice
implementation details.

Instructor and student mission docs describe the intended classroom and agentic
contract. Runtime validation stays explicit about which parts are deterministic
today and which parts are evidence expectations for human or agent review.

## Main workflows

Validate assets:

```bash
cargo run -q -p eatme-cli -- assets validate --json
```

Check generated Gadugi adapters:

```bash
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
```

Understand how generated adapter counts stay aligned with the asset inventory:
[Generated Asset Consistency](generated-asset-consistency.md).

Build the docs site:

```bash
python3 -m venv .venv
. .venv/bin/activate
python -m pip install -r requirements-docs.txt
mkdocs build --strict
```

Run a real lesson smoke:

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
