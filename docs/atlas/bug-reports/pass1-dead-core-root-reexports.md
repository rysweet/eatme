# PASS 1: dead `eatme-core` root re-exports

- **Checklist:** dead exports (`ast-lsp-bindings`)
- **Verdict:** FAIL

## Finding
`eatme-core` re-exports AST and collaboration symbols at the crate root, but the current downstream consumers do not import those names from `eatme_core`.

## Evidence
- `crates/eatme-core/src/lib.rs:8-12` re-exports `Program`, `Procedure`, `Statement`, `CodeComment`, `CollaborativeProject`, `EditSession`, and `NavigationTarget`.
- `docs/atlas/ast-lsp-bindings/README.md:13-24` records `eatme-alice` consuming command/manifest/hash helpers and `eatme-assets` consuming AST types, but it does not show any root-level AST or collaboration consumers.
- PASS 1 search results:
  - `rg -n 'eatme_core::(Program|Procedure|Statement)|use eatme_core::\{[^}]*\b(Program|Procedure|Statement)\b' crates/eatme-alice crates/eatme-cli -g '*.rs'` -> no matches
  - `rg -n 'eatme_core::(CodeComment|CollaborativeProject|EditSession|NavigationTarget)|use eatme_core::\{[^}]*\b(CodeComment|CollaborativeProject|EditSession|NavigationTarget)\b' crates/eatme-alice crates/eatme-cli -g '*.rs'` -> no matches

## Why this is a bug
Layer 2 is supposed to expose dead public surface. Right now the atlas inventories these root exports but does not call out that part of the root API is effectively unused by the current first-party consumers.

## Impact
The documented public surface is larger than the real contract. That makes future refactors riskier because unused root exports look supported when they may already be dead.

## Suggested fix
Either remove the unused root re-exports, or mark them as intentionally public and add a consumer-level justification in `docs/atlas/ast-lsp-bindings/README.md`.
