# Sharing readiness boundary

Student and teacher sharing scenarios define a review handoff, not a deployed
sharing feature. They help a learner or instructor package enough evidence for a
classroom review, peer conversation, or teacher remix decision without claiming
that hosted sharing, deployed sharing, platform success, a live community
platform, or platform access controls work.

## What the boundary means

| Audience | Ready means | Ready does not mean |
| --- | --- | --- |
| Student | The student can hand off a packet that names the Alice artifact, visible behavior, run result, context or attribution, and one next revision. | The artifact was uploaded, hosted, published to a community platform, or proven available through a deployed service. |
| Instructor | The instructor can review the packet, connect it to classroom expectations, and decide what feedback or remix step comes next. | The instructor has proof of a live sharing platform, public gallery, account workflow, moderation, permissions, or community deployment. |
| Teacher-community curator | The curator can package a teacher-facing share card with classroom constraints, student evidence expectations, accessibility notes, attribution, and remix feedback prompts. | The curator has proven that the activity is distributed through a hosted teacher community or ranked by a platform. |

The scenario names use "share" in the classroom sense: a packet, card, note, or
prompt that one person can review. The word does not imply deployment.

## Quick start

Use the sharing readiness scenarios when a student or teacher needs a bounded
handoff artifact:

1. Choose the scenario that matches the handoff:
   `student-artifact-package-share-evidence` for one learner artifact, or
   `teacher-community-sharing-loop` for a teacher-facing activity note.
2. Collect the required packet fields from the scenario contract.
3. Add the review boundary in plain language.
4. Validate the asset set and generated adapters before using the packet in an
   agentic or Gadugi flow.

The output is ready when another person can review the artifact, evidence, and
next step without needing a public URL, account workflow, hosted gallery,
deployment log, or platform screenshot.

## Student share packet

Use `student-artifact-package-share-evidence` when the learner is ready to hand
off one Alice artifact or screenshot for review.

The packet is complete when it contains:

| Packet item | Required content |
| --- | --- |
| Artifact reference | The Alice world, screenshot, classroom artifact, or exported file if one is already available. |
| Student change | One student-owned change to scene, code, camera, audio, data, interaction, or timing. |
| Visible run result | What another person can observe after the world or artifact is run or viewed. |
| Context or attribution | Classroom context, source attribution, peer role, or resource note needed for fair review. |
| Next revision | One specific change the student would try next based on evidence or feedback. |
| Review boundary | A plain statement that the packet is for instructor or peer review, not proof of deployed sharing. |

Good packets are small and reviewable. They do not need a public URL, account,
hosted gallery entry, deployment log, or platform screenshot.

## Instructor handoff

Use the student packet as evidence for a classroom review conversation:

1. Confirm the packet names the artifact and visible behavior.
2. Check that the student explains their own change in plain language.
3. Separate setup evidence from learner evidence.
4. Ask for one next revision instead of treating the first packet as final.
5. Record feedback as a classroom note, rubric response, or remix prompt.

The instructor may attach a real Alice launch manifest as setup evidence when the
mission needs it. That manifest proves only the stated launch-smoke boundary. It
does not prove learner understanding, artifact quality, public sharing, hosted
availability, or platform success.

### Tutorial: review a student packet

A reviewer can use this short loop for a classroom conversation:

1. Ask the student to point to the artifact or screenshot.
2. Ask what they changed in Alice.
3. Run or view the artifact evidence and name the visible result.
4. Check attribution, classroom context, or partner role.
5. Ask for one next revision.
6. Record feedback without treating the packet as a deployed sharing result.

The handoff loop is complete when the next revision is clear. It is not blocked
by the absence of a hosted gallery entry or publishing workflow.

## Teacher-community handoff

Use `teacher-community-sharing-loop` when the output is a teacher-facing activity
handoff. The handoff includes:

| Output | Purpose |
| --- | --- |
| Share card | Summarizes the activity, learning goal, artifact expectations, attribution, and reuse context. |
| Classroom handoff note | Names setup constraints, student evidence expectations, accessibility notes, and review responsibilities. |
| Remix feedback prompt | Asks another teacher what they would keep, adapt, simplify, or test next. |

The handoff is editable classroom documentation. It is not a platform publishing
flow and should not describe public distribution unless a separate
platform/deployment feature is explicitly scoped and proven.

### Tutorial: prepare a teacher handoff

Use this sequence when a teacher wants another teacher to reuse or adapt an
Alice activity:

1. Name the activity, audience, prerequisites, timing, and setup constraints.
2. Link the editable scenario and persona assets that ground the activity.
3. Describe the student evidence another teacher should expect.
4. Add attribution and any accessibility notes.
5. Ask two remix questions: what to keep, and what to change next time.
6. Keep the handoff editable so the next classroom can adapt it.

