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
| `desktop-first-lesson-next-action.json` | Validate schema version, non-empty top-level status, optional candidate actions, optional required next evidence, optional non-claims, optional proof-artifact declarations, optional evidence boundaries, and supplied artifact text fields. |
| `evidence_boundaries[]` | When provided, validate included boundary ids, input status values, metadata state, source, detail, claim, non-claims, artifact metadata, and artifact text fields. Missing boundary entries normalize to missing output entries. |
| Save Project and Select Project proof-artifact declarations | When provided, validate declaration shape and normalize each proof artifact to `present`, `missing`, or `blocked`. Missing declarations normalize to `missing`. A `present` proof artifact means readable artifact availability only. |

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
| Desktop next-action artifact `status` | Any non-empty string; canonical values are `present`, `missing`, `blocked`, and `invalid` | Top-level producer state. Missing or empty values are invalid. Unknown non-empty values are preserved as producer status and do not create special proof behavior. |
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

Only `schema_version` and a non-empty `status` are required for the
next-action artifact itself. Other fields are validated when present and
otherwise normalize to conservative defaults, missing proof-artifact state, or
missing boundary output.

Field contract:

| Field | Type | Required | Contract |
| --- | --- | --- | --- |
| `schema_version` | string | Always | Must be `eatme.alice-desktop-first-lesson-next-action/v1`. |
| `status` | string | Always | Must be non-empty. `present`, `missing`, `blocked`, and `invalid` are canonical values; other non-empty producer statuses are preserved without adding proof behavior. |
| `detail` or `reason` | string | Optional | When supplied, must be non-empty, display-safe, and evidence-bound. It must not claim completion, grading, creative assessment, Save completion, full UI automation, full world execution, deployed sharing, or platform success. If omitted, readiness uses a conservative default detail. |
| `candidate_actions` | array of strings | Optional | When supplied, must be a non-empty array of non-empty action ids or action labels. If omitted, the output list is empty. |
| `requires_next_evidence` or `requiresNextEvidence` | array of strings | Optional | When supplied, must be a non-empty array. Each item must name concrete evidence to collect next, not a success claim. If omitted, the output list is empty. |
| `does_not_claim` or `doesNotClaim` | array of strings | Optional | When supplied, must be a non-empty array of display-safe limitation text. Readiness output still preserves canonical non-claims even when this input field is absent. |
| `save_project_proof_artifact` | object | Optional | When omitted, normalizes to `missing`. `blocked` declarations or blockers normalize to `blocked`; `missing` declarations normalize to `missing`; a readable safe artifact normalizes to `present`. |
| `select_project_proof_artifact` | object | Optional | When omitted, normalizes to `missing`. `blocked` declarations or blockers normalize to `blocked`; `missing` declarations normalize to `missing`; a readable safe artifact normalizes to `present`. |
| `evidence_boundaries` or `evidenceBoundaries` | array | Optional | When supplied, must be a non-empty array. Included entries must satisfy the boundary contract below. Missing required boundary ids normalize to missing output entries. |

Bounded artifact excerpt:

This excerpt is not a copy-paste-valid full fixture. It shows the accepted shape
for one boundary; omitted proof declarations and omitted boundary entries
normalize to conservative missing output states.

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

Readiness emits a complete first-lesson boundary set. Artifact input may provide
a complete set or a partial set. Each boundary entry that appears must identify
one required boundary id and a non-empty status; each required id that is absent
from artifact input normalizes to a missing output boundary.

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
| `source` | string | Required for `present`; optional otherwise | Non-empty display-safe source category such as `automation_scenario`. Omitted non-`present` entries use the default source. |
| `metadata_state` or `metadataState` | string | Required for `present`; optional otherwise | Non-empty metadata availability state. It does not prove the boundary. Omitted non-`present` entries use a conservative normalized state. |
| `detail` | string | Required for `present`; optional otherwise | Non-empty, display-safe observation or limitation text. Optional non-`present` details are validated when supplied; otherwise default missing/blocker text is used. |
| `claim` | string | Required for `present`; optional otherwise | Bounded claim only. Optional non-`present` claims are validated when supplied; output uses a conservative not-proven claim for non-`present` statuses. |
| `does_not_prove` or `doesNotProve` | array of strings | Required for `present`; optional otherwise | Non-empty when required. Optional values are validated when supplied and merged with canonical non-claims for that boundary. |
| `artifact` | object | Optional | Path metadata must resolve under the comparison evidence root when supplied. Unsafe, unreadable, empty, or escaping paths make the boundary invalid or missing. |

