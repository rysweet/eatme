# Evidence Artifact Contract

The evidence artifact contract defines the shape and wording accepted by
first-lesson readiness and silver-thread desktop evidence validation. It keeps
readiness reports evidence-bound, rejects malformed artifacts, and prevents
placeholder or unsupported success language from entering plain output, JSON
reports, or PR handoffs.

This contract applies to the existing readiness and desktop evidence path. It
does not add completion detection, grading, creative assessment, generated
scenario content, or full Alice UI automation.

## Quick start

Validate a comparison manifest before using its first-lesson evidence:

```bash
export NODE_OPTIONS=--max-old-space-size=32768

cargo run -q -p eatme-cli -- alice check-lesson-readiness \
  --manifest runs/comparisons/first-lessons-real-ui-actions/local/comparison-manifest.json \
  --json
```

The command fails closed for malformed readiness evidence. If a desktop
next-action artifact, evidence boundary, proof-artifact declaration, or evidence
text field violates this contract, the readiness result is `not_ready` or
`blocked`, and the problem appears in `issues`, `not_yet_shown`,
`evidence_progress`, or the affected `evidence_boundaries[]` entry.

Use the result only for the bounded claims named in the report. A passing shape
check does not prove first-lesson completion, grading, creative assessment, Save
completion, rendering correctness, or full UI automation.

## Validation scope

The validator reads the same evidence surfaces used by
[First-Lesson Evidence Readiness](first-lesson-evidence-readiness.md) and
[Lesson Session Readiness](lesson-session-readiness.md):

| Surface | Contract enforced |
| --- | --- |
| `desktop-first-lesson-next-action.json` | Schema version, status, candidate actions, required next evidence, non-claims, proof-artifact declarations, evidence boundaries, and safe wording. |
| `evidence_boundaries[]` | Required boundary ids, supported status values, source, metadata state, detail, claim, non-claims, artifact metadata, and safe wording. |
| Save Project and Select Project proof artifacts | Declaration shape, safe artifact path, readable artifact metadata, and distinct `present`, `missing`, or `blocked` state. |
| Readiness report text fields | Rejection of filler wording and unsupported affirmative claims. |

Validation is limited to readiness and desktop evidence artifacts. Scenario
authoring, adapter generation, Alice launch, RabbitHole execution, grading, and
UI automation remain separate systems.

## Required artifact shape

Desktop next-action evidence uses the existing artifact path:

```text
run-window-evidence/desktop-first-lesson-next-action.json
```

The artifact must be valid JSON, safely rooted under the comparison evidence
directory, and use this schema version:

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

Required collections must be arrays and must not be empty when the artifact uses
them to support a readiness claim. Required strings must be strings after
trimming and must not be empty. Unknown or misspelled status values are invalid.
Missing required evidence-boundary fields are invalid.

### Required evidence boundaries

Readiness validates the complete first-lesson boundary set. Each boundary entry
must have a stable `id`, supported `status`, display-safe `detail`, bounded
`claim`, and a non-empty `does_not_prove` list when the boundary can otherwise
be misread as a capability claim.

| Boundary id | Required meaning | Required non-claim boundary |
| --- | --- | --- |
| `select_project` | Select Project scenario evidence | Does not prove full UI automation or first-lesson completion. |
| `procedure_edit` | Procedure/edit scenario evidence | Does not prove code correctness, grading, or first-lesson completion. |
| `save_project` | Save action or proof-artifact availability | Does not prove Save completion, grading, creative assessment, or first-lesson completion. |
| `visible_rendering` | Visible rendering observation | Does not prove rendering correctness, creative assessment, or first-lesson completion. |
| `grading` | Grading boundary evidence | Does not prove creative assessment or first-lesson completion unless distinct evidence exists. |
| `creative_assessment` | Creative assessment boundary evidence | Does not replace instructor judgment or prove first-lesson completion. |
| `first_lesson_completion` | Completion boundary evidence | Does not prove full UI automation or creative quality unless distinct evidence exists. |

