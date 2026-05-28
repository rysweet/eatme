# Data Flow

Behavioral layer 6 maps the main artifact transformations in `eatme`: scenario assets, launch-smoke execution, `.a3p` grading, and web-platform request loops.

## Flow inventory

| Flow | Start artifact | Main transforms | End artifact |
| --- | --- | --- | --- |
| Scenario runtime | Scenario YAML | YAML parse -> validation -> launch contract | Alice desktop or TS server run |
| Instructor grading | Student `.a3p` ZIP | ZIP read -> `program.xml` parse -> AST extraction -> grading | `GradingReport` + `QualityScore` |
| Launch smoke | CLI invocation | dependency check -> package -> launch -> evidence capture | `LaunchSmokeManifest` |
| Web platform | HTTP request | request -> TS server -> JSON response -> assertions | `StepResult` / scenario verdict |

## Mermaid overview

![Data flow Mermaid](data-flow-mermaid.svg)

## DOT overview

![Data flow DOT](data-flow-dot.svg)

## Source files

- [data-flow.mmd](data-flow.mmd)
- [data-flow.dot](data-flow.dot)
