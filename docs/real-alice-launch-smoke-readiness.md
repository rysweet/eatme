# Real Alice launch-smoke readiness

Real Alice launch-smoke readiness maps existing `real-alice-launch-smoke`
manifest evidence into the shared readiness report shape. It is a bounded
reporting path only: it consumes launch-smoke evidence that already exists and
does not launch Alice, inspect the UI, run a full world, grade work, assess
creativity, complete Save, prove sharing/platform deployment, or add new desktop
automation.

The report answers one question:

> Did the selected `real-alice-launch-smoke` comparison carry enough existing
> launch-smoke manifest evidence to report launch-smoke readiness?

It does not answer whether first-lesson completion, full world execution,
project correctness, creative quality, visible rendering correctness, grading,
Save completion, sharing/platform deployment, or full Alice UI automation has
been proven.

## When to use it

Use this report when a reviewer, CI job, or lane PR needs a conservative
readiness summary for the baseline real Alice launch smoke.

| Need | Use |
| --- | --- |
| Show that real Alice launch-smoke evidence is present and coherent | `alice check-lesson-readiness` on a `real-alice-launch-smoke` comparison manifest |
| Explain missing, failed, malformed, or manifest-only launch evidence | The same readiness report with `status: "not_ready"` or `status: "blocked"` |
| Claim a completed Alice lesson, full world execution, grading result, creative quality, Save completion, deployed sharing/platform success, Full Alice UI automation, or visible rendering correctness | Do not use launch-smoke readiness for this |

`real-alice-launch-smoke` is matched by exact scenario id. Other scenario ids
continue to use their existing readiness behavior.

## Usage

Create or execute the launch-smoke comparison through the existing comparison
path:

```bash
export NODE_OPTIONS=--max-old-space-size=32768

cargo run -q -p eatme-cli -- alice compare-launch-smoke \
  --run-id local-real-alice-launch-smoke \
  --json
```

When execution is requested and both comparison targets are configured, provide
the target Alice homes. The baseline `real-alice-launch-smoke` scenario must not
gain a new `EATME_REAL_ALICE=1` requirement; leaving that variable set from other
workflows is harmless, but it is required only for non-baseline scenario ids:

```bash
export NODE_OPTIONS=--max-old-space-size=32768
export ALICE_BASELINE_HOME=/path/to/alice-reference
export ALICE_MODERNIZED_HOME=/path/to/alice-candidate

cargo run -q -p eatme-cli -- alice compare-launch-smoke \
  --run-id local-real-alice-launch-smoke \
  --json \
  --no-memory \
  --offline-package \
  --execute
```

Then check readiness from the comparison manifest:

```bash
cargo run -q -p eatme-cli -- alice check-lesson-readiness \
  --manifest runs/comparisons/real-alice-launch-smoke/local-real-alice-launch-smoke/comparison-manifest.json \
  --json
```

Without `--json`, the same command renders a plain bounded readiness report for
humans and PR descriptions.

## Implementation contract

The implementation is an exact `scenario_id == "real-alice-launch-smoke"` branch
inside the existing `alice check-lesson-readiness` path. It must not add a new
command, change first-lesson readiness semantics, require `EATME_REAL_ALICE=1`
for the baseline scenario, launch Alice, or read artifact files from paths named
in the manifest.

The branch must inspect only the comparison manifest and embedded target launch
manifests. Artifact paths are evidence only as manifest-level metadata: the
readiness report may summarize that log, window, screenshot, and startup
artifacts were recorded, but it must not open those files to derive new facts.

## Evidence mapping

The launch-smoke readiness branch consumes only existing manifest evidence. It
does not read screenshots, logs, DOM state, browser state, learner content, or
raw tool output to create new evidence.

