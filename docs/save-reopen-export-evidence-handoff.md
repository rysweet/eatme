# Save, reopen, and export evidence handoff

The save/reopen/export evidence handoff scenario gives instructors and students
a practical bridge from starter-project preflight evidence to a shareable
evidence package.

Use this page when you need to run or edit the scenario, prepare the expected
handoff outputs, or refresh the generated Gadugi adapter.

## Quick start

1. Confirm starter-project preflight evidence exists for the classroom run.
2. Open the canonical scenario prompt:

   ```text
   assets/scenarios/eatme/instructor-student-save-reopen-export-evidence-handoff.yaml#agentic_test_prompt
   ```

3. Record the three required handoff outputs:
   `save_reopen_handoff_card`, `export_evidence_package_checklist`, and
   `instructor_review_boundary_note`.
4. Review whether the output includes the evidence requested by the scenario
   acceptance probes.
5. Keep the handoff package with the instructor review record or next workflow.

## What the scenario covers

`instructor-student-save-reopen-export-evidence-handoff` starts after
`starter-project-open-save-export-preflight` has shown that Alice opened the
bundled starter project and captured inspectable setup evidence.

The handoff scenario asks an instructor acceptance agent to produce:

| Output | Purpose |
| --- | --- |
| Save/reopen handoff card | Tells the student what opened-project reference, save name, save location, and reopen confirmation to record. |
| Export evidence package checklist | Names the saved project reference, reopen confirmation, export or share artifact, and handoff destination. |
| Instructor review boundary note | Separates operational evidence quality from human review and states what the evidence does not prove. |

The scenario does not launch Alice, automate the full UI, grade saved work,
judge creativity, certify a student, or prove learning outcomes. It documents
the evidence package needed before an instructor or later workflow reviews the
saved and exported work.

## Evidence flow

Use the handoff in this order:

1. Run or cite starter-project preflight evidence.
2. Ask the student to record the opened project reference.
3. Ask the student to save the project with a clear name and location.
4. Ask the student to reopen the saved project and record observable
   confirmation.
5. Ask the student to record the export or share artifact expected by the
   assignment.
6. Record the handoff destination, such as an instructor review queue, class
   folder, optional LMS folder, optional shared drive, or next workflow.
7. Add a review boundary note that explains what still needs human review.

Good evidence is visible and easy to find. Use artifact names, screenshot
references, short confirmation notes, class-folder names, or optional LMS
submission labels when the class already uses an LMS. Do not require private
paths, credentials, tokens, hidden log-only signals, brittle UI coordinates, or
unsupported implementation details.

The export/share package should include only the intended review artifacts, such
as the saved project or export reference, reopen confirmation, student
explanation, and review boundary note. Exclude secrets, credentials, access
tokens, personal data, private paths, unrelated local files, and any artifact the
student or instructor did not mean to share.

## Configuration

The handoff is configured through editable scenario assets, not environment
variables:

| Setting | Source |
| --- | --- |
| Scenario id | `id: instructor-student-save-reopen-export-evidence-handoff` |
| Scenario kind | `kind: instructor_agentic_flow` |
| Instructor personas | `personas.instructors` in the canonical scenario |
| Student personas | `personas.students` in the canonical scenario |
| Expected outputs | `agentic_flow.expected_outputs` in the canonical scenario |
| Review boundary | `unsupported_policy`, `avoid`, and `rubric` in the canonical scenario |

No authentication, network service, external storage, or privileged automation is
required. `NODE_OPTIONS` is not part of the scenario contract. If a local
Node-based wrapper hosts the agentic review and hits a heap limit, use the saved
wrapper preference for that invocation:

```bash
export NODE_OPTIONS=--max-old-space-size=32768
```

The Rust asset commands and canonical scenario assets do not require
`NODE_OPTIONS`.

## Run an instructor acceptance review

Use the canonical scenario as the prompt source for an instructor acceptance
agent:

```text
assets/scenarios/eatme/instructor-student-save-reopen-export-evidence-handoff.yaml#agentic_test_prompt
```

The agent output must include the three expected outputs:

```text
save_reopen_handoff_card
export_evidence_package_checklist
instructor_review_boundary_note
```

A useful handoff card uses plain classroom wording:

