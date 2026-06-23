# Gallery/media/import parity walkthrough

This tutorial walks through the covered model/texture import closure path and the
bounded audio/camera/export evidence path enforced by the named LookingGlass and
EatMe tests.

Last updated: 2026-06-23.

## What you will do

You will:

1. Import a safe model and texture into LookingGlass.
2. Bind the texture to a scene object.
3. Save/checkpoint, export, validate, and reopen the project.
4. Record bounded audio cue metadata without claiming native playback.
5. Verify camera and share fallback evidence.
6. Confirm the EatMe matrix status matches the proven behavior.

## Before you start

Set the Node memory option:

```bash
export NODE_OPTIONS=--max-old-space-size=32768
```

Use a running LookingGlass server for browser/API checks:

```bash
cd <lookingglass-repo>
npm run build:server
node dist-server/cli.js serve --port 3099 --evidence-dir ./evidence --api-token gadugi-local-api-token
```

In another terminal, point EatMe at that server:

```bash
cd <eatme-repo>
export EATME_WEB_PLATFORM=1
export ALICE_WEB_URL=http://127.0.0.1:3099
export ALICE_LOCAL_API_TOKEN=gadugi-local-api-token
```

## 1. Import model and texture resources

Use only supported file types:

| Resource | Supported extensions |
| --- | --- |
| Model | `.glb`, `.gltf` |
| Texture | `.png`, `.jpg`, `.jpeg`, `.webp` |

Illustrative API snippet. Use the executable fixtures in
`test/model-texture-camera-joint-export-workflow.contract.test.ts` and
`e2e/import-model-texture-workflow.spec.ts` for closure evidence.

```ts
import {
  assignTextureToModel,
  createWorkflowState,
  exportA3pArchive,
  exportWebPackage,
  importModelAsset,
  importTextureAsset,
} from "./src/model-texture-camera-joint-export-workflow.js";

const state0 = createWorkflowState({ project });
const state1 = await importModelAsset(state0, {
  fileName: "student-rocket.glb",
  bytes: rocketBytes,
  objectName: "Rocket",
});
const state2 = await importTextureAsset(state1, {
  fileName: "rocket-surface.png",
  bytes: textureBytes,
});
const state3 = assignTextureToModel(state2, {
  objectName: "Rocket",
  texturePath: "resources/textures/rocket-surface.png",
});

const a3pBytes = await exportA3pArchive(state3);
const webPackage = await exportWebPackage(state3, {
  title: "Rocket texture checkpoint",
});
```

The target behavior stores imported resources under sanitized project-owned
paths:

```text
resources/models/student-rocket.glb
resources/textures/rocket-surface.png
```

The project metadata stores project resource identifiers:

```text
project/models/student-rocket.glb
project/textures/rocket-surface.png
```

## 2. Reject unsafe imports

The import API rejects unsafe or unsupported examples:

| Input | Result |
| --- | --- |
| `../rocket.glb` | Rejected as traversal. |
| `/tmp/rocket.glb` | Rejected as path input instead of basename. |
| `scene/rocket.glb` | Rejected because separators are not allowed. |
| `scene%2frocket.glb` | Must not persist as a path escape; closure must prove rejection or safe basename normalization. |
| `rocket.exe` | Rejected as unsupported model extension. |
| `texture.svg` | Rejected as unsupported texture extension. |
| Empty bytes | Rejected because the import would create a false resource. |

Use a built-in fallback asset in the lesson when a third-party asset is unsafe,
unsupported, missing permission, too large, or visually unsuitable.

## 3. Prove checkpoint, export, and reopen persistence

Run the LookingGlass tests that cover the import path:

```bash
cd <lookingglass-repo>
export NODE_OPTIONS=--max-old-space-size=32768

npm run test -- \
  test/imported-project-assets-security.contract.test.ts \
  test/model-texture-import-checkpoint-closure.contract.test.ts \
  test/imported-project-assets.test.ts \
  test/imported-asset-project-io.test.ts \
  test/model-texture-camera-joint-export-workflow.contract.test.ts \
  test/project-export.test.ts
npm run test:e2e -- e2e/import-model-texture-workflow.spec.ts
```

The proof is complete when tests show:

- the imported model and texture metadata are present;
- resource bytes are written to `.a3p` and web package artifacts;
- texture bindings still point at project-owned texture IDs;
- reopened project state still includes the imported asset metadata and resource
  bytes;
- package validation reports safe ZIP paths and required package files.

That is enough to keep `model-texture-import-checkpoint` as a LookingGlass
`covered` row because the cited tests are present on LookingGlass `main`.

## 4. Record bounded audio storyboard evidence