| Evidence | Ready mapping | Non-ready mapping |
| --- | --- | --- |
| Exact scenario id | The comparison and embedded launch manifests identify `real-alice-launch-smoke`. | Wrong, missing, or ambiguous scenario ids are `not_ready`. |
| Executed launch-smoke target evidence | Required embedded target launch manifests for the baseline and modernized targets are present and structurally valid. | Manifest-only, missing, partial, or malformed launch evidence is `not_ready`. |
| Launch result | Required launch-smoke evidence has no failure category and reports successful required assertions. | Failed assertions, non-null failure categories, incomplete status, or contradictory evidence are `not_ready` unless an explicit known blocker is present. |
| Launch-smoke assertions | Existing manifest assertions needed for launch-smoke readiness pass, including real Alice execution evidence and required startup artifact metadata. | Missing, malformed, failed, or unsafe assertion evidence is `not_ready`. |
| Artifacts represented by the manifest | Manifest-level artifact metadata for log, window, screenshot, and startup evidence is safe to summarize as launch-smoke evidence. | Missing required artifact metadata remains `not_ready`; explicit unsupported-environment or dependency blockers remain `blocked` when represented as blockers. |

Successful launch-smoke evidence maps to `status: "ready"` only for the bounded
launch-smoke claim. Missing, partial, malformed, unsafe, manifest-only, failed,
or contradictory evidence maps to `status: "not_ready"`. Evidence that is
coherent but carries an explicit known blocker maps to `status: "blocked"`.

## Plain output contract

Plain output uses launch-smoke-specific wording and keeps the scope boundary
visible. A ready report may look like this:

```text
Real Alice launch-smoke readiness: ready

Shown:
- Real Alice launch-smoke manifest evidence is shown.
- Required launch-smoke assertions are shown.
- Launch-smoke artifact metadata is shown.

Not yet shown:
- No missing launch-smoke evidence is reported.

Unproven:
- First-lesson completion is not proven.
- Full world execution is not proven.
- Grading is not proven.
- Creative assessment is not proven.
- Full Alice UI automation is not proven.
- Visible rendering correctness is not proven.
- Save completion is not proven.
- Deployed sharing/platform success is not proven.
```

A non-ready report keeps failures as missing or blocked evidence instead of
turning them into success:

```text
Real Alice launch-smoke readiness: not ready

Shown:
- Real Alice launch-smoke scenario identity is shown.

Not yet shown:
- Required launch-smoke manifest evidence is not yet shown.
- Required launch-smoke assertions are not yet shown.
- Launch-smoke artifact metadata is not yet shown.

Unproven:
- First-lesson completion is not proven.
- Full world execution is not proven.
- Grading is not proven.
- Creative assessment is not proven.
- Full Alice UI automation is not proven.
- Visible rendering correctness is not proven.
- Save completion is not proven.
- Deployed sharing/platform success is not proven.
```

The launch-smoke report does not render first-lesson-specific claims such as
Select Project, procedure/edit, Save completion, desktop next action, grading
evidence, creative assessment evidence, or first-lesson completion evidence.
It also does not claim full world execution or deployed sharing/platform
success.

## JSON API

`alice check-lesson-readiness` emits the existing readiness schema for this
scenario:

```text
eatme.alice-lesson-session-readiness/v1
```

Consumers should use the normalized `status` field and the user-facing evidence
arrays. Legacy fields remain available for compatibility, but they must not be
interpreted as lesson completion or assessment.

| Field | Type | Launch-smoke meaning |
| --- | --- | --- |
| `scenario_id` | string | Must be `real-alice-launch-smoke` for this branch. |
| `status` | string | `ready`, `not_ready`, or `blocked` for bounded launch-smoke readiness. |
| `passed` | boolean | Structural readiness result for the evidence that was inspected. |
| `readiness_status` | string | Backward-compatible detailed status. Prefer `status` for new consumers. |
| `shown_evidence` | array | Launch-smoke evidence facts that are present and safe to summarize. |
| `not_yet_shown` | array | Missing, failed, malformed, unsafe, incomplete, or blocked launch-smoke evidence. |
| `unproven_claims` | array | Required non-claims for launch-smoke readiness. |
| `target_evidence` | array | Existing per-target launch-smoke evidence summarized from the comparison manifest. |
| `issues` | array | Structural problems such as missing manifests, wrong scenario ids, failed assertions, or malformed evidence. |
| `limitations` | array | Compatibility non-claims; must include the launch-smoke non-claims. |

Example ready JSON excerpt:

