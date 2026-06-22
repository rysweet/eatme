# Gallery/media/import parity

Use this guide to build and prove the bounded LookingGlass behavior behind the
EatMe gallery/media/import parity rows.

Last updated: 2026-06-22.

## Contents

- [Evidence boundary](#evidence-boundary)
- [Configuration](#configuration)
- [Run LookingGlass evidence checks](#run-lookingglass-evidence-checks)
- [Run EatMe closure checks](#run-eatme-closure-checks)
- [Update the parity matrix](#update-the-parity-matrix)
- [Troubleshooting](#troubleshooting)
- [Related documentation](#related-documentation)

## Evidence boundary

This guide describes the closure state enforced by the parity matrix and
EatMe closure tests. The repository matrix remains the source of truth; do not
promote any row from `partial` to `covered` from documentation alone.

The workstream covers only these parity rows:

| Matrix row | Target LookingGlass status | Evidence boundary |
| --- | --- | --- |
| `model-texture-import-checkpoint` | `covered` | Imported model and texture resources are sanitized, stored as project resources, assigned to scene objects, included in `.a3p` and web-package exports, and still present after reopen/checkpoint validation. |
| `media-audio-cue-storyboard` | `partial` | Audio support is bounded to resource metadata, manifest persistence, storyboard cue timing, and playback-bridge trigger evidence. It does not claim native browser playback, device media permissions, or complete audio authoring. |
| `audio-camera-and-export-sharecase` | `partial` | Camera/viewpoint state, export package generation, validation, and download/share-artifact fallback are covered. The row remains partial while audio is metadata/playback-bridge bounded and native Web Share availability is browser-dependent. |

Do not use this guide to close setup, save/reopen, class-sharing, broad gallery,
or unrelated curriculum gaps.

## Configuration

Use the saved Node memory preference for LookingGlass commands:

```bash
export NODE_OPTIONS=--max-old-space-size=32768
```

Use these environment variables when running EatMe web closure checks:

| Variable | Required value | Used by |
| --- | --- | --- |
| `NODE_OPTIONS` | `--max-old-space-size=32768` | LookingGlass builds, Vitest, Playwright, and Gadugi checks. |
| `EATME_WEB_PLATFORM` | `1` | EatMe web-platform closure tests. |
| `ALICE_WEB_URL` | Base URL for a running LookingGlass server | EatMe tests that call browser/API behavior. |
| `EATME_REAL_ALICE` | `1` | RabbitHole baseline checks only. Not required for LookingGlass closure. |
| `ALICE_HOME` | Path to RabbitHole checkout | RabbitHole baseline checks only. |

## Run LookingGlass evidence checks

Run the targeted source/API tests from the LookingGlass repository:

```bash
cd <lookingglass-repo>
export NODE_OPTIONS=--max-old-space-size=32768

npm run build
npm run test -- \
  test/imported-project-assets-security.contract.test.ts \
  test/model-texture-import-checkpoint-closure.contract.test.ts \
  test/project-audio-bounded-evidence.contract.test.ts \
  test/project-export-share-fallback.contract.test.ts \
  test/imported-project-assets.test.ts \
  test/imported-asset-project-io.test.ts \
  test/model-texture-camera-joint-export-workflow.contract.test.ts \
  test/project-export.test.ts \
  test/camera-workflow.test.ts \
  test/project-audio-contract.test.ts \
  test/project-audio-project-io-contract.test.ts \
  test/audio-workflow-parity.test.ts \
  test/audio.test.ts
```

Run the browser evidence checks when the UI/share fallback behavior is part of
the claim:

```bash
cd <lookingglass-repo>
export NODE_OPTIONS=--max-old-space-size=32768
npm run test:e2e -- e2e/alice-evidence-workflow.spec.ts
npm run test:e2e -- e2e/import-model-texture-workflow.spec.ts
```

The LookingGlass evidence is valid only when the checks prove all relevant
claims:

1. Model imports accept `.glb` and `.gltf`, reject unsupported extensions, reject
   empty payloads, and never persist traversal, absolute paths, encoded path
   escapes, or unsafe archive paths. Encoded separators require direct evidence:
   either reject them or normalize them to a safe basename before persistence.
2. Texture imports accept `.png`, `.jpg`, `.jpeg`, and `.webp`, reject unsupported
   extensions, and bind through project-owned `project/textures/...` resource
   identifiers.
3. Exported `.a3p` and web packages include only sanitized project resource
   paths under `resources/models/` and `resources/textures/`.
4. Reopened project archives preserve imported asset metadata, resource bytes,
   scene model references, texture bindings, camera workflow state, and package
   validation evidence.
5. Audio checks prove `aliceAudio` manifest persistence, cue timing, background
   metadata, supported-format validation, decode-status recording, and
   playback-bridge trigger calls only.
6. Share checks prove deterministic export/download artifacts and fallback when
   native `navigator.share` or `navigator.canShare` is unavailable.

### Canonical LookingGlass evidence references

Use these exact references in EatMe closure assertions so the row does not drift
from the evidence it claims:

| Evidence reference | Claim |
| --- | --- |
| `LookingGlass:test/model-texture-import-checkpoint-closure.contract.test.ts` | Imported model bytes, texture bytes, assignment metadata, camera checkpoint state, export, and reopen persistence. |
| `LookingGlass:test/imported-project-assets-security.contract.test.ts` | Unsafe resource names are rejected and safe model/texture metadata stays project-scoped. |
| `LookingGlass:test/model-texture-camera-joint-export-workflow.contract.test.ts` | Public workflow API imports resources, assigns textures, exports resources, and generates share fallback artifacts. |
| `LookingGlass:test/imported-asset-project-io.test.ts` | Imported asset descriptors, resource bytes, model resource IDs, and texture bindings are written and read through `.a3p`. |
| `LookingGlass:test/project-audio-bounded-evidence.contract.test.ts` | Audio evidence is bounded to metadata and playback-bridge claims, not native playback. |
| `LookingGlass:test/project-export-share-fallback.contract.test.ts` | Export/share evidence is browser-download fallback evidence, not native Web Share success. |

## Run EatMe closure checks

Run the matrix and scenario checks from the EatMe repository:

```bash
cd <eatme-repo>

cargo run -q -p eatme-cli -- assets validate --json
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json

cargo test -p eatme-assets --test remaining_curriculum_gaps
cargo test -p eatme-assets --test gallery_media_import_parity_closure
cargo test -p eatme-assets --test web_platform_scenario_parity
cargo test -p eatme-assets --test curriculum_coverage_summary
```

Run the targeted web closure tests against LookingGlass:

```bash
cd <eatme-repo>
export EATME_WEB_PLATFORM=1
export ALICE_WEB_URL=http://127.0.0.1:5173

cargo test -p eatme-alice --test project_io_resource_management
cargo test -p eatme-alice --test camera_and_viewpoint_e2e
cargo test -p eatme-alice --test web_platform_curriculum_e2e -- --test-threads=1
```

Run RabbitHole baseline checks only when changing the Java baseline evidence:

```bash
cd <eatme-repo>
export EATME_REAL_ALICE=1
export ALICE_HOME=/path/to/RabbitHole

cargo run -q -p eatme-cli -- alice run-howto \
  --alice-home "$ALICE_HOME" \
  --scenario model-texture-import-checkpoint \
  --run-id parity-model-texture-import-checkpoint \
  --runs-dir runs \
  --timeout 1800 \
  --json
```

## Update the parity matrix

Edit `assets/parity/rabbithole-lookingglass-journey-matrix.yaml` only after both
LookingGlass evidence and EatMe closure tests exist.

`model-texture-import-checkpoint` uses this covered wording because the evidence
references above are enforced:

```yaml
looking_glass:
  status: covered
  source_status: "Covered by LookingGlass imported model, texture, camera/export, safe resource, and reopen persistence contract tests"
  command: EATME_WEB_PLATFORM=1 ALICE_WEB_URL=${ALICE_WEB_URL} cargo test -p eatme-alice --test web_platform_curriculum_e2e -- --test-threads=1
  expected_behavior: "Student imports model and texture resources, applies the texture, checkpoints camera/export state, reopens the A3P, and verifies project-owned resources remain available."
closure:
  required:
    - LookingGlass:test/model-texture-import-checkpoint-closure.contract.test.ts proves import, assignment, camera checkpoint, export, and reopen persistence
    - LookingGlass:test/imported-project-assets-security.contract.test.ts proves unsafe resource names are rejected and safe metadata is project-scoped
    - LookingGlass:test/model-texture-camera-joint-export-workflow.contract.test.ts proves the public workflow API, resource export package, and share fallback behavior
    - LookingGlass:test/imported-asset-project-io.test.ts proves imported resource metadata and bytes round-trip through project IO
```

Keep the audio rows partial unless the full row claim is proven by native
browser/API behavior:

```yaml
looking_glass:
  status: partial
  source_status: "Bounded metadata and fallback support"
  reason: "Audio evidence is limited to resource metadata, manifest persistence, cue timing, and playback-bridge triggers; native playback/full authoring is not claimed."
```

## Update scenarios

Update the source scenarios first:

- `assets/scenarios/eatme/media-audio-cue-storyboard.yaml`
- `assets/scenarios/eatme/audio-camera-and-export-sharecase.yaml`
- `assets/scenarios/eatme/model-texture-import-checkpoint.yaml`

Regenerate or check the Gadugi mirrors only when the repository expects generated
scenario files to be in sync:

```bash
cd <eatme-repo>
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
```

Scenario text must match the matrix boundary before generated mirrors or parity
rows are updated. Use "audio metadata", "cue timing", "playback-bridge trigger",
and "download/share fallback". Do not describe simulated audio as native
playback. Do not describe share-artifact fallback as guaranteed native Web Share
success.

## Troubleshooting

| Symptom | Fix |
| --- | --- |
| `model-texture-import-checkpoint` cannot be promoted | Confirm LookingGlass tests prove import, safe metadata, export/checkpoint inclusion, and reopen persistence. A scenario-only wording change is not enough. |
| Audio row is tempting to mark `covered` | Leave it `partial` unless native playback and full authoring are proven. Metadata, cue timing, and playback-bridge calls are bounded support. |
| Web share is unavailable in a browser test | Keep the export/download fallback evidence. Native Web Share is optional and browser-policy dependent. |
| Generated scenario mirrors drift | Update the `assets/scenarios/eatme/*.yaml` source, then run the Gadugi generation check. |
| Unsafe resource names appear in evidence | Fail the closure. Project resources must use sanitized relative identifiers and must not persist raw local filesystem paths. |

## Related documentation

- [Gallery/media/import parity API contract](../reference/gallery-media-import-parity-contract.md)
- [Gallery/media/import parity walkthrough](../tutorials/gallery-media-import-parity-walkthrough.md)
- [Alice Web parity gap scenarios](../alice-web-parity-gap-scenarios.md)
- [Import/export workflow](../import-export-workflow.md)
- [Web platform testing](../web-platform-testing.md)
