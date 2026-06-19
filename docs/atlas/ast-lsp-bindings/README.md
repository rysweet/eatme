Mode: static-approximation

# AST+LSP Bindings

Static export map derived from `crates/*/src/lib.rs`, crate manifests, and `eatme-cli`
call sites. No LSP server was used; this layer is a static approximation of public symbol
boundaries and cross-crate consumers.

## Public API surface

| Crate | Public surface observed in `lib.rs` | Cross-crate binding notes |
| --- | --- | --- |
| `eatme-core` | Re-exports AST, collaboration, command, hash, and manifest types/functions; `pr199_recovery` remains a public module. | `eatme-alice` binds to command/manifest/hash helpers; `eatme-assets` binds to AST types. |
| `eatme-alice` | Re-exports comparison/readiness APIs, dependency checks, discovery, packaging, launch smoke orchestration, launch options, and scenarios. | `eatme-cli` imports these at the crate root and dispatches subcommands through them. |
| `eatme-assets` | Re-exports assessment, grading, quality scoring, sharing-platform, gadugi, and validation/report types; also exposes `validate_assets`. | `eatme-cli` calls validation and gadugi APIs directly; `grading.rs` also consumes grading helpers. |

## Consumption paths used for the approximation

- `crates/eatme-cli/src/main.rs` imports `eatme_alice::{...}` for deps, discovery, package,
  launch smoke, comparison, and readiness flows.
- `crates/eatme-cli/src/main.rs` calls `eatme_assets::validate_assets`,
  `validate_persona_crew`, `validate_scenario_asset`, and `generate_gadugi_adapters`.
- `crates/eatme-cli/src/grading.rs` combines `eatme_assets::validate_assets` and
  `eatme_assets::grade_first_lesson_readiness` with `eatme_alice::check_dependencies`.

## Mermaid

```mermaid
graph LR
    cli["eatme-cli consumers"]

    core_ast["eatme-core ast exports"]
    core_cmd["eatme-core command exports"]
    core_manifest["eatme-core manifest exports"]
    core_misc["eatme-core collaboration/hash/pr199"]

    alice_compare["eatme-alice compare/readiness"]
    alice_ops["eatme-alice deps/discover/package"]
    alice_launch["eatme-alice launch surface"]

    assets_assess["eatme-assets assessment/sharing"]
    assets_grade["eatme-assets grading surface"]
    assets_validate["eatme-assets validation/report/gadugi"]

    alice_compare --> core_manifest
    alice_ops --> core_cmd
    alice_launch --> core_cmd
    alice_launch --> core_manifest
    alice_launch --> core_misc
    assets_grade --> core_ast
    cli --> alice_compare
    cli --> alice_ops
    cli --> alice_launch
    cli --> assets_assess
    cli --> assets_grade
    cli --> assets_validate
```

## DOT

```dot
digraph ast_lsp_bindings {
    graph [rankdir=LR, fontsize=10, labelloc=t, label="eatme public API surface (static approximation)"];
    node [shape=record, fontsize=10];

    cli [label="{eatme-cli|crate-root alice imports + direct eatme_assets calls}"];

    subgraph cluster_core {
        label="eatme-core exports";
        color=lightgrey;
        core_ast [label="{ast|Procedure|Program|Statement}"];
        core_cmd [label="{command|CommandOutput|CommandRunner|CommandSpec|RealCommandRunner}"];
        core_manifest [label="{manifest|ArtifactInfo|AssertionResult|LaunchSmokeManifest}"];
        core_misc [label="{other public surface|collaboration|fs_hash|pr199_recovery}"];
    }

    subgraph cluster_alice {
        label="eatme-alice exports";
        color=lightgrey;
        alice_compare [label="{compare|comparison + readiness APIs}"];
        alice_ops [label="{deps/discover/package|DependencyReport|AliceDiscovery|PackageOptions}"];
        alice_launch [label="{launch surface|run_launch_smoke|LaunchSmokeOptions|LaunchSmokeScenario|write_preflight_blocked_manifest}"];
    }

    subgraph cluster_assets {
        label="eatme-assets exports";
        color=lightgrey;
        assets_assess [label="{assessment + sharing|creative_assessment|sharing_platform}"];
        assets_grade [label="{grading surface|GradingReport + lesson-specific graders}"];
        assets_validate [label="{validation/report/gadugi|validate_assets|validate_persona_crew|validate_scenario_asset|generate_gadugi_adapters}"];
    }

    alice_compare -> core_manifest;
    alice_ops -> core_cmd;
    alice_launch -> core_cmd;
    alice_launch -> core_manifest;
    alice_launch -> core_misc;
    assets_grade -> core_ast;
    cli -> alice_compare;
    cli -> alice_ops;
    cli -> alice_launch;
    cli -> assets_assess;
    cli -> assets_grade;
    cli -> assets_validate;
}
```

## Source files

- [ast-lsp-bindings.mmd](ast-lsp-bindings.mmd)
- [ast-lsp-bindings.dot](ast-lsp-bindings.dot)