The handoff succeeds when another teacher can understand the classroom activity
and provide useful remix feedback. It does not require a teacher account,
ranking, moderation queue, public gallery, or platform distribution step.

## Configuration

No deployment or platform configuration is required for sharing readiness.
`NODE_OPTIONS` is optional local runtime tuning for memory-heavy agentic or
Gadugi runs; omit it unless local validation needs the extra heap:

```bash
export NODE_OPTIONS=--max-old-space-size=32768
```

Use the repository asset checks to validate the scenario documentation and
generated adapters:

```bash
cargo run -q -p eatme-cli -- assets validate --json
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
```

These commands validate asset shape and adapter freshness. They do not upload,
host, publish, moderate, or prove any deployed sharing service.

## PR 173 exact-head readiness evidence

PR 173 readiness was evaluated on branch
`wave6-deployed-sharing-gap-1778302300`.

| Evidence item | Result |
| --- | --- |
| exact evaluated HEAD SHA | `7757f298bbdf220b37882c912abb05cae2277bd8` |
| master sync status | `origin/master` is an ancestor of the exact evaluated HEAD SHA; no rebase was required for this check. |
| validation commands | `NODE_OPTIONS=--max-old-space-size=32768 mkdocs build --strict`; `NODE_OPTIONS=--max-old-space-size=32768 cargo run -q -p eatme-cli -- assets validate --json`; `NODE_OPTIONS=--max-old-space-size=32768 cargo run -q -p eatme-cli -- assets generate-gadugi --check --json`; `TMPDIR=/tmp NODE_OPTIONS=--max-old-space-size=32768 ./scripts/quality-gates.sh` |
| readiness result | The listed documentation, asset, generated-adapter, and quality gates completed for this readiness boundary. |

This evidence is limited to PR 173 readiness. It does not claim hosted sharing,
deployed sharing, platform success, full UI automation, grading, creative
assessment, Save completion, visible rendering correctness, or first-lesson
completion.

## Scenario contract reference

The sharing readiness boundary is expressed through scenario assets, not a
network API.

| Scenario id | Contract |
| --- | --- |
| `student-artifact-package-share-evidence` | Student packet for artifact reference, student change, visible run result, attribution or classroom context, next revision, and review boundary. |
| `teacher-community-sharing-loop` | Teacher-facing handoff for share card, classroom note, accessibility notes, attribution, student evidence expectations, and remix feedback. |

## Output contract reference

Sharing readiness outputs are plain review artifacts. They should use these
fields so humans and agents can inspect them consistently.

| Output | Required fields |
| --- | --- |
| Student artifact review packet | Artifact reference, student change, visible run result, context or attribution, next revision, review boundary |
| Student evidence handoff prompt | Artifact reference, observable behavior question, student explanation question, feedback request, revision request |
| Instructor review boundary note | Environment evidence if present, learner evidence still required, human judgment still required, unsupported claims |
| Teacher-community share card | Activity purpose, audience, prerequisites, timing, classroom constraints, attribution, editable scenario/persona links, student evidence |
| Classroom handoff note | Setup assumptions, learner-facing evidence, accessibility notes, adaptation choices, support signals |
| Remix feedback prompt | Classroom fit question, learner evidence question, accessibility question, one suggested revision |

Acceptance probes should reject responses that:

- present a packet as proof of hosted or deployed sharing
- require a public URL, account, platform gallery, or deployment log
- rank teachers, classmates, or artifacts by platform popularity
- hide missing learner evidence behind a launch manifest
- omit attribution, classroom context, accessibility notes, or the next revision
  when the scenario asks for them

## Example student packet

```text
Artifact:
Alice world: space-rescue-v2.a3p

Student change:
I changed the astronaut's turn so the rescue ship is visible before the dialog
starts.

Visible run result:
When the world runs, the camera shows the ship first, then the astronaut turns
toward it before speaking.

Context or attribution:
Classroom build based on the scene-composition lesson. My partner suggested the
camera check.

Next revision:
I will shorten the pause before the dialog because the first run felt slow.

Review boundary:
This packet is for peer and instructor review. It does not prove the project was
uploaded, hosted, or shared through a deployed platform.
```

## Example teacher handoff note

```text
Activity:
Two-object scene revision with visible run evidence.

Student evidence expected:
Artifact reference, one student-owned change, visible run result, attribution or
classroom context, and one next revision.

Accessibility note:
Students can submit a screenshot plus written observation if they cannot export
the Alice world during class.

Remix prompt:
What setup step would another teacher need before using this activity, and what
would you simplify for a shorter class period?

Boundary:
This is a classroom handoff. It does not claim a hosted community platform,
public gallery, deployment, permissions model, or moderation workflow.
```