Absent, non-array, or partially malformed `evidence_boundaries` input is not
silently normalized into success. The affected boundary is reported as `missing`
or `invalid`, and readiness does not promote it to `shown`.

## Text contract

Evidence text is treated as untrusted input. The shared
`desktop_evidence::evidence_text_contract` validation applies to artifact text
fields such as `detail`, `claim`, `reason`, `summary`, `requires_next_evidence`,
`does_not_claim`, and boundary `does_not_prove` values.

### Rejected filler wording

Readiness evidence must describe a real bounded observation or limitation. It
rejects placeholder wording such as:

```text
dummy evidence
sample scenario
TODO: fill this in
lorem ipsum readiness text
example invented classroom event
```

The validator also rejects unsupported scenario narrative that is not tied to a
readiness evidence boundary. Use scenario assets for authored classroom intent;
use evidence artifacts only for observations, blockers, required next evidence,
and non-claims.

### Rejected unsupported affirmative claims

The validator rejects affirmative claims that the artifact does not explicitly
support. These phrases are invalid when they appear as success claims without a
matching capability-specific evidence boundary:

```text
The first lesson is complete.
The project was graded.
Creative assessment passed.
Full UI automation succeeded.
The saved world received a grade.
RabbitHole completed the whole first lesson.
```

The rejection applies to equivalent wording, not only these exact sentences.
Launch evidence, Save option evidence, screenshot evidence, proof-artifact
availability, and Run-window evidence cannot be reworded as completion, grading,
creative assessment, or full UI automation.

### Allowed limitation wording

Restrained negative or limitation wording is valid and should be preferred in
human reports:

```text
First-lesson completion is not proven.
This does not prove completion.
Grading is not assessed.
Creative assessment is not claimed.
UI automation is not complete.
Save completion requires distinct finish-state evidence.
```

Allowed limitation wording must stay attached to the relevant boundary or
next-action evidence. Do not hide limitations in comments, generated adapters,
or PR prose only.

## API reference

The internal Rust validator is organized under `desktop_evidence`:

| Component | Responsibility |
| --- | --- |
| `desktop_evidence::evidence_text_contract` | Shared artifact text validation for filler rejection, unsupported affirmative claim rejection, and allowed limitation wording. |
| `desktop_evidence::first_lesson_boundaries` | Boundary shape validation, required boundary ids, status normalization, safe artifact metadata, and boundary text contract checks. |
| `desktop_evidence::first_lesson_next_action` | Desktop next-action artifact validation, proof-artifact declarations, next-evidence semantics, and text contract checks. |
| `compare::lesson_readiness` | Consumes validation failures through existing readiness reporting without adding new evidence behavior. |

Validation failures are surfaced through the existing `ValidationError` and
readiness issue paths. Error messages identify the field or boundary class, but
do not dump full artifact text, artifact contents, screenshots, logs, secrets,
or absolute host paths.

### `desktop-first-lesson-next-action.json`

| Field | Type | Required | Contract |
| --- | --- | --- | --- |
| `schema_version` | string | Yes | Must be `eatme.alice-desktop-first-lesson-next-action/v1`. |
| `status` | string | Yes | Must be a supported readiness state such as `present`, `missing`, `blocked`, or `invalid`. |
| `detail` or `reason` | string | Yes | Must be display-safe, non-empty, and evidence-bound. |
| `candidate_actions` | array of strings | Required when next actions are claimed | Must contain non-empty action ids or labels. |
| `requires_next_evidence` or `requiresNextEvidence` | array of strings | Required when evidence remains outstanding | Must name concrete next evidence, not a success claim. |
| `does_not_claim` or `doesNotClaim` | array of strings | Yes | Must preserve non-claims for completion, grading, creative assessment, and UI automation. |
| `save_project_proof_artifact` | object | Yes | Must declare `present`, `missing`, or `blocked`; present artifacts must be safely rooted and readable. |
| `select_project_proof_artifact` | object | Yes | Must declare `present`, `missing`, or `blocked`; present artifacts must be safely rooted and readable. |
| `evidence_boundaries` or `evidenceBoundaries` | array | Yes | Must include valid boundary entries for the first-lesson boundary set. |

