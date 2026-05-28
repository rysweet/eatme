# Crate Dependencies

The eatme workspace centers on a CLI entrypoint, Alice execution adapters, reusable scenario assets, and shared core contracts.

## Workspace dependency map

```mermaid
flowchart LR
    cli["eatme-cli"] --> alice["eatme-alice"]
    cli --> assets["eatme-assets"]
    cli --> core["eatme-core"]

    alice --> core
    alice --> assets
    assets --> core

    support["eatme-test-support"] -. test helpers .-> alice
    support -. shared fixtures .-> core
```

## Scenario flow through tests

```mermaid
flowchart TD
    scenarios["Scenario inputs<br/>assets/scenarios + personas"] --> validation["eatme-assets<br/>validation + curriculum fixtures"]
    validation --> cliTests["eatme-cli<br/>command and reporting tests"]
    cliTests --> aliceTests["eatme-alice<br/>desktop, compare, and readiness suites"]
    aliceTests --> evidence["Evidence outputs<br/>reports, proofs, and readiness artifacts"]
    evidence --> gates["Regression gates<br/>crate tests + docs scenarios"]
```
