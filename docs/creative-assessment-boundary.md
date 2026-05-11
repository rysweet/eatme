# Creative assessment boundary

The `creative_assessment` module in `eatme-assets` declares the boundary
between what can be machine-assessed and what needs human review for the
Building a Scene first-lesson. It produces a `CreativeAssessmentReport` that
enumerates assessment aspects in four categories, each classified as either
machine-assessable or human-review-needed.

This module does not perform assessment. It declares **what** can be assessed
by automated tools and **what** requires a human reviewer, instructor, or
peer. The declaration is static — it describes the assessment boundary for
the Building a Scene lesson family, not a per-student result.

## Contents

- [Purpose](#purpose)
- [Usage](#usage)
- [Assessment categories](#assessment-categories)
- [Machine-assessable aspects](#machine-assessable-aspects)
- [Human-review-needed aspects](#human-review-needed-aspects)
- [API reference](#api-reference)
- [Integration with the grading report](#integration-with-the-grading-report)
- [Examples](#examples)
- [Design decisions](#design-decisions)
- [Related documentation](#related-documentation)

## Purpose

The Building a Scene lesson involves both mechanical preconditions (can Alice
launch? are assets valid?) and creative outcomes (did the student build an
interesting scene? did they understand 3D concepts?). The grading report
covers the mechanical preconditions. This module covers the boundary between
mechanical and creative assessment.

Without this boundary declaration, tooling might assume that a passing grading
report means the lesson was completed successfully, or that machine assessment
can evaluate creative work. Neither is true.

The creative assessment boundary makes three things explicit:

1. **Which aspects are machine-assessable** — structural checks that automated
   tools can evaluate deterministically (e.g., "did the student place at least
   one object?").
2. **Which aspects need human review** — creative, pedagogical, or subjective
   assessments that require human judgment (e.g., "does the scene tell a
   story?").
3. **Why** each aspect is classified the way it is — a rationale field on each
   aspect prevents drift between what we claim to assess and what we actually
   can.

## Usage

The creative assessment boundary is accessed through the Rust API. It is not
a CLI command — it is a library type used by other modules and tests.

```rust
use eatme_assets::CreativeAssessmentReport;

let report = CreativeAssessmentReport::for_building_a_scene();

// What can machines check?
for aspect in &report.machine_assessable {
    println!("{}: {} ({})", aspect.name, aspect.rationale, aspect.category);
}

// What needs a human?
for aspect in &report.human_review_needed {
    println!("{}: {} ({})", aspect.name, aspect.rationale, aspect.category);
}
```

The report is `Serialize`-only (not `Deserialize`) to prevent external input
from influencing the boundary declaration.

### JSON output

When serialized, the report produces:

```json
{
  "lesson": "building-a-scene-first-world",
  "machine_assessable": [
    {
      "name": "object-placed",
      "category": "structural",
      "rationale": "Scene graph can be inspected for at least one user-placed object"
    },
    {
      "name": "code-edited",
      "category": "structural",
      "rationale": "Code editor history can confirm at least one edit was made"
    },
    {
      "name": "world-runs",
      "category": "structural",
      "rationale": "Runtime execution can confirm the world ran without fatal errors"
    },
    {
      "name": "object-count",
      "category": "quantitative",
      "rationale": "Scene graph object count is a deterministic integer comparison"
    },
    {
      "name": "code-compiles",
      "category": "structural",
      "rationale": "Compilation success is a binary deterministic check"
    },
    {
      "name": "method-count",
      "category": "quantitative",
      "rationale": "Number of methods defined is a deterministic count"
    }
  ],
  "human_review_needed": [
    {
      "name": "scene-composition",
      "category": "creative",
      "rationale": "Spatial arrangement and aesthetic choices require human judgment"
    },
    {
      "name": "narrative-intent",
      "category": "creative",
      "rationale": "Whether the scene tells a coherent story is subjective"
    },
    {
      "name": "code-quality",
      "category": "pedagogical",
      "rationale": "Code organization and naming reflect understanding, not just correctness"
    },
    {
      "name": "concept-understanding",
      "category": "pedagogical",
      "rationale": "Whether the student grasps 3D coordinate systems requires conversation or explanation"
    },
    {
      "name": "creative-risk-taking",
      "category": "creative",
      "rationale": "Experimentation beyond the tutorial steps shows initiative that machines cannot evaluate"
    },
    {
      "name": "peer-collaboration",
      "category": "pedagogical",
      "rationale": "Quality of peer feedback and collaboration is observable only by humans in the room"
    }
  ]
}
```

## Assessment categories

Each assessment aspect belongs to one of four categories:

| Category | Meaning | Typical assessor |
| --- | --- | --- |
| `structural` | Binary presence/absence checks on project artifacts | Machine |
| `quantitative` | Numeric measurements on project artifacts | Machine |
| `creative` | Aesthetic, narrative, or design-quality judgments | Human |
| `pedagogical` | Understanding, growth, and learning-process evaluations | Human |

The category determines the natural assessor, not a hard rule. A `structural`
aspect *could* be reviewed by a human, and a `pedagogical` aspect *could*
have a machine proxy. The boundary declaration reflects what is reliable and
honest, not what is theoretically possible.

## Machine-assessable aspects

These aspects can be evaluated deterministically from project artifacts
without human judgment:

| Aspect | Category | Rationale |
| --- | --- | --- |
| `object-placed` | structural | Scene graph can be inspected for at least one user-placed object. |
| `code-edited` | structural | Code editor history can confirm at least one edit was made. |
| `world-runs` | structural | Runtime execution can confirm the world ran without fatal errors. |
| `object-count` | quantitative | Scene graph object count is a deterministic integer comparison. |
| `code-compiles` | structural | Compilation success is a binary deterministic check. |
| `method-count` | quantitative | Number of methods defined is a deterministic count. |

Machine-assessable aspects correspond to the lesson interaction steps in the
[grading report](first-lesson-grading-report.md) (`place-object`,
`edit-code`, `run-world`). When those steps can eventually be evaluated at
runtime, these aspects define what the machine evaluation should check.

## Human-review-needed aspects

These aspects require human judgment and cannot be reliably machine-assessed:

| Aspect | Category | Rationale |
| --- | --- | --- |
| `scene-composition` | creative | Spatial arrangement and aesthetic choices require human judgment. |
| `narrative-intent` | creative | Whether the scene tells a coherent story is subjective. |
| `code-quality` | pedagogical | Code organization and naming reflect understanding, not just correctness. |
| `concept-understanding` | pedagogical | Whether the student grasps 3D coordinate systems requires conversation or explanation. |
| `creative-risk-taking` | creative | Experimentation beyond the tutorial steps shows initiative that machines cannot evaluate. |
| `peer-collaboration` | pedagogical | Quality of peer feedback and collaboration is observable only by humans in the room. |

Human-review-needed aspects are not deficiencies in the tooling. They are
fundamental characteristics of creative and pedagogical assessment. The
boundary declaration prevents eatme from overstating what automated tools
can evaluate.

## API reference

### Types

```rust
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct CreativeAssessmentReport {
    pub lesson: String,
    pub machine_assessable: Vec<AssessmentAspect>,
    pub human_review_needed: Vec<AssessmentAspect>,
}

#[derive(Clone, Debug, Serialize)]
pub struct AssessmentAspect {
    pub name: String,
    pub category: AssessmentCategory,
    pub rationale: String,
}

#[derive(Clone, Debug, Serialize)]
pub enum AssessmentCategory {
    #[serde(rename = "structural")]
    Structural,
    #[serde(rename = "quantitative")]
    Quantitative,
    #[serde(rename = "creative")]
    Creative,
    #[serde(rename = "pedagogical")]
    Pedagogical,
}
```

All types derive `Serialize` but not `Deserialize`. The boundary declaration
is constructed by the factory function, not parsed from external input.

### Factory function

```rust
impl CreativeAssessmentReport {
    pub fn for_building_a_scene() -> Self
}
```

Returns the creative assessment boundary report for the Building a Scene
first-lesson. The function is a static declaration — it takes no arguments
and returns the same report every time. It performs no I/O.

### Re-exports

The types are re-exported from the `eatme-assets` crate root:

```rust
pub use creative_assessment::{
    AssessmentAspect, AssessmentCategory, CreativeAssessmentReport,
};
```

### Module location

```text
crates/eatme-assets/src/creative_assessment.rs      — Types and factory (< 300 lines)
crates/eatme-assets/src/creative_assessment_tests.rs — Tests (< 300 lines)
```

Both modules stay well under the repository's 500-line quality gate.

## Integration with the grading report

The creative assessment boundary complements the
[grading report](first-lesson-grading-report.md) but lives in a separate
module. The relationship:

```text
GradingReport (grading_report.rs)
  ├── Precondition steps: validate-assets, check-dependencies, launch-smoke
  │   → Machine-evaluated (ready/blocked)
  ├── Lesson interaction steps: place-object, edit-code, run-world
  │   → Machine-evaluated when executed (not-yet-tested until then)
  └── Creative assessment boundary (creative_assessment.rs)
      ├── machine_assessable: structural/quantitative checks
      │   → What the lesson interaction steps CAN evaluate
      └── human_review_needed: creative/pedagogical aspects
          → What NO step can evaluate — requires human review
```

The grading report tells you **whether** lesson steps can run. The creative
assessment boundary tells you **what** those steps can and cannot assess
once they do run.

Neither module performs creative assessment. Together they make the assessment
boundary explicit so that downstream consumers (instructors, agents, CI) do
not overstate what automated tooling proves.

## Examples

### Listing machine-assessable aspect names

```rust
let report = CreativeAssessmentReport::for_building_a_scene();
let names: Vec<&str> = report
    .machine_assessable
    .iter()
    .map(|a| a.name.as_str())
    .collect();
assert_eq!(names, [
    "object-placed",
    "code-edited",
    "world-runs",
    "object-count",
    "code-compiles",
    "method-count",
]);
```

### Checking category distribution

```rust
let report = CreativeAssessmentReport::for_building_a_scene();
let structural_count = report
    .machine_assessable
    .iter()
    .filter(|a| matches!(a.category, AssessmentCategory::Structural))
    .count();
assert_eq!(structural_count, 4); // object-placed, code-edited, world-runs, code-compiles
```

### Serializing to JSON

```rust
let report = CreativeAssessmentReport::for_building_a_scene();
let json = serde_json::to_string_pretty(&report).unwrap();
println!("{json}");
```

### Using in tests to prevent assessment overreach

```rust
#[test]
fn grading_report_does_not_claim_creative_assessment() {
    let boundary = CreativeAssessmentReport::for_building_a_scene();
    // Every human-review-needed aspect must NOT appear in machine-assessable
    for human_aspect in &boundary.human_review_needed {
        assert!(
            !boundary.machine_assessable.iter().any(|m| m.name == human_aspect.name),
            "Aspect '{}' is listed as both machine-assessable and human-review-needed",
            human_aspect.name
        );
    }
}
```

## Design decisions

### Why a separate module, not part of grading_report.rs?

The grading report is operational — it evaluates preconditions and reports
readiness. The creative assessment boundary is declarative — it describes
what *kinds* of assessment are possible. Mixing them would confuse "can we
run?" with "what can we evaluate?" The separate module also keeps both files
under the 300-line target.

### Why Serialize-only?

The boundary declaration is a hardcoded static report, not user input. Making
it `Deserialize` would create an attack surface where external input could
redefine what eatme claims to assess. `Serialize`-only ensures the boundary
is controlled by code, not configuration.

### Why four categories instead of two?

A binary machine/human split loses information. `structural` and
`quantitative` are both machine-assessable but serve different purposes
(presence checks vs. numeric measurements). `creative` and `pedagogical` are
both human-review-needed but evaluate different dimensions (aesthetic quality
vs. learning outcomes). The four categories help downstream consumers
understand *why* an aspect is classified the way it is.

### Why hardcoded aspects instead of a YAML declaration?

The boundary is a code-level contract, not a configuration file. Hardcoded
aspects are testable, reviewable in PRs, and cannot drift from the crate's
actual capabilities. If the boundary changes, the change appears in a code
diff with test updates — not a silent YAML edit.

## Related documentation

- [First-Lesson Grading Report](first-lesson-grading-report.md) — Readiness
  preflight with step dependencies and status propagation.
- [Scenario Authoring](scenario-authoring.md) — How scenario YAML files
  define lesson steps.
- [Validation and Quality Gates](validation-quality-gates.md) — Repository
  quality gates including the 500-line module limit.
- [Student Missions](student-missions.md) — Learner journey descriptions
  that reference assessment boundaries.
- [Instructor Missions](instructor-missions.md) — Instructor-facing mission
  prompts and rubrics.
- [Lesson Readiness Module Boundary](lesson-readiness-module-boundary.md) —
  Module boundary for lesson readiness comparison code.