```text
Opened project reference: starter project preflight manifest or screenshot
Save name and location: MyStarterProject.a3p in the class project folder
Reopen check: student reopened MyStarterProject.a3p and recorded a screenshot
Export or share artifact: exported package selected by the assignment
Handoff destination: instructor review queue or optional LMS folder
Review boundary: this package is evidence for review, not a grade or proof of learning
```

## Output contract

Consumers should treat the three named outputs as the public handoff API:

| Output | Required fields | Must not include |
| --- | --- | --- |
| `save_reopen_handoff_card` | Opened-project reference, save name, save location, observable reopen confirmation request | Private local paths, credentials, hidden implementation assertions |
| `export_evidence_package_checklist` | Saved project reference, reopen confirmation reference, export/share artifact reference, handoff destination | Claims that preflight evidence proves export/share completion |
| `instructor_review_boundary_note` | Human review boundary, unsupported claims, next-review owner | Automated grading, creative assessment, mastery, certification, proof of learning |

Reviewers can accept artifact references such as screenshot names, manifest
paths, optional LMS submission labels, class-folder names, or student-written
confirmation notes. The scenario asks reviewers to reject evidence that only a
machine can inspect or that depends on brittle UI coordinates.

## Example handoff package

```markdown
# Save/reopen/export handoff

Opened project reference: `runs/starter-project-open-save-export-preflight/local-starter-project-open-save-export-preflight/manifest.json`

Save/reopen handoff card:
- Save name: `Maya-Starter-Scene.a3p`
- Save location: class project folder
- Reopen confirmation: screenshot `maya-starter-reopened.png` and student note
  "I reopened the saved file before exporting."

Export evidence package checklist:
- Saved project reference: `Maya-Starter-Scene.a3p`
- Reopen confirmation: `maya-starter-reopened.png`
- Export/share artifact: assignment upload `maya-starter-evidence.zip`
- Handoff destination: Unit 1 Alice evidence review queue

Instructor review boundary:
This package is operational evidence for instructor review. It does not grade the
world, assess creativity automatically, certify mastery, or prove the student
learned the target concepts.
```

## Tutorial: prepare a classroom handoff

Use this sequence when introducing the workflow to a class:

1. Show students where the starter-project preflight evidence is recorded.
2. Ask students to save the opened project with a name that includes their name or
   team name and the assignment label.
3. Ask students to close and reopen the saved project.
4. Ask students to capture one visible confirmation that the reopened project is
   the saved work they intend to submit.
5. Ask students to prepare the export or share artifact named by the assignment.
6. Ask students to submit the export/share artifact with the reopen confirmation
   and a short explanation.
7. Review the package as evidence for feedback, not as an automated grade.

## Canonical asset

Edit the canonical eatme scenario when the handoff wording changes:

```text
assets/scenarios/eatme/instructor-student-save-reopen-export-evidence-handoff.yaml
```

The non-code wording fields are:

| Field | What to edit |
| --- | --- |
| `resource_basis` | Links to the starter-project preflight and related share-evidence scenario. |
| `purpose` | The short evidence boundary for the handoff. |
| `agentic_test_prompt` | The instructor acceptance prompt. |
| `acceptance_criteria` | Given/when/then expectations for save, reopen, export, handoff, and unsupported claims. |
| `acceptance_probes` | Checklist items used to inspect the agent output. |
| `rubric` | Evidence criteria for save/reopen, package readiness, and review boundary. |
| `avoid` | Language and evidence types the agent must not use. |

Keep wording practical. Describe artifact references, observable confirmation,
handoff destination, and human review boundaries. Do not claim automated
grading, creative assessment, learner-world judgment, certified work, mastery,
full Alice coverage, or proof that a student learned the material.

## Gadugi adapter

The generated adapter is:

```text
assets/scenarios/gadugi/instructor-student-save-reopen-export-evidence-handoff.yaml
```

Do not hand-edit the generated adapter to change intent. Edit the canonical
eatme scenario and regenerate:

```bash
cargo run -q -p eatme-cli -- assets generate-gadugi --json
```

Check committed adapter freshness:

```bash
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
```

## Validate the contract

Validate assets:

```bash
cargo run -q -p eatme-cli -- assets validate --json
```

Check generated adapter consistency:

```bash
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
```

Build the documentation site:

```bash
mkdocs build --strict
```
