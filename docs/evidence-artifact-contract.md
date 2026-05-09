# Evidence Artifact Contract

This document defines the evidence artifact contract for first-lesson readiness
and silver-thread desktop evidence validation. The contract describes which
artifact fields are accepted, how artifact states are normalized, and which
wording is rejected before readiness reports consume the artifact.

The contract is intentionally narrow. It validates evidence artifact inputs and
feeds failures into readiness reporting. It does not enforce arbitrary PR prose,
review comments, generated scenario content, Alice UI automation, grading,
creative assessment, or first-lesson completion.

## Scope

The validator applies to evidence artifacts that are read by
[First-Lesson Evidence Readiness](first-lesson-evidence-readiness.md) and
[Lesson Session Readiness](lesson-session-readiness.md).

| Surface | Contract |
| --- | --- |
| `desktop-first-lesson-next-action.json` | Validate schema version, top-level status, candidate actions, required next evidence, non-claims, proof-artifact declarations, evidence boundaries, and artifact text fields. |
| `evidence_boundaries[]` | Validate boundary ids, input status values, metadata state, source, detail, claim, non-claims, artifact metadata, and artifact text fields. |
| Save Project and Select Project proof-artifact declarations | Validate declaration shape and normalize each proof artifact to `present`, `missing`, or `blocked`. A `present` proof artifact means readable artifact availability only. |

Readiness output is not re-parsed as a separate text-validation surface. Instead,
the readiness reporter consumes validated artifact fields and uses conservative
report wording. If artifact validation fails, the failure is surfaced through
`not_yet_shown`, `issues`, `evidence_progress`, or the affected
`evidence_boundaries[]` entry.

## Status values

The contract separates accepted artifact input values from readiness output
states.

| Context | Accepted values | Meaning |
| --- | --- | --- |
| Desktop next-action artifact `status` | `present`, `missing`, `blocked`, `invalid` | Top-level artifact state. Unknown or empty values are invalid. |
| Boundary artifact `status` input | `present`, `missing`, `blocked`, `invalid`, `declared`, `observed` | `present`, `missing`, `blocked`, and `invalid` are evidence states. Legacy `declared` and `observed` are metadata-only inputs and must normalize to output `missing` unless distinct evidence is present. |
| Boundary `metadata_state` input | `declared`, `observed`, `missing`, `blocked`, `invalid`, or another display-safe non-empty producer state | Metadata availability only. It never upgrades a boundary claim to `present`. |
| Proof-artifact declaration `status` | `present`, `missing`, `blocked` | Proof artifact availability state. Missing or unreadable artifact paths normalize to `missing`; explicit blockers normalize to `blocked`. |
| Readiness output item state | `present`, `missing`, `invalid`, `not_observed`, `blocked` | Output state used by `shown_evidence[]`, `not_yet_shown[]`, `evidence_progress.items[]`, and boundary reporting. |

`not_observed` is an output state, not a desktop next-action artifact input
status. Use it when a producer ran but the expected observation was not made.

## `desktop-first-lesson-next-action.json`

The artifact path remains:

```text
run-window-evidence/desktop-first-lesson-next-action.json
```

The artifact must be valid JSON, safely rooted under the comparison evidence
directory, and use:

```json
{
  "schema_version": "eatme.alice-desktop-first-lesson-next-action/v1"
}
```

Field contract:

| Field | Type | Required | Contract |
| --- | --- | --- | --- |
| `schema_version` | string | Always | Must be `eatme.alice-desktop-first-lesson-next-action/v1`. |
| `status` | string | Always | Must be `present`, `missing`, `blocked`, or `invalid`. |
| `detail` or `reason` | string | Always | Non-empty, display-safe, and evidence-bound. It must not claim completion, grading, creative assessment, Save completion, or full UI automation. |
| `candidate_actions` | array of strings | Always when `status` is `present` or `blocked` | Non-empty for `present` or `blocked`; every item must be a non-empty action id or action label. |
| `requires_next_evidence` or `requiresNextEvidence` | array of strings | Always when `status` is `blocked`; required for `present` when remaining evidence is named | Non-empty for `blocked`; each item must name concrete evidence to collect next, not a success claim. |
| `does_not_claim` or `doesNotClaim` | array of strings | Always | Non-empty. Must include non-claims for first-lesson completion, grading, creative assessment, Save completion, and full Alice UI automation. |
| `save_project_proof_artifact` | object | Always | Must declare `present`, `missing`, or `blocked`. `present` requires a safe, readable artifact. |
| `select_project_proof_artifact` | object | Always | Must declare `present`, `missing`, or `blocked`. `present` requires a safe, readable artifact. |
| `evidence_boundaries` or `evidenceBoundaries` | array | Always | Non-empty and complete for the first-lesson boundary set. Each entry must satisfy the boundary contract below. |

Minimal bounded example:

```json
{
  "schema_version": "eatme.alice-desktop-first-lesson-next-action/v1",
  "status": "blocked",
  "detail": "Desktop next-action evidence was read as an observation only.",
  "candidate_actions": ["save-project"],
  "requires_next_evidence": [
    "Collect explicit Save finish-state evidence before reporting Save completion."
  ],
  "does_not_claim": [
    "full Alice UI automation",
    "grading",
    "creative assessment",
    "Save completion",
    "first-lesson completion"
  ],
  "save_project_proof_artifact": {
    "status": "missing",
    "reason": "Save completion evidence is not yet proven."
  },
  "select_project_proof_artifact": {
    "status": "blocked",
    "blocker": {
      "reason": "Select Project proof collection is blocked by an explicit desktop affordance boundary.",
      "codes": ["select_project_proof_unavailable"]
    }
  },
  "evidence_boundaries": [
    {
      "id": "save_project",
      "status": "present",
      "source": "automation_scenario",
      "metadata_state": "observed",
      "detail": "Save action evidence is present for this scenario boundary.",
      "claim": "Save action evidence is present for this scenario boundary.",
      "does_not_prove": [
        "desktop Save completion",
        "grading",
        "creative assessment",
        "first-lesson completion"
      ]
    }
  ]
}
```

