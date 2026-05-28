# Repository Surface

Top-level structural view of the repository root, centered on the Rust workspace,
committed asset trees, and the docs/automation surfaces requested for the atlas.

## Scope snapshot

| Surface | Observed structure | Notes |
| --- | --- | --- |
| Workspace crates | `eatme-core`, `eatme-alice`, `eatme-assets`, `eatme-cli`, `eatme-test-support` | Declared in the root `Cargo.toml` workspace. |
| Assets | `assets/scenarios/eatme/`, `assets/scenarios/gadugi/`, `assets/personas/` | Scenario YAML and persona crew assets are committed in-tree. |
| Documentation | `docs/`, including `docs/atlas/` | Atlas output nests inside the wider MkDocs docs tree. |
| Automation | `scripts/quality-gates.sh`, `.github/workflows/` | Local and CI entry points for validation/publishing. |

## Mermaid

```mermaid
graph TD
    root["eatme root"]
    cargo["Cargo.toml workspace"]
    crates["crates/"]
    core["eatme-core"]
    alice["eatme-alice"]
    assets_crate["eatme-assets"]
    cli["eatme-cli"]
    support["eatme-test-support"]
    assets["assets/"]
    scenarios["assets/scenarios/"]
    eatme_scen["scenarios/eatme/"]
    gadugi_scen["scenarios/gadugi/"]
    personas["assets/personas/"]
    docs["docs/"]
    atlas["docs/atlas/"]
    scripts["scripts/"]
    quality["quality-gates.sh"]
    github[".github/"]
    workflows["workflows/"]

    root --> cargo
    root --> crates
    crates --> core
    crates --> alice
    crates --> assets_crate
    crates --> cli
    crates --> support
    root --> assets
    assets --> scenarios
    scenarios --> eatme_scen
    scenarios --> gadugi_scen
    assets --> personas
    root --> docs
    docs --> atlas
    root --> scripts
    scripts --> quality
    root --> github
    github --> workflows
```

## DOT

```dot
digraph repo_surface {
    graph [rankdir=TB, fontsize=10, labelloc=t, label="eatme repository surface"];
    node [shape=box, style=rounded, fontsize=10];

    root [label="eatme root"];
    cargo [label="Cargo.toml workspace"];

    subgraph cluster_crates {
        label="crates/";
        color=lightgrey;
        core [label="eatme-core"];
        alice [label="eatme-alice"];
        assets_crate [label="eatme-assets"];
        cli [label="eatme-cli"];
        support [label="eatme-test-support"];
    }

    subgraph cluster_assets {
        label="assets/";
        color=lightgrey;
        scenarios [label="assets/scenarios/"];
        eatme_scen [label="scenarios/eatme/"];
        gadugi_scen [label="scenarios/gadugi/"];
        personas [label="assets/personas/"];
    }

    subgraph cluster_docsops {
        label="docs + automation";
        color=lightgrey;
        docs [label="docs/"];
        atlas [label="docs/atlas/"];
        scripts [label="scripts/"];
        quality [label="quality-gates.sh"];
        github [label=".github/"];
        workflows [label="workflows/"];
    }

    root -> cargo;
    root -> core;
    root -> alice;
    root -> assets_crate;
    root -> cli;
    root -> support;
    root -> scenarios;
    scenarios -> eatme_scen;
    scenarios -> gadugi_scen;
    root -> personas;
    root -> docs;
    docs -> atlas;
    root -> scripts;
    scripts -> quality;
    root -> github;
    github -> workflows;
}
```

## Source files

- [repo-surface.mmd](repo-surface.mmd)
- [repo-surface.dot](repo-surface.dot)
