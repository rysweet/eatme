# Compile-time Dependencies

Direct dependency map derived from the root workspace manifest plus per-crate
`Cargo.toml` files for `eatme-core`, `eatme-alice`, `eatme-assets`, and `eatme-cli`.

## Manifest inventory

| Crate | Internal workspace deps | External deps | Dev-only deps |
| --- | --- | --- | --- |
| `eatme-core` | _none_ | `anyhow`, `serde`, `serde_json`, `sha2` | _none_ |
| `eatme-alice` | `eatme-core` | `anyhow`, `serde`, `serde_json`, `serde_yaml` | `eatme-assets`, `eatme-test-support`, `regex`, `roxmltree`, `ureq` (`json`), `zip` |
| `eatme-assets` | `eatme-core` | `anyhow`, `serde`, `serde_json`, `serde_yaml` | _none_ |
| `eatme-cli` | `eatme-alice`, `eatme-assets`, `eatme-core` | `anyhow`, `clap` (`derive`, `env`), `serde`, `serde_json` | _none_ |
| `eatme-test-support` | `eatme-core` | `anyhow` | _none_ |

Workspace-managed versions come from the root `Cargo.toml`: `anyhow 1.0`, `clap 4.5`,
`serde 1.0`, `serde_json 1.0`, `serde_yaml 0.9`, and `sha2 0.10`.

## Mermaid

```mermaid
graph TD
    core["eatme-core"]
    alice["eatme-alice"]
    assets_crate["eatme-assets"]
    cli["eatme-cli"]
    support["eatme-test-support"]

    anyhow["anyhow 1.0"]
    serde["serde 1.0"]
    serde_json["serde_json 1.0"]
    serde_yaml["serde_yaml 0.9"]
    sha2["sha2 0.10"]
    clap["clap 4.5"]
    regex["regex 1 (dev)"]
    roxmltree["roxmltree 0.20 (dev)"]
    ureq["ureq 2 + json (dev)"]
    zip["zip 2 (dev)"]

    core --> anyhow
    core --> serde
    core --> serde_json
    core --> sha2

    support --> anyhow
    support --> core

    alice --> anyhow
    alice --> core
    alice --> serde
    alice --> serde_json
    alice --> serde_yaml
    alice -. dev .-> assets_crate
    alice -. dev .-> support
    alice -. dev .-> regex
    alice -. dev .-> roxmltree
    alice -. dev .-> ureq
    alice -. dev .-> zip

    assets_crate --> anyhow
    assets_crate --> core
    assets_crate --> serde
    assets_crate --> serde_json
    assets_crate --> serde_yaml

    cli --> anyhow
    cli --> clap
    cli --> alice
    cli --> assets_crate
    cli --> core
    cli --> serde
    cli --> serde_json
```

## DOT

```dot
digraph compile_deps {
    graph [rankdir=LR, fontsize=10, labelloc=t, label="eatme compile-time dependencies"];
    node [shape=box, style=rounded, fontsize=10];

    subgraph cluster_workspace {
        label="workspace crates";
        color=lightgrey;
        core [label="eatme-core"];
        alice [label="eatme-alice"];
        assets_crate [label="eatme-assets"];
        cli [label="eatme-cli"];
        support [label="eatme-test-support"];
    }

    subgraph cluster_external {
        label="external crates";
        color=lightgrey;
        anyhow [label="anyhow 1.0"];
        serde [label="serde 1.0"];
        serde_json [label="serde_json 1.0"];
        serde_yaml [label="serde_yaml 0.9"];
        sha2 [label="sha2 0.10"];
        clap [label="clap 4.5"];
        regex [label="regex 1 (dev)"];
        roxmltree [label="roxmltree 0.20 (dev)"];
        ureq [label="ureq 2 + json (dev)"];
        zip [label="zip 2 (dev)"];
    }

    core -> anyhow;
    core -> serde;
    core -> serde_json;
    core -> sha2;

    support -> anyhow;
    support -> core [label="path"];

    alice -> anyhow;
    alice -> core [label="path"];
    alice -> serde;
    alice -> serde_json;
    alice -> serde_yaml;
    alice -> assets_crate [style=dashed, label="dev path"];
    alice -> support [style=dashed, label="dev path"];
    alice -> regex [style=dashed, label="dev"];
    alice -> roxmltree [style=dashed, label="dev"];
    alice -> ureq [style=dashed, label="dev"];
    alice -> zip [style=dashed, label="dev"];

    assets_crate -> anyhow;
    assets_crate -> core [label="path"];
    assets_crate -> serde;
    assets_crate -> serde_json;
    assets_crate -> serde_yaml;

    cli -> anyhow;
    cli -> clap;
    cli -> alice [label="path"];
    cli -> assets_crate [label="path"];
    cli -> core [label="path"];
    cli -> serde;
    cli -> serde_json;
}
```

## Source files

- [compile-deps.mmd](compile-deps.mmd)
- [compile-deps.dot](compile-deps.dot)