Status-specific requirements:

| Boundary status input | Required text behavior | Output behavior |
| --- | --- | --- |
| `present` | `detail`, `claim`, and `does_not_prove` must describe only the named bounded evidence. | May become output `present` and appear in `shown_evidence[]`. |
| `missing` | Supplied `detail` and `claim` must avoid unsupported affirmative claims; omission uses default missing/not-proven wording. | Output `missing`; appears in `not_yet_shown[]` or boundary reporting. |
| `blocked` | Supplied `detail` must carry only an explicit blocker or limitation; omission uses default blocker-safe wording. | Output `blocked`; appears as not yet shown or not yet proven with the supplied or default reason. |
| `invalid` | Supplied `detail` must identify only the invalid boundary class without dumping raw artifact contents; omission uses default invalid wording. | Output `invalid`; readiness fails closed. |
| `declared` or `observed` | Supplied `detail` may describe metadata only, but must not claim boundary evidence is present; omission uses default metadata-only wording. | Output `missing` with `metadata_state` preserved as `declared` or `observed`. |

## Text contract

The shared text contract applies only to supplied artifact input fields and
status-required fields consumed by this evidence path, including `detail`,
`claim`, `reason`, `summary`, `requires_next_evidence`, `does_not_claim`, and
boundary `does_not_prove` values. Optional omissions normalize to conservative
defaults rather than failing validation, except for `schema_version`, top-level
`status`, boundary `id`, boundary `status`, and fields required by a `present`
boundary. This contract is not a general prose linter for PR descriptions or
human review comments.

Artifact text that is supplied or status-required must be non-empty after
trimming and must not contain placeholder or filler language such as:

```text
dummy evidence
sample scenario
TODO: fill this in
lorem ipsum readiness text
example invented classroom event
```

Artifact text must also reject unsupported affirmative claim categories unless a
distinct capability-specific boundary provides evidence for that exact claim:

- affirmative first-lesson completion
- project grading or saved-world grading
- successful creative assessment
- full UI automation success
- RabbitHole whole-lesson completion
- Save completion success
- full world execution success
- deployed sharing or platform success

Allowed limitation wording is explicit and bounded:

```text
First-lesson completion is not proven.
This does not prove completion.
Grading is not assessed.
Creative assessment is not claimed.
UI automation is not complete.
Save completion requires distinct finish-state evidence.
Full world execution, deployed sharing, and platform success are not claimed.
```

## Implementation components

The contract is implemented under `desktop_evidence` without changing the public
readiness purpose.

| Component | Responsibility |
| --- | --- |
| `desktop_evidence::evidence_text_contract` | Shared artifact text validation for filler rejection, unsupported affirmative claim rejection, and allowed limitation wording. |
| `desktop_evidence::first_lesson_boundaries` | Boundary shape validation, required boundary id normalization, status normalization, safe artifact metadata, default missing boundaries, and boundary text checks. |
| `desktop_evidence::first_lesson_next_action` | Desktop next-action artifact validation, optional proof-artifact declaration normalization, optional next-evidence semantics, and text checks. |
| `compare::lesson_readiness` | Consume validation failures through existing readiness reporting without adding new proof behavior. |

Validation errors should identify the field or boundary class. They must not dump
full artifact text, raw artifact contents, screenshots, logs, secrets, or
absolute host paths.

## Related documentation

- [First-Lesson Evidence Readiness](first-lesson-evidence-readiness.md)
- [Lesson Session Readiness](lesson-session-readiness.md)
- [Validation and Quality Gates](validation-quality-gates.md)