## `evidence_boundaries[]`

Readiness validates a complete first-lesson boundary set. Each required boundary
must appear once.

| Boundary id | Required meaning | Required non-claim boundary |
| --- | --- | --- |
| `select_project` | Select Project scenario evidence | Does not prove full UI automation, project-selection success beyond the named boundary, or first-lesson completion. |
| `procedure_edit` | Procedure/edit scenario evidence | Does not prove code correctness, learner understanding, grading, or first-lesson completion. |
| `save_project` | Save action or proof-artifact availability | Does not prove desktop Save completion, grading, creative assessment, or first-lesson completion. |
| `visible_rendering` | Visible rendering observation | Does not prove rendering correctness, animation correctness, creative quality, or first-lesson completion. |
| `grading` | Grading boundary evidence | Does not prove creative assessment or first-lesson completion unless distinct evidence exists. |
| `creative_assessment` | Creative assessment boundary evidence | Does not replace instructor judgment or prove first-lesson completion. |
| `first_lesson_completion` | Completion boundary evidence | Does not prove full UI automation or creative quality unless distinct evidence exists. |

Field contract:

| Field | Type | Required | Contract |
| --- | --- | --- | --- |
| `id` | string | Always | Must match one required first-lesson boundary id. |
| `status` | string | Always | Must be `present`, `missing`, `blocked`, `invalid`, `declared`, or `observed`. Legacy `declared` and `observed` normalize to output `missing`. |
| `source` | string | Always | Non-empty display-safe source category such as `automation_scenario`. |
| `metadata_state` or `metadataState` | string | Always | Non-empty metadata availability state. It does not prove the boundary. |
| `detail` | string | Always | Non-empty, display-safe observation or limitation text. |
| `claim` | string | Always | Bounded claim only. For non-`present` statuses, it must state that the boundary claim is not proven. |
| `does_not_prove` or `doesNotProve` | array of strings | Always | Non-empty. Must preserve the non-claims required for that boundary. |
| `artifact` | object | Required when the boundary relies on an artifact path | Path metadata must resolve under the comparison evidence root. Unsafe, unreadable, empty, or escaping paths make the boundary invalid or missing. |

Status-specific requirements:

| Boundary status input | Required text behavior | Output behavior |
| --- | --- | --- |
| `present` | `detail`, `claim`, and `does_not_prove` must describe only the named bounded evidence. | May become output `present` and appear in `shown_evidence[]`. |
| `missing` | `detail` and `claim` must say the boundary evidence is absent or not proven. | Output `missing`; appears in `not_yet_shown[]` or boundary reporting. |
| `blocked` | `detail` must carry the explicit blocker or limitation. | Output `blocked`; appears as not yet shown or not yet proven with the supplied reason. |
| `invalid` | `detail` must identify the invalid boundary class without dumping raw artifact contents. | Output `invalid`; readiness fails closed. |
| `declared` or `observed` | `detail` may describe metadata only, but must not claim boundary evidence is present. | Output `missing` with `metadata_state` preserved as `declared` or `observed`. |

## Text contract

The shared text contract applies only to artifact input fields consumed by this
evidence path, including `detail`, `claim`, `reason`, `summary`,
`requires_next_evidence`, `does_not_claim`, and boundary `does_not_prove` values.
It is not a general prose linter for PR descriptions or human review comments.

Artifact text must be non-empty after trimming and must not contain placeholder
or filler language such as:

```text
dummy evidence
sample scenario
TODO: fill this in
lorem ipsum readiness text
example invented classroom event
```

Artifact text must also reject unsupported affirmative claims unless a distinct
capability-specific boundary provides evidence for that exact claim:

```text
The first lesson is complete.
The project was graded.
Creative assessment passed.
Full UI automation succeeded.
The saved world received a grade.
RabbitHole completed the whole first lesson.
Save completed successfully.
```

Allowed limitation wording is explicit and bounded:

```text
First-lesson completion is not proven.
This does not prove completion.
Grading is not assessed.
Creative assessment is not claimed.
UI automation is not complete.
Save completion requires distinct finish-state evidence.
```

## Implementation components

The contract is implemented under `desktop_evidence` without changing the public
readiness purpose.

| Component | Responsibility |
| --- | --- |
| `desktop_evidence::evidence_text_contract` | Shared artifact text validation for filler rejection, unsupported affirmative claim rejection, and allowed limitation wording. |
| `desktop_evidence::first_lesson_boundaries` | Boundary shape validation, required boundary ids, status normalization, safe artifact metadata, and boundary text checks. |
| `desktop_evidence::first_lesson_next_action` | Desktop next-action artifact validation, proof-artifact declarations, next-evidence semantics, and text checks. |
| `compare::lesson_readiness` | Consume validation failures through existing readiness reporting without adding new proof behavior. |

Validation errors should identify the field or boundary class. They must not dump
full artifact text, raw artifact contents, screenshots, logs, secrets, or
absolute host paths.

## Related documentation

- [First-Lesson Evidence Readiness](first-lesson-evidence-readiness.md)
- [Lesson Session Readiness](lesson-session-readiness.md)
- [Validation and Quality Gates](validation-quality-gates.md)
