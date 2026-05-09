# Overclaim boundary detection

The overclaim boundary detection system enforces honest readiness language across
scenario assets, generated Gadugi adapters, and documentation pages. It prevents
starter-project preflight evidence from being described with phrases that imply
broader readiness than the evidence actually supports.

## Contents

- [Usage](#usage)
- [How it works](#how-it-works)
- [Module layout](#module-layout)
- [Overclaim rules contract](#overclaim-rules-contract)
- [API reference](#api-reference)
- [Configuration](#configuration)
- [Examples](#examples)
- [Adding a new overclaim rule](#adding-a-new-overclaim-rule)
- [Authoring workflow](#authoring-workflow)
- [Maintenance checklist](#maintenance-checklist)

## Usage

Run the starter-project preflight boundary tests, which include overclaim
detection:

```bash
cargo test -p eatme-assets starter_project_preflight_boundary
```

Run all eatme-assets crate tests (includes boundary tests alongside other
contract tests):

```bash
cargo test -p eatme-assets
```

The overclaim check is part of the standard Rust quality gate:

```bash
./scripts/quality-gates.sh
```

## How it works

The detection system reads overclaim rules from a Markdown table in the
[Default-workflow PR Readiness](default-workflow-pr-readiness.md) contract
document. Each rule defines a **prohibited phrase** and a **bounded
replacement**. The detector scans document text line by line, normalizing
whitespace and case, and flags any line that contains a prohibited phrase
without a preceding negation boundary (such as "not", "does not", "without").

Whitespace normalization collapses any run of whitespace to a single space and
lowercases before comparison, so `"PR   ready"` matches `"pr ready"`. This
prevents cosmetic formatting differences from bypassing overclaim detection.

This means documentation that explains what the evidence is *not* (negative
boundary statements) passes the check. Documentation that claims the evidence
*is* something overbroad fails.

Example of passing text:

```text
Starter-project preflight evidence is not PR ready.
```

Example of failing text:

```text
This starter-project preflight evidence is PR ready.
```

The failure output names the file path, line number, matched phrase, contract
source, and suggested bounded replacement wording.

## Module layout

The overclaim infrastructure lives in `crates/eatme-assets/src/` and is
test-only (`#[cfg(test)]`):

| File | Responsibility |
| --- | --- |
| `overclaim_test_helpers.rs` | Shared overclaim detection helpers: rule parsing, line scanning, assertion functions, negation-boundary detection, and failure formatting. |
| `starter_project_preflight_boundary_tests.rs` | Boundary contract tests for the starter-project preflight scenario, generated adapter, evidence doc, and readiness report. Uses helpers from `overclaim_test_helpers`. |

Both modules are registered in `crates/eatme-assets/src/lib.rs`:

```rust
#[cfg(test)]
mod overclaim_test_helpers;
#[cfg(test)]
mod starter_project_preflight_boundary_tests;
```

The split keeps each module under the repository 500-line module-size gate.
The helpers module is reusable by any future boundary test module that needs
overclaim detection.

## Overclaim rules contract

The source of truth for overclaim rules is the Markdown table in
[Default-workflow PR Readiness](default-workflow-pr-readiness.md) with the
header:

```text
| Prohibited phrase | Bounded replacement |
```

The current documented rules are:

| Prohibited phrase | Bounded replacement |
| --- | --- |
| `PR ready` | `starter-project preflight evidence recorded` |
| `merge ready` | `starter-project evidence boundary satisfied` |
| `production ready` | `bounded preflight evidence available for review` |
| `ready for merge` | `readiness gaps are documented for later gates` |
| `readiness guaranteed` | `readiness depends on the separate readiness gates` |
| `complete PR readiness` | `starter-project preflight evidence only` |
| `proves visible rendering correctness` | `screenshot or window evidence is observation evidence only` |
| `proves save/reopen/export` | `save, reopen, and export remain readiness gaps` |
| `first lesson is complete` | `starter-project preflight evidence only` |
| `grades learner work` | `records evidence for review; it does not grade` |
| `assesses creativity` | `names an editable change without assessing creativity` |

The boundary tests verify that this table in the contract document matches the
expected rule set exactly. Adding, removing, or changing a rule in the contract
document requires updating the `REQUIRED_DOCUMENTED_OVERCLAIM_RULES` constant in
the boundary test file.

## API reference

The `overclaim_test_helpers` module exposes these test-only helpers. All items
are `pub` within the test module tree and are not part of the public crate API.

### Types

#### `OverclaimRule`

```rust
pub struct OverclaimRule {
    pub phrase: String,
    pub normalized_phrase: String,
    pub bounded_replacement: String,
}
```

A single overclaim rule parsed from the contract document. The
`normalized_phrase` is the whitespace-collapsed, lowercased form used for
matching.

Constructor:

```rust
OverclaimRule::new(phrase: &str, bounded_replacement: &str) -> Self
```

Creates a rule from the raw phrase and replacement strings. The constructor
automatically derives `normalized_phrase` by collapsing whitespace runs to a
single space and lowercasing. Tests typically use this constructor rather than
setting struct fields directly.

#### `ReadinessOverclaim<'a>`

```rust
pub struct ReadinessOverclaim<'a> {
    pub file: &'static str,
    pub line_number: usize,
    pub phrase: &'a str,
    pub bounded_replacement: &'a str,
}
```

A detected overclaim violation, including the file, line number, matched
phrase, and the bounded replacement from the contract.

### Constants

| Constant | Value | Purpose |
| --- | --- | --- |
| `CONTRACT_DOC_PATH` | `docs/default-workflow-pr-readiness.md` | Path to the contract document containing the overclaim rules table. |
| `EVIDENCE_DOC_PATH` | `docs/starter-project-preflight-evidence.md` | Path to the evidence document that is checked against overclaim rules. |

### Functions

#### `read_repo_text(root, repo_relative_path) -> String`

Reads a file from the repository by joining the root path with a
repo-relative path. Panics with a descriptive message if the file is missing.

#### `read_contract_overclaim_rules(root) -> Vec<OverclaimRule>`

Reads the contract document and parses the overclaim rules table. This is the
standard entry point for loading rules in boundary tests.

#### `overclaim_rules_from_contract(text) -> Vec<OverclaimRule>`

Parses overclaim rules from arbitrary Markdown text. Finds the
`| Prohibited phrase | Bounded replacement |` header, reads the table rows
below it, and returns the parsed rules. Panics if the header is missing or no
rules are defined.

The parser stops at the first line that does not start with `|`, so unrelated
Markdown tables elsewhere in the document are ignored.

#### `assert_no_doc_overclaims(file, text, rules)`

Asserts that the given text contains no overclaim violations against the
provided rules. On failure, prints each violation with file path, line number,
matched phrase, contract source, and bounded replacement.

#### `doc_overclaims_in(file, text, rules) -> Vec<ReadinessOverclaim>`

Scans text for overclaim violations and returns them without asserting. Use
this when you need to inspect violations programmatically, such as in the
detector self-tests.

#### `assert_rules_match_contract(rules, expected)`

Asserts that the parsed overclaim rules match an expected set of
`(phrase, replacement)` pairs exactly. Use this to keep the contract document
and boundary tests in sync.

#### `assert_contains_none_with_message(text, needles, message)`

Asserts that none of the given needle strings appear in the text (after
whitespace normalization and lowercasing). The custom message appears on
failure.

#### `format_overclaim_failures(violations) -> String`

Formats a list of overclaim violations into a human-readable multi-line
string. Each line names the file, phrase, line number, contract source, and
bounded replacement wording.

### Negation boundary detection

The detector skips matches where the prohibited phrase is preceded by a
negation boundary. Recognized negation prefixes are:

- `not`
- `does not`
- `do not`
- `without`

This allows documentation to explain what the evidence is not, while still
catching positive overclaims. For example:

```text
"It is not PR ready."            → passes (negation boundary "not")
"It does not prove PR ready."    → passes (negation boundary "does not")
"This evidence is PR ready."     → fails
```

## Configuration

The overclaim detection tests do not require real Alice, Node, or desktop
execution. They read committed assets and documentation files only.

| Setting | Required | Purpose |
| --- | --- | --- |
| `NODE_OPTIONS=--max-old-space-size=32768` | No | Safe to export for workflows that invoke Node-based tooling. Not used by these Rust tests. |
| `EATME_REAL_ALICE=1` | No | Required only for real Alice launch-smoke runs. |
| `ALICE_HOME` | No | Required only when running real Alice launch commands. |
| `TMPDIR=/tmp` | Recommended | Avoids Unix socket path length failures in deep worktrees. |

Use the repository root as the working directory.

## Examples

### Run only the overclaim detector self-tests

```bash
cargo test -p eatme-assets starter_project_preflight_boundary::readiness_overclaim_detector
```

These tests verify the detector itself: negation boundary handling, actionable
failure detail formatting, and rule-table parsing of unrelated Markdown tables.

### Check evidence doc wording in isolation

```bash
cargo test -p eatme-assets starter_project_preflight_boundary::scoped_starter_project_preflight_docs
```

This test reads `docs/starter-project-preflight-evidence.md` and checks it
against the contract overclaim rules. It fails if the evidence page uses
prohibited phrases outside of negative boundary statements.

### Check contract document integrity

```bash
cargo test -p eatme-assets starter_project_preflight_boundary::documented_contract
```

This test verifies that the contract document defines the expected overclaim
rules table, that the table exactly matches the boundary test expectations, and
that the overclaim check is described as current executable behavior (not
planned future work).

### Using overclaim helpers in a new boundary test module

```rust
use crate::overclaim_test_helpers::{
    assert_no_doc_overclaims, read_contract_overclaim_rules, read_repo_text,
};
use std::path::{Path, PathBuf};

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn my_new_evidence_doc_does_not_overclaim() {
    let root = repository_root();
    let text = read_repo_text(&root, "docs/my-new-evidence.md");
    let rules = read_contract_overclaim_rules(&root);
    assert_no_doc_overclaims("docs/my-new-evidence.md", &text, &rules);
}
```

Register the new test module in `crates/eatme-assets/src/lib.rs`:

```rust
#[cfg(test)]
mod my_new_boundary_tests;
```

## Adding a new overclaim rule

1. Add the rule to the Markdown table in
   [Default-workflow PR Readiness](default-workflow-pr-readiness.md) under the
   `| Prohibited phrase | Bounded replacement |` header:

   ```text
   | `my overbroad phrase` | `bounded replacement wording` |
   ```

2. Add the matching tuple to `REQUIRED_DOCUMENTED_OVERCLAIM_RULES` in
   `starter_project_preflight_boundary_tests.rs`:

   ```rust
   ("my overbroad phrase", "bounded replacement wording"),
   ```

3. Run the boundary tests to verify the rule is parsed and enforced:

   ```bash
   cargo test -p eatme-assets starter_project_preflight_boundary
   ```

4. Fix any new violations in scenario assets, generated adapters, or evidence
   documentation by replacing the prohibited phrase with the bounded replacement
   or restructuring the sentence as a negative boundary statement.

## Authoring workflow

Use this workflow when changing overclaim detection behavior.

1. **Identify the change type:**

   | Change | Where to edit |
   | --- | --- |
   | Add or change an overclaim rule | Contract doc table + `REQUIRED_DOCUMENTED_OVERCLAIM_RULES` constant |
   | Fix a false positive (negation not detected) | `is_negated_boundary()` in `overclaim_test_helpers.rs` |
   | Add a new document to overclaim checking | New `#[test]` function using `assert_no_doc_overclaims` |
   | Reuse helpers in another test module | Import from `crate::overclaim_test_helpers` |

2. **Keep contract and tests in sync.** The `documented_contract` test asserts
   exact rule-set equality between the Markdown table and the Rust constant.
   Changing one without the other is a test failure.

3. **Keep public claims honest.** Overclaim rules exist because
   starter-project preflight evidence is intentionally narrow. Do not weaken
   rules to make overbroad documentation pass. Instead, fix the documentation.

4. **Preserve negative boundary wording.** Documentation pages are allowed to
   explain what the evidence is *not*. Do not remove negative boundary
   statements to satisfy a poorly-written overclaim rule.

5. **Run the focused tests and the full quality gate:**

   ```bash
   cargo test -p eatme-assets starter_project_preflight_boundary
   ./scripts/quality-gates.sh
   ```

## Maintenance checklist

Before merging a change that touches overclaim detection:

| Check | Command |
| --- | --- |
| Format Rust files | `cargo fmt --check` |
| Run boundary tests | `cargo test -p eatme-assets starter_project_preflight_boundary` |
| Run all asset crate tests | `cargo test -p eatme-assets` |
| Validate assets | `cargo run -q -p eatme-cli -- assets validate --json` |
| Check generated adapters | `cargo run -q -p eatme-cli -- assets generate-gadugi --check --json` |
| Enforce module size | `find crates -name '*.rs' -not -path '*/target/*' -exec wc -l {} + \| awk '$2 != "total" && $1 > 500 { print; bad=1 } END { exit bad }'` |
| Full quality gate | `./scripts/quality-gates.sh` |
| Build docs | `mkdocs build --strict` |

When changing these files together, commit them as a unit:

```text
docs/default-workflow-pr-readiness.md
docs/starter-project-preflight-evidence.md
docs/overclaim-boundary-detection.md
crates/eatme-assets/src/overclaim_test_helpers.rs
crates/eatme-assets/src/starter_project_preflight_boundary_tests.rs
```
