# Code Atlas

Structural and behavioral layers for the eatme workspace atlas.

## Structural layers

1. [Repository Surface](repo-surface/README.md) — workspace crates, asset trees, docs, scripts, and GitHub automation.
2. [AST+LSP Bindings](ast-lsp-bindings/README.md) — static public export map and cross-crate bindings.
3. [Compile-time Dependencies](compile-deps/README.md) — Cargo manifest dependencies, including dev-only edges.
4. [Runtime Topology](runtime-topology/README.md) — CLI dispatch flow across `deps`, `assets`, and `alice` commands.

## Behavioral layers

5. [API Contracts](api-contracts/README.md) — CLI commands, public crate entry points, and web-platform REST contracts.
6. [Data Flow](data-flow/README.md) — scenario YAML, launch smoke, A3P grading, and web-platform request lifecycles.
7. [Service Components](service-components/README.md) — per-crate component maps for `eatme-core`, `eatme-alice`, `eatme-assets`, and `eatme-cli`.
8. [User Journeys](user-journeys/README.md) — student, author, instructor, web-platform, and developer end-to-end sequences.

## Existing reference

- [Crate dependencies](crate-dependencies.md)
