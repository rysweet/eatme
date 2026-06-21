# Alice Web Parity Gap Scenarios

These scenarios make Alice Java-to-web parity review explicit. They are not a
replacement-readiness claim. They are gates for finding gaps and proving closure.

## Scenario set

| Scenario | Gap family | Closure probes |
| --- | --- | --- |
| `alice-web-a3p-save-load-parity` | `.a3p` parse/save/reopen, project identity, resources, statement round-trip, archive safety | `a3p_content_coverage`, `a3p_roundtrip_coverage`, `real_a3p_pipeline_integration`, `malformed_input_resilience` |
| `alice-web-story-api-runtime-parity` | procedures, parameters, functions, loops, conditionals, events, collision/proximity, text/speech | `parameters_e2e`, `functions_e2e`, `loops_and_conditionals_e2e`, `nested_control_flow_e2e`, `events_collision_support`, `events_and_collision_e2e`, `text_and_speech_e2e` |
| `alice-web-gallery-media-parity` | starter gallery, model resources, camera/viewpoint, audio/text media, import/export | `a3p_content_coverage`, `camera_and_viewpoint_e2e`, `text_and_speech_e2e`, `import_export_support`, `project_io_resource_management` |

Each scenario writes a gap matrix under `runs/<scenario>/<RUN_ID>/`. A gap is
closed only when the matrix row has both:

1. a named Java Alice baseline behavior, and
2. a passing closure test or real Alice evidence artifact.

## Run the deterministic parts

```bash
cargo run -q -p eatme-cli -- assets validate --json
cargo run -q -p eatme-cli -- assets generate-gadugi --check --json

cargo test -p eatme-alice --test a3p_content_coverage
cargo test -p eatme-alice --test a3p_roundtrip_coverage
cargo test -p eatme-alice --test real_a3p_pipeline_integration
cargo test -p eatme-alice --test malformed_input_resilience

cargo test -p eatme-alice --test parameters_e2e
cargo test -p eatme-alice --test functions_e2e
cargo test -p eatme-alice --test loops_and_conditionals_e2e
cargo test -p eatme-alice --test nested_control_flow_e2e
cargo test -p eatme-alice --test events_collision_support
cargo test -p eatme-alice --test events_and_collision_e2e
cargo test -p eatme-alice --test text_and_speech_e2e

cargo test -p eatme-alice --test camera_and_viewpoint_e2e
cargo test -p eatme-alice --test import_export_support
cargo test -p eatme-alice --test project_io_resource_management
```

## Run with real Alice evidence

```bash
export ALICE_HOME=/path/to/RabbitHole
export EATME_REAL_ALICE=1

cargo run -q -p eatme-cli -- alice launch-smoke \
  --alice-home "$ALICE_HOME" \
  --scenario alice-web-a3p-save-load-parity \
  --run-id local-a3p-parity \
  --runs-dir runs \
  --timeout 900 \
  --json \
  --no-memory \
  --offline-package
```

Repeat with:

- `alice-web-story-api-runtime-parity`
- `alice-web-gallery-media-parity`

## Boundary

These scenarios do not prove that the web port can replace Java Alice. They
prove whether named gap families have enough evidence to be closed.
