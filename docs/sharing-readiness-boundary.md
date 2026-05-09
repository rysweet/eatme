# Sharing readiness boundary

Student and teacher sharing scenarios define a review handoff, not a deployed
sharing feature.

Sharing readiness is the classroom handoff layer for Alice artifacts and
teacher-facing activity notes. It helps students, peers, instructors, and
teacher-community curators package reviewable evidence without claiming that a
hosted sharing service, deployed platform, public gallery, moderation workflow,
or access-control system exists.

## Contents

- [What the boundary means](#what-the-boundary-means)
- [Quick start](#quick-start)
- [Student share packet](#student-share-packet)
- [Instructor handoff](#instructor-handoff)
- [Teacher-community handoff](#teacher-community-handoff)
- [Configuration](#configuration)
- [Recovery evidence artifacts](#recovery-evidence-artifacts)
- [Scenario contract reference](#scenario-contract-reference)
- [Output contract reference](#output-contract-reference)
- [Example student packet](#example-student-packet)
- [Example teacher handoff note](#example-teacher-handoff-note)

## What the boundary means

| Audience | Ready means | Ready does not mean |
| --- | --- | --- |
| Student | The student can hand off a packet that names the Alice artifact, observable behavior, context or attribution, and one next revision. | The artifact was uploaded, hosted, published to a community platform, or proven available through a deployed service. |
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
| Observable result | What another person can observe from the artifact, screenshot, or student-described run evidence. |
| Context or attribution | Classroom context, source attribution, peer role, or resource note needed for fair review. |
| Next revision | One specific change the student would try next based on evidence or feedback. |
| Review boundary | A plain statement that the packet is for instructor or peer review, not proof of deployed sharing. |

Good packets are small and reviewable. They do not need a public URL, account,
hosted gallery entry, deployment log, or platform screenshot.

## Instructor handoff

Use the student packet as evidence for a classroom review conversation:

1. Confirm the packet names the artifact and observable behavior.
2. Check that the student explains their own change in plain language.
3. Separate setup evidence from learner evidence.
4. Ask for one next revision instead of treating the first packet as final.
5. Record feedback as a classroom note, rubric response, or remix prompt.

The instructor may attach a real Alice launch manifest as setup evidence when the
mission needs it. That manifest proves only the stated launch-smoke boundary. It
does not prove learner understanding, artifact quality, public sharing, hosted
availability, rendering correctness, grading correctness, or platform success.

### Tutorial: review a student packet

A reviewer can use this short loop for a classroom conversation:

1. Ask the student to point to the artifact or screenshot.
2. Ask what they changed in Alice.
3. Review the artifact evidence and name the observable result.
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
Gadugi runs. Use the repository's saved Node heap preference when running
Node-based workflows around the evidence checks:

```bash
export NODE_OPTIONS=--max-old-space-size=32768
```

The Rust asset validation and Gadugi generator commands do not require Node, but
keeping the variable exported is safe for repository-wide workflow runs.

Use the repository asset checks to validate the scenario documentation and
generated adapters:

```bash
cargo run -q -p eatme-cli -- assets validate --json
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
```

Build documentation in strict mode when this page or linked readiness docs
change:

```bash
mkdocs build --strict
```

These commands validate asset shape and adapter freshness. They do not upload,
host, publish, moderate, or prove any deployed sharing service.

The documentation build validates documentation navigation. It does not
render-check Alice output, grade artifacts, or prove any deployed sharing
service.

## Recovery evidence artifacts

PR recovery records for sharing readiness use current-head evidence. Historical
SHAs, previous validation output, and earlier workflow logs are useful context,
but they are not proof for the branch head currently being reviewed.

Keep exact PR evidence in a review artifact such as a PR comment, session
artifact, or final recovery note. Do not commit time-sensitive SHAs, dirty/clean
worktree claims, or validation outcomes into this guide; those facts expire with
each new commit.

For a sharing-readiness PR such as `#173` on branch
`wave6-deployed-sharing-gap-1778302300`, the review artifact records:

| Evidence item | Source | Rule |
| --- | --- | --- |
| Local branch, head, and worktree state | `git --no-pager status --short --branch`, `git --no-pager rev-parse --abbrev-ref HEAD`, and `git --no-pager rev-parse HEAD` | Cite local validation only for the recorded checkout. |
| PR metadata | `gh pr view 173 --json headRefName,headRefOid,mergeStateStatus,mergeable,statusCheckRollup,reviewDecision,state,url` | Treat GitHub checks and mergeability as evidence for the recorded PR head. |
| Asset validity | `cargo run -q -p eatme-cli -- assets validate --json` | Rerun for the evaluated head before claiming asset readiness. |
| Generated adapter freshness | `cargo run -q -p eatme-cli -- assets generate-gadugi --check --json` | Rerun whenever canonical scenario assets or generated adapters are in scope. |
| Documentation validity | `mkdocs build --strict` | Rerun when this guide or linked readiness docs change. |
| Repository quality | `TMPDIR=/tmp ./scripts/quality-gates.sh` | Rerun before full repository readiness claims. |
| Claim boundary | Review of this page, `docs/default-workflow-pr-readiness.md`, the sharing scenarios, generated adapters, and Rust guard tests | State only the classroom sharing-readiness evidence that was actually checked. |

If local `HEAD` differs from the PR head, the artifact must state the mismatch
and must not describe local validation as proof for the published PR head. If
the heads match and the checks pass, the artifact may say that the current head
satisfies the classroom sharing-readiness boundary.

The recovery statement must remain narrow: it may cite bounded
silver-thread/e2e sharing-readiness evidence for classroom handoff artifacts. It
must not claim hosted sharing, deployed sharing, platform success, full UI
automation, rendering correctness, grading correctness, creative assessment,
Save completion, lesson completion, production readiness, deployment success,
merge completion, or manual merge.

## Scenario contract reference

The sharing readiness boundary is expressed through scenario assets, not a
network API.

| Scenario id | Contract |
| --- | --- |
| `student-artifact-package-share-evidence` | Student packet for artifact reference, student change, observable result, attribution or classroom context, next revision, and review boundary. |
| `teacher-community-sharing-loop` | Teacher-facing handoff for share card, classroom note, accessibility notes, attribution, student evidence expectations, and remix feedback. |

Generated Gadugi adapters consume these scenario contracts. Do not hand-edit the
generated adapters to broaden mission intent; update the canonical eatme
scenario and regenerate adapters instead.

## Output contract reference

Sharing readiness outputs are plain review artifacts. They should use these
fields so humans and agents can inspect them consistently.

| Output | Required fields |
| --- | --- |
| Student artifact review packet | Artifact reference, student change, observable result, context or attribution, next revision, review boundary |
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
- describe screenshot or launch evidence as rendering correctness
- treat packet review as grading correctness, creative assessment, Save
  completion, lesson completion, production readiness, or deployment success
- omit attribution, classroom context, accessibility notes, or the next revision
  when the scenario asks for them

## Example student packet

```text
Artifact:
Alice world: space-rescue-v2.a3p

Student change:
I changed the astronaut's turn so the rescue ship is visible before the dialog
starts.

Observable result:
The packet screenshot shows the rescue ship in frame before the astronaut's
dialog. My run note says the astronaut turns toward the ship before speaking.

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
Two-object scene revision with observable artifact evidence.

Student evidence expected:
Artifact reference, one student-owned change, observable result, attribution or
classroom context, and one next revision.

Accessibility note:
Students can submit a screenshot plus written observation if they cannot export
the Alice world during class.

Remix prompt:
What setup step would another teacher need before using this activity, and what
would you simplify for a shorter class period?

Boundary:
This is a classroom handoff. It does not claim a hosted community platform,
public gallery, deployment, permissions model, moderation workflow, rendering
correctness, grading correctness, creative assessment, Save completion, lesson
completion, production readiness, or platform success.
```
