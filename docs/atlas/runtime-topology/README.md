# Runtime Topology

Runtime view of the CLI dispatcher in `crates/eatme-cli/src/main.rs`, with focused detail
on the `deps`, `assets`, and `alice` command families and their crate-level handoffs.

## Flow notes

| Entry | Runtime handoff | Notes |
| --- | --- | --- |
| `Deps::Check` | `eatme_alice::check_dependencies(&RealCommandRunner)` | CLI instantiates the runner once in `main()`. |
| `Assets::Validate` | `validate_scenario_asset`, `validate_persona_crew`, or `validate_assets` | `is_scenario_asset_path()` routes scenario files to the stricter scenario validator. |
| `Assets::GenerateGadugi` | `generate_gadugi_adapters` | Emits or checks generated adapter YAML. |
| `Assets::GradingReport` | `grading::run_grading_report` | Internally combines `validate_assets` with `check_dependencies`. |
| `Alice::LaunchSmoke` | `ensure_real_alice_gate` -> `LaunchSmokeScenario` -> `run_launch_smoke` | Launch execution is the primary runtime handoff into `eatme-alice`. |
| `Alice::CompareLaunchSmoke` | Optional gate when `--execute`, then `run_launch_smoke_comparison` | Rebuilds comparison inputs before launch comparison. |
| `Alice::RunFirstLessonReadiness` | Optional execute gate -> `run_first_lesson_readiness_sequence` | Shares the same real-Alice gating pattern. |

## Mermaid

```mermaid
graph LR
    main["eatme-cli main()"]
    parse["Cli::parse()"]
    dispatch["Commands match"]
    runner["RealCommandRunner"]

    main --> parse --> dispatch
    main --> runner

    deps_cmd["Deps::Check"]
    deps_api["eatme_alice::check_dependencies"]
    dispatch --> deps_cmd --> deps_api
    deps_api --> runner

    assets_validate["Assets::Validate"]
    asset_route["scenario/persona/root dispatch"]
    assets_scenario["validate_scenario_asset"]
    assets_persona["validate_persona_crew"]
    assets_all["validate_assets"]
    assets_gadugi["Assets::GenerateGadugi"]
    assets_gadugi_api["generate_gadugi_adapters"]
    assets_grading["Assets::GradingReport"]
    assets_grading_api["grading::run_grading_report"]

    dispatch --> assets_validate --> asset_route
    asset_route --> assets_scenario
    asset_route --> assets_persona
    assets_validate --> assets_all
    dispatch --> assets_gadugi --> assets_gadugi_api
    dispatch --> assets_grading --> assets_grading_api
    assets_grading_api --> assets_all
    assets_grading_api --> deps_api

    discover["discover_alice"]
    package["package_alice"]
    gate["ensure_real_alice_gate"]
    launch_prep["LaunchSmokeScenario build"]
    launch["run_launch_smoke"]
    compare["run_launch_smoke_comparison"]
    lesson_contract["check_lesson_session_contract"]
    lesson_readiness["check_lesson_session_readiness"]
    first_lesson["run_first_lesson_readiness_sequence"]

    dispatch --> discover --> runner
    dispatch --> package --> runner
    dispatch --> gate
    gate --> launch_prep --> launch
    gate --> compare
    gate --> first_lesson
    dispatch --> lesson_contract
    dispatch --> lesson_readiness
```

## DOT

```dot
digraph runtime_topology {
    graph [rankdir=LR, fontsize=10, labelloc=t, label="eatme CLI runtime topology"];
    node [shape=box, style=rounded, fontsize=10];

    main [label="eatme-cli main()"];
    parse [label="Cli::parse()"];
    dispatch [label="Commands match"];
    runner [label="RealCommandRunner"];

    main -> parse -> dispatch;
    main -> runner;

    subgraph cluster_deps {
        label="Deps";
        color=lightgrey;
        deps_cmd [label="Deps::Check"];
        deps_api [label="eatme_alice::check_dependencies"];
        deps_cmd -> deps_api;
    }

    subgraph cluster_assets {
        label="Assets";
        color=lightgrey;
        assets_validate [label="Assets::Validate"];
        asset_route [label="scenario/persona/root dispatch"];
        assets_scenario [label="validate_scenario_asset"];
        assets_persona [label="validate_persona_crew"];
        assets_all [label="validate_assets"];
        assets_gadugi [label="Assets::GenerateGadugi"];
        assets_gadugi_api [label="generate_gadugi_adapters"];
        assets_grading [label="Assets::GradingReport"];
        assets_grading_api [label="grading::run_grading_report"];

        assets_validate -> asset_route;
        asset_route -> assets_scenario [label="scenario path/schema"];
        asset_route -> assets_persona [label="persona path"];
        assets_validate -> assets_all [label="repo root"];
        assets_gadugi -> assets_gadugi_api;
        assets_grading -> assets_grading_api;
    }

    subgraph cluster_alice {
        label="Alice";
        color=lightgrey;
        discover [label="discover_alice"];
        package [label="package_alice"];
        gate [label="ensure_real_alice_gate"];
        launch_prep [label="LaunchSmokeScenario build"];
        launch [label="run_launch_smoke"];
        compare [label="run_launch_smoke_comparison"];
        lesson_contract [label="check_lesson_session_contract"];
        lesson_readiness [label="check_lesson_session_readiness"];
        first_lesson [label="run_first_lesson_readiness_sequence"];

        gate -> launch_prep -> launch;
        gate -> compare [style=dashed, label="execute flow"];
        gate -> first_lesson [style=dashed, label="execute flow"];
    }

    dispatch -> deps_cmd;
    deps_api -> runner;

    dispatch -> assets_validate;
    dispatch -> assets_gadugi;
    dispatch -> assets_grading;
    assets_grading_api -> assets_all;
    assets_grading_api -> deps_api;

    dispatch -> discover;
    discover -> runner;
    dispatch -> package;
    package -> runner;
    dispatch -> gate;
    dispatch -> lesson_contract;
    dispatch -> lesson_readiness;
}
```

## Source files

- [runtime-topology.mmd](runtime-topology.mmd)
- [runtime-topology.dot](runtime-topology.dot)