Use audio workflow-state metadata and cue timing for storyboard evidence. This
snippet is illustrative; executable evidence lives in
`test/project-audio-contract.test.ts` and
`test/project-audio-project-io-contract.test.ts`.

```ts
import {
  addProjectAudioResource,
  createDefaultProjectAudioState,
  serializeProjectAudioWorkflowManifest,
  upsertAudioCue,
} from "./src/project-audio.js";

const audio0 = createDefaultProjectAudioState();
const audio1 = addProjectAudioResource(audio0, {
  id: "cue-ding",
  name: "Ding cue",
  path: "resources/audio/cue-ding.wav",
  format: "wav",
  sizeBytes: dingBytes.byteLength,
  duration: 0.8,
  decodeStatus: "decode-unavailable",
});
const audio2 = upsertAudioCue(audio1, {
  id: "rocket-launch-cue",
  name: "Rocket launch cue",
  resourceId: "cue-ding",
  trigger: "worldRun",
  loop: false,
  volume: 0.8,
  pan: 0,
});
const manifest = serializeProjectAudioWorkflowManifest(audio2);
```

This proves cue metadata and manifest persistence. It does not prove that the
browser produced audible sound.

Playback-bridge trigger evidence is separate. It uses the legacy
`ProjectAudioState` surface tested by
`test/audio-workflow-parity.test.ts::Alice audio workflow playback bridge`.
Scenario and matrix wording must say "audio metadata", "cue timing", "manifest
persistence", or "playback-bridge trigger"; it must not say "native audio
playback" unless that behavior has separate tests.

## 5. Verify camera and share fallback behavior

Use camera workflow state for viewpoint evidence. This snippet is illustrative;
the executable contract is `test/camera-workflow.test.ts`.

```ts
import {
  applyCameraPreset,
  createDefaultCameraWorkflowState,
  saveCameraMarker,
} from "./src/camera-workflow.js";

const camera0 = createDefaultCameraWorkflowState();
const camera1 = applyCameraPreset(camera0, "isometric");
const camera2 = saveCameraMarker(camera1, { name: "Sharecase overview" });
```

Use web package export and share-artifact generation for sharing evidence. This
snippet is illustrative; executable evidence lives in `test/project-export.test.ts`
and `e2e/alice-evidence-workflow.spec.ts`.

```ts
import {
  exportWebPackage,
  generateShareArtifacts,
  validateWebPackage,
} from "./src/project-export.js";

const exported = await exportWebPackage(project, {
  title: "Sharecase overview",
  description: "Camera marker plus bounded audio cue metadata.",
});
const validation = await validateWebPackage({
  packageBase64: exported.package.base64,
});
const shareArtifacts = await generateShareArtifacts({
  packageBase64: exported.package.base64,
  title: "Sharecase overview",
});
```

This proves a valid export package and deterministic share artifacts. It does not
prove native Web Share succeeded. If `navigator.share` is unavailable, the UI
keeps export/download available and records browser-download share evidence.

## 6. Confirm EatMe closure

Run the EatMe checks:

```bash
cd <eatme-repo>

cargo run -q -p eatme-cli -- assets validate --json
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json
cargo test -p eatme-assets --test remaining_curriculum_gaps
cargo test -p eatme-assets --test gallery_media_import_parity_closure
cargo test -p eatme-assets --test web_platform_scenario_parity

EATME_WEB_PLATFORM=1 ALICE_WEB_URL="${ALICE_WEB_URL:-http://localhost:3099}" \
  cargo test -p eatme-assets --test gallery_media_import_parity_closure

cd "${LOOKINGGLASS_HOME:?}"
EATME_WEB_PLATFORM=1 ALICE_WEB_URL="${ALICE_WEB_URL:-http://localhost:3099}" npm test -- \
  test/model-texture-import-checkpoint-closure.contract.test.ts \
  test/imported-project-assets-security.contract.test.ts \
  test/imported-asset-project-io.test.ts \
  test/model-texture-camera-joint-export-workflow.contract.test.ts \
  test/project-audio-bounded-evidence.contract.test.ts \
  test/project-export-share-fallback.contract.test.ts
```

The matrix state enforced by the closure tests is:

| Row | LookingGlass status |
| --- | --- |
| `model-texture-import-checkpoint` | `covered` |
| `media-audio-cue-storyboard` | `partial` |
| `audio-camera-and-export-sharecase` | `partial` |

## Related documentation

- [Gallery/media/import parity usage guide](../howto/gallery-media-import-parity.md)
- [Gallery/media/import parity API contract](../reference/gallery-media-import-parity-contract.md)
- [Alice Web parity gap scenarios](../alice-web-parity-gap-scenarios.md)
