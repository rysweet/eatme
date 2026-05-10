# Lesson readiness module boundary

Lesson readiness comparison keeps
`crates/eatme-alice/src/compare/lesson_readiness.rs` as a thin coordinator. It
loads the comparison manifest, delegates evidence inspection to focused helpers,
and assembles the readiness report. Cohesive readiness logic belongs in
submodules under:

```text
crates/eatme-alice/src/compare/lesson_readiness/
```

This boundary exists so first-lesson evidence-gap reporting can change the
readiness output intentionally without broadening the evidence contract. Missing,
invalid, incomplete, or insufficient first-lesson evidence may produce the fixed
gap notice documented in
[First-Lesson Evidence Readiness](first-lesson-evidence-readiness.md), but it
must not imply lesson completion, grading, creative assessment, or full Alice UI
automation.

Desktop proof handling lives in `lesson_readiness/desktop_proof.rs`. Its helpers
qualify existing desktop evidence without broadening accepted proof roots,
formats, paths, lesson identifiers, or report wording unless the readiness output
is intentionally changed and documented.

## Usage

For fast local preflight while editing readiness code, run:

```bash
TMPDIR=/tmp cargo fmt --check
TMPDIR=/tmp cargo clippy --workspace --all-targets --all-features -- -D warnings
TMPDIR=/tmp cargo test --workspace --all-features
```

Before submitting readiness changes, run the repository quality gate:

```bash
TMPDIR=/tmp ./scripts/quality-gates.sh
```

The quality gate already runs formatting, clippy, workspace tests, Rust module
line-count checks, and coverage. The separate commands above are useful
preflight checks, not additional required gates.

The module-size gate requires Rust source modules under `crates/` to stay at or
below 500 lines. Move helper code into cohesive readiness submodules instead of
letting `lesson_readiness.rs` absorb new behavior.

## Module responsibilities

| Module | Responsibility |
| --- | --- |
| `lesson_readiness.rs` | Coordinates first-lesson readiness comparison, evidence delegation, status calculation, and report assembly. |
| `lesson_readiness/assertions.rs` | Extracts and checks required launch/action assertion evidence from launch manifests. |
| `lesson_readiness/desktop_proof.rs` | Owns desktop proof contract evaluation and first-lesson evidence boundary qualification. |
| `lesson_readiness/no_go.rs` | Extracts explicit no-go contracts from UI action evidence. |
| `lesson_readiness/output.rs` | Owns readiness status normalization, human summaries, role envelopes, required evidence labels, and the fixed evidence-gap sentence. |
| `lesson_readiness/progress.rs` | Builds backward-compatible evidence progress summaries from target evidence and issues. |

## Implementation rule

When adding readiness behavior, keep `lesson_readiness.rs` focused on
orchestration. Move cohesive helper logic into the relevant
`lesson_readiness/` submodule and expose it with the narrowest visibility needed,
preferably `pub(super)`.

Desktop proof qualification must preserve existing behavior: do not broaden
evidence roots, proof formats, accepted paths, lesson identifiers, or report
wording unless the readiness output intentionally changes. If output changes are
intentional, update [First-Lesson Evidence Readiness](first-lesson-evidence-readiness.md)
with the user-facing contract at the same time.