```json
{
  "schema_version": "eatme.alice-lesson-session-readiness/v1",
  "scenario_id": "real-alice-launch-smoke",
  "status": "ready",
  "passed": true,
  "shown_evidence": [
    {
      "id": "launch_smoke_manifest",
      "state": "present",
      "summary": "Real Alice launch-smoke manifest evidence is shown.",
      "detail": "Existing launch-smoke manifest evidence matched the real-alice-launch-smoke scenario.",
      "does_not_prove": [
        "first-lesson completion",
        "full world execution",
        "grading",
        "creative assessment",
        "full Alice UI automation",
        "visible rendering correctness",
        "Save completion",
        "deployed sharing/platform success"
      ]
    }
  ],
  "not_yet_shown": [],
  "unproven_claims": [
    "First-lesson completion is not proven.",
    "Full world execution is not proven.",
    "Grading is not proven.",
    "Creative assessment is not proven.",
    "Full Alice UI automation is not proven.",
    "Visible rendering correctness is not proven.",
    "Save completion is not proven.",
    "Deployed sharing/platform success is not proven."
  ]
}
```

Example non-ready JSON excerpt:

```json
{
  "schema_version": "eatme.alice-lesson-session-readiness/v1",
  "scenario_id": "real-alice-launch-smoke",
  "status": "not_ready",
  "passed": false,
  "shown_evidence": [
    {
      "id": "scenario_identity",
      "state": "present",
      "summary": "Real Alice launch-smoke scenario identity is shown.",
      "detail": "The comparison manifest selected real-alice-launch-smoke.",
      "does_not_prove": [
        "first-lesson completion",
        "full world execution",
        "full Alice UI automation"
      ]
    }
  ],
  "not_yet_shown": [
    {
      "id": "launch_smoke_manifest",
      "state": "missing",
      "summary": "Required launch-smoke manifest evidence is not yet shown.",
      "detail": "No executed launch-smoke manifest was available for the selected target.",
      "does_not_prove": [
        "first-lesson completion",
        "full world execution",
        "grading",
        "creative assessment",
        "full Alice UI automation",
        "visible rendering correctness",
        "Save completion",
        "deployed sharing/platform success"
      ]
    }
  ],
  "unproven_claims": [
    "First-lesson completion is not proven.",
    "Full world execution is not proven.",
    "Grading is not proven.",
    "Creative assessment is not proven.",
    "Full Alice UI automation is not proven.",
    "Visible rendering correctness is not proven.",
    "Save completion is not proven.",
    "Deployed sharing/platform success is not proven."
  ]
}
```

## Configuration

Launch-smoke readiness adds no new configuration. It uses the existing manifest
path supplied to `alice check-lesson-readiness`.

| Setting | Required for | Description |
| --- | --- | --- |
| `NODE_OPTIONS=--max-old-space-size=32768` | Local validation and agentic/Gadugi-heavy runs | Saved local preference for Node-backed runner capacity. |
| `EATME_REAL_ALICE=1` | Executed non-baseline real Alice comparison runs | Explicit opt-in for desktop execution. |
| `ALICE_BASELINE_HOME` | Executed comparison runs | Original/reference Alice checkout. |
| `ALICE_MODERNIZED_HOME` | Executed comparison runs | Candidate/RabbitHole Alice checkout. |

The readiness check itself does not package Alice, start Xvfb, launch Java,
collect screenshots, run browser automation, or open UI windows. It maps the
manifest evidence already produced by the launch-smoke harness.

## Validation commands

Use the existing asset checks to keep the documentation, canonical scenario, and
generated Gadugi adapter aligned:

```bash
export NODE_OPTIONS=--max-old-space-size=32768

cargo run -q -p eatme-cli -- assets validate --json
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
```

For a focused scenario check:

```bash
export NODE_OPTIONS=--max-old-space-size=32768

cargo run -q -p eatme-cli -- assets validate \
  --path assets/scenarios/eatme/real-alice-launch-smoke.yaml \
  --json
```

## Safe PR wording

Use wording that names the bounded report:

```text
Maps existing real-alice-launch-smoke manifest evidence into bounded readiness
output. The report shows launch-smoke readiness only and keeps first-lesson
completion, full world execution, grading, creative assessment, Full Alice UI
automation, visible rendering correctness, Save completion, and deployed
sharing/platform success explicitly unproven.
```

Avoid wording that promotes launch smoke to lesson, UI, rendering, grading,
creative-assessment, Save, sharing, platform, or deployment success. Those
outcomes remain not proven by this report.