### `evidence_boundaries[]`

| Field | Type | Required | Contract |
| --- | --- | --- | --- |
| `id` | string | Yes | Must match a known first-lesson boundary id. |
| `status` | string | Yes | Must be supported; unknown or empty status is invalid. |
| `source` | string | Yes | Short source category such as `automation_scenario`. |
| `metadata_state` or `metadataState` | string | Yes | Declares metadata availability without upgrading claim support. |
| `detail` | string | Yes | Display-safe observation or limitation text. |
| `claim` | string | Yes | Bounded claim only; no unsupported completion or grading claims. |
| `does_not_prove` or `doesNotProve` | array of strings | Yes | Non-empty for capability-sensitive boundaries. |
| `artifact` | object | Optional | If present, path metadata must resolve under the comparison evidence root. |

## Configuration

| Setting | Required for | Description |
| --- | --- | --- |
| `NODE_OPTIONS=--max-old-space-size=32768` | Agentic and Gadugi-heavy local runs | Saved local preference for Node-backed runner capacity. |
| `EATME_REAL_ALICE=1` | Real Alice desktop execution | Explicit opt-in gate for desktop runs that produce evidence. |
| `ALICE_BASELINE_HOME` | First-lesson comparison execution | Original Alice checkout used by the baseline target. |
| `ALICE_MODERNIZED_HOME` | First-lesson comparison execution | RabbitHole or modernized Alice checkout used by the modernized target. |

The evidence artifact contract has no separate feature flag. It is part of the
readiness validation path, so CI and local runs receive the same failures.

## Tutorials

### Repair a malformed boundary artifact

1. Run `alice check-lesson-readiness --json` for the comparison manifest.
2. Find the invalid boundary in `issues`, `not_yet_shown`, or
   `evidence_boundaries[]`.
3. Fix the artifact shape: use strings for text fields, arrays for collections,
   supported status values, and safe evidence-root-relative artifact paths.
4. Keep the boundary claim narrow. For Save evidence, describe observed Save
   action or artifact availability, not Save completion.
5. Re-run readiness and confirm the boundary is shown only for its bounded
   evidence claim.

Safe repaired Save boundary:

```json
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
```

### Replace unsupported success wording

When a report says a claim is rejected for overclaiming, rewrite the text as a
bounded observation plus a limitation:

| Unsafe wording | Safe replacement |
| --- | --- |
| `The first lesson is complete.` | `First-lesson completion is not proven by this evidence.` |
| `The project was graded.` | `Grading is not assessed by this evidence.` |
| `Creative assessment passed.` | `Creative assessment is not claimed by this evidence.` |
| `Full UI automation succeeded.` | `UI automation is not complete; this evidence covers only the named boundary.` |
| `Save completed successfully.` | `Save action evidence is present; Save completion requires distinct finish-state evidence.` |

### Review a PR safely

1. Read `Shown` as bounded evidence only.
2. Read every `Not yet shown` line before approving a readiness claim.
3. Check `evidence_boundaries[]` for invalid, missing, or empty boundary fields.
4. Confirm `does_not_claim` and `does_not_prove` still preserve completion,
   grading, creative assessment, Save completion, and full UI automation
   limitations.
5. Reject PR wording that converts observations into completed lessons, grades,
   creative assessment, or full UI automation.

## Related documentation

- [First-Lesson Evidence Readiness](first-lesson-evidence-readiness.md)
- [Lesson Session Readiness](lesson-session-readiness.md)
- [Validation and Quality Gates](validation-quality-gates.md)
