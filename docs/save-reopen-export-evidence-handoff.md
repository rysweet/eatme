# Save, reopen, and export evidence handoff

The save/reopen/export evidence handoff scenario gives instructors and students
a practical bridge from starter-project preflight evidence to a shareable
evidence package.

Use this page when you need to run or edit the scenario, prepare the expected
handoff outputs, or refresh the generated Gadugi adapter.

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
5. Ask the student to prepare an export or share artifact.
6. Record the handoff destination, such as an instructor, LMS folder, shared
   drive, or next workflow.
7. Add a review boundary note that explains what still needs human review.

Good evidence is visible and easy to find. Use artifact names, screenshot
references, short confirmation notes, shared-folder names, or LMS submission
labels. Do not require private paths, credentials, tokens, hidden log-only
signals, brittle UI coordinates, or unsupported implementation details.

The export/share package should include only the intended review artifacts, such
as the saved project or export reference, reopen confirmation, student
explanation, and review boundary note. Exclude secrets, credentials, access
tokens, personal data, private paths, unrelated local files, and any artifact the
student or instructor did not mean to share.

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
Export or share artifact: exported package or share link for instructor review
Handoff destination: instructor LMS assignment folder
Review boundary: this package is evidence for review, not a grade or proof of learning
```

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
