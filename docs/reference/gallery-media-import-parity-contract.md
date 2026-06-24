# Gallery/media/import parity API contract

This reference defines the target LookingGlass public/API behavior and EatMe
closure contract for the gallery/media/import parity workstream.

Last updated: 2026-06-23.

## Contents

- [Status rules](#status-rules)
- [LookingGlass import/resource API](#lookingglass-importresource-api)
- [LookingGlass camera API](#lookingglass-camera-api)
- [LookingGlass export/share API](#lookingglass-exportshare-api)
- [LookingGlass audio API](#lookingglass-audio-api)
- [EatMe closure contract](#eatme-closure-contract)
- [Security contract](#security-contract)
- [Non-claims](#non-claims)

## Status rules

| Status | Meaning |
| --- | --- |
| `covered` | The RabbitHole row behavior and the exact LookingGlass behavior are both named, tested, and enforced by EatMe closure tests. |
| `partial` | LookingGlass proves a bounded subset, or a composite row includes at least one bounded subclaim. The row must state the limitation. |
| `not_supported` | The web behavior is intentionally unsupported and the matrix names why. |

`covered` is valid only when source/API tests and EatMe matrix/scenario tests
agree on the same user-visible claim. The committed parity matrix remains the
authoritative status source.

## LookingGlass import/resource API

### Imported asset creation

Module: `src/imported-project-assets.ts`

```ts
createImportedProjectAsset(upload, existingAssets?)
projectResourceIdToArchivePath(projectResourceId)
archivePathToProjectResourceId(archivePath)
applySurfaceTextureBinding(object, textureResourceId)
```

`createImportedProjectAsset` accepts:

| Field | Rule |
| --- | --- |
| `kind` | `model` or `texture`. |
| `fileName` | Basename only. No empty names, path separators, parent traversal, absolute paths, or unsupported extensions. |
| `displayName` | Optional trimmed user-facing label. Falls back to a title derived from the safe basename. |
| `bytes` | Non-empty `Uint8Array`. |

Supported model extensions are `.gltf` and `.glb`. Supported texture extensions
are `.png`, `.jpg`, `.jpeg`, and `.webp`.

Created model resources use:

```text
project/models/<safe-file-name>
resources/models/<safe-file-name>
```

Created texture resources use:

```text
project/textures/<safe-file-name>
resources/textures/<safe-file-name>
```

Duplicate imported assets are deduplicated with numeric suffixes, for example
`rocket.glb`, then `rocket-2.glb`.

### Model/texture workflow

Module: `src/model-texture-camera-joint-export-workflow.ts`

```ts
createWorkflowState({ project })
importModelAsset(state, { fileName, bytes, objectName? })
importTextureAsset(state, { fileName, bytes })
assignTextureToModel(state, { objectName, texturePath, materialName? })
setCameraWorkflowState(state, cameraWorkflow)
exportA3pArchive(state)
exportWebPackage(state, options?)
generateShareArtifacts(input)
```

The workflow contract is:

1. Imported resources are copied into workflow state as bytes plus sanitized
   archive paths.
2. `importModelAsset` records imported asset metadata and optionally assigns the
   model resource to an existing scene object.
3. `importTextureAsset` records imported texture metadata and resource bytes.
4. `assignTextureToModel` requires a previously imported texture and an existing
   object, then stores a surface texture binding using a `project/textures/...`
   resource ID.
5. `exportA3pArchive` writes project resources into the archive.
6. `exportWebPackage` includes imported resources in the package and updates the
   package hash and size after resource insertion.

### Resource manager

Module: `src/resource-manager.ts`

`createResourceManager(loader, options?)` manages `texture`, `model`, and
`audio` resources. The parity closure uses it to prove resource registration,
lazy loading, reference counting, cache eviction, and resource metadata access.

Resource lookups fail visibly for unknown keys. They must not synthesize success
for missing imported resources.

## LookingGlass camera API

Module: `src/camera-workflow.ts`

Schema version:

```text
eatme.alice-camera-workflow-state/v1
```

Public operations:

```ts
createDefaultCameraWorkflowState()
validateCameraWorkflowState(state)
moveCamera(state, input)
panCamera(state, input)
zoomCamera(state, input)
focusCamera(state, input)
orbitCamera(state, input)
applyCameraPreset(state, preset)
setCameraMode(state, mode)
saveCameraMarker(state, { name })
restoreCameraMarker(state, markerId)
deleteCameraMarker(state, markerId)
listCameraMarkers(state)
```

Supported presets are `home`, `front`, `back`, `left`, `right`, `top`, and
`isometric`. Supported modes are `orbit` and `first-person`.

Validation requires finite numeric vectors, field of view from `1` to `120`
degrees, pitch from `-89` to `89` degrees, non-empty marker names, and active
marker references that point to existing markers.

## LookingGlass export/share API

Module: `src/project-export.ts`

### Web package export

```ts
exportWebPackage(project, options?)
```

Returns:

```json
{
  "schema_version": "alice-web.export-web-package-result/v1",
  "status": "exported",
  "runtime": "alice-web",
  "package": {
    "filename": "alice-project.alice-web.zip",
    "mimeType": "application/zip",
    "sizeBytes": 12345,
    "sha256": "..."
  },
  "artifacts": {
    "entrypoint": "index.html",
    "manifest": "manifest.json",
    "share": "share.json",
    "preview": "preview.png",
    "project": "project/project.json",
    "validation": "validation.json"
  }
}
```

The package must include all required artifacts, safe ZIP paths, Alice web
identity, an entrypoint with embedded project data, and a PNG preview.

### Package validation

```ts
validateWebPackage({ packageBase64 })
```

Returns `valid: true` only when the package base64 decodes, the ZIP is readable,
required files are present once, paths are safe, identity is Alice web identity,
and the entrypoint is playable by the web runtime.

### Share artifacts

```ts
generateShareArtifacts({ packageBase64, title?, description?, canonicalUrl? })
```

Returns:

```json
{
  "schema_version": "alice-web.share-artifacts-result/v1",
  "status": "shared",
  "runtime": "alice-web",
  "artifacts": {
    "share": "share.json",
    "preview": "preview.png",
    "entrypoint": "index.html",
    "package": "alice-project.alice-web.zip"
  }
}
```

`status: "shared"` means deterministic share artifacts were produced. It does
not mean the browser completed native Web Share. UI evidence must distinguish
native share availability from export/download fallback availability.

## LookingGlass audio API

Modules: `src/audio.ts` and `src/project-audio.ts`

Supported audio extensions are:

```text
.mp3, .wav, .ogg, .m4a
```

Workflow manifest key:

```text
aliceAudio
```

Workflow manifest schema:

```text
alice-web.audio-manifest/v1
```

### Workflow-state manifest API

The `ProjectAudioWorkflowState` surface stores bounded audio metadata for
project IO and scenario evidence. It is the surface behind `aliceAudio` manifest
persistence.

```ts
createDefaultProjectAudioState()
addProjectAudioResource(state, resource)
setBackgroundAudio(state, background)
upsertAudioCue(state, cue)
removeAudioCue(state, cueId)
startAudioCue(state, cueId)
stopAudioCue(state, cueId)
serializeProjectAudioWorkflowManifest(state)
applyProjectAudioWorkflowManifest(manifest, resources)
```

The workflow-state contract proves:

1. Supported audio resource metadata is validated.
2. Audio resource paths are safe and match their declared format.
3. Cue IDs, cue names, trigger types, background settings, volume, pan, and active
   cue references are validated.
4. `aliceAudio` manifests round-trip through project IO when the referenced bytes
   exist.

### Legacy playback-bridge API

The playback bridge uses the older `ProjectAudioState` surface and is evidence
for deterministic trigger calls only:

```ts
createEmptyProjectAudioState()
registerAudioAsset(state, input)
setBackgroundMusic(state, input)
addAudioCue(state, input)
createProjectAudioPlaybackBridge(state, options)
```

The playback-bridge contract proves that configured background music and
timeline cues call the supplied output adapter with deterministic arguments. It
does not prove native browser playback or full audio authoring.

Rows that depend on native browser playback or full audio authoring remain
`partial`.

## EatMe closure contract

EatMe closure tests enforce the documentation boundary through:

| Surface | Requirement |
| --- | --- |
| `assets/parity/rabbithole-lookingglass-journey-matrix.yaml` | `model-texture-import-checkpoint`, `media-audio-cue-storyboard`, and `audio-camera-and-export-sharecase` are `covered` only when the named LookingGlass contract tests pass. |
| `assets/scenarios/eatme/*.yaml` | Source scenarios name exact evidence, non-claims, and fallback behavior before generated mirrors are updated. |
| `assets/scenarios/gadugi/*.yaml` | Generated mirrors remain in sync with source scenarios when checked in. |
| `crates/eatme-assets/tests/*` | Matrix/scenario wording rejects broad audio, native share, and unsupported media claims. |
| `crates/eatme-alice/tests/*` | Web closure tests require LookingGlass behavior through browser/API evidence. |

The closure tests reject these overclaims unless a dedicated evidence row proves
them:

- native audio playback
- full audio authoring
- native Web Share success
- deployed sharing/platform success
- arbitrary external model import
- open-asset provenance as a substitute for user import
- class-sharing parity
- broad setup/save parity outside this workstream

### Canonical evidence references

EatMe closure asserts these exact LookingGlass references for the covered
`model-texture-import-checkpoint` row:

| Evidence reference | Required for |
| --- | --- |
| `LookingGlass:test/model-texture-import-checkpoint-closure.contract.test.ts` | Import, assignment, camera checkpoint, export, and reopen persistence. |
| `LookingGlass:test/imported-project-assets-security.contract.test.ts` | Unsafe resource names are rejected and safe metadata is project-scoped. |
| `LookingGlass:test/model-texture-camera-joint-export-workflow.contract.test.ts` | Public workflow API, resource export package, and share fallback behavior. |
| `LookingGlass:test/imported-asset-project-io.test.ts` | Imported resource metadata and bytes round-trip through project IO. |

These references support the covered native audio/export/share rows:

| Evidence reference | Claim |
| --- | --- |
| `LookingGlass:test/project-audio-native-authoring.contract.test.ts` | Native Web Audio playback and audio authoring evidence. |
| `LookingGlass:test/project-export-native-web-share.contract.test.ts` | Native Web Share success only after a real matching package file is shared. |
| `LookingGlass:test/project-export-share-fallback.contract.test.ts` | Browser-download handling when native Web Share is unavailable or rejected. |

## Security contract

Imported media is untrusted input.

| Input | Required handling |
| --- | --- |
| Model and texture filenames | Trim, require supported extension, reject empty names, separators, traversal, and absolute paths. Encoded path escapes must not persist as raw or decoded separators; promotion requires direct evidence that they are rejected or normalized to a safe basename before persistence. |
| Audio resource paths | Require safe `resources/audio/...` style paths and matching supported extension. |
| Project archives | Reject unsafe ZIP paths, duplicate required files, missing resource bytes, and escaping write targets. |
| Export/share artifacts | Include only project-owned sanitized resources. Do not persist raw local filesystem paths, credentials, temporary directories, or machine-specific paths. |
| Unsupported formats | Fail visibly with a typed/domain error. Do not silently create success-shaped metadata. |

## Non-claims

This contract does not claim:

- LookingGlass replaces RabbitHole for all Alice media behavior.
- Audio metadata support is native audio playback.
- Playback-bridge calls are audible output.
- Generated share artifacts mean native Web Share succeeded.
- Open-asset pipeline provenance is the same as user import evidence.
- Model/texture import closure covers class export/import or broad gallery parity.

## Related documentation

- [Gallery/media/import parity usage guide](../howto/gallery-media-import-parity.md)
- [Gallery/media/import parity walkthrough](../tutorials/gallery-media-import-parity-walkthrough.md)
- [Evidence artifact contract](../evidence-artifact-contract.md)
- [Sharing platform readiness](../sharing-platform-readiness.md)
