# Alice content coverage tests

The Alice content coverage tests validate the breadth of Alice features bundled
inside `.a3p` starter project archives. They complement the 8 core lesson
smoke tests by exercising starter gallery discovery, model/joint hierarchies,
camera transforms, audio references, and billboard elements — all extracted
from real `.a3p` ZIP archives without launching the Alice desktop.

## Contents

- [Usage](#usage)
- [Environment gate](#environment-gate)
- [Module layout](#module-layout)
- [Shared helper API](#shared-helper-api)
- [Test categories](#test-categories)
  - [Parser robustness (a3p_content_coverage)](#parser-robustness-a3p_parser_support)
  - [Content pattern matching (a3p_content_coverage)](#content-pattern-matching-a3p_parser_support)
  - [Round-trip (a3p_roundtrip_coverage)](#round-trip-import_export_support)
  - [Starter project gallery](#starter-project-gallery)
- [Configuration](#configuration)
- [Examples](#examples)
- [Writing new content coverage tests](#writing-new-content-coverage-tests)
- [Regex pattern reference](#regex-pattern-reference)
- [Security considerations](#security-considerations)
- [Maintenance checklist](#maintenance-checklist)
- [Explicit non-claims](#explicit-non-claims)
- [Related documentation](#related-documentation)

## Usage

Run all content coverage unit tests (always available, no Alice required):

```bash
cargo test -p eatme-alice --test a3p_content_coverage
cargo test -p eatme-alice --test a3p_roundtrip_coverage
```

Run the integration tests that scan real `.a3p` archives (requires
`EATME_REAL_ALICE=1` and a packaged Alice checkout):

```bash
EATME_REAL_ALICE=1 cargo test -p eatme-alice --test a3p_gallery_coverage
```

Run all `eatme-alice` tests (integration tests skip automatically when the
environment gate is absent):

```bash
cargo test -p eatme-alice
```

## Environment gate

| Variable | Required value | Effect |
| --- | --- | --- |
| `EATME_REAL_ALICE` | `1` | Enables integration tests that scan real `.a3p` starter projects. Any other value or absence causes those tests to skip with a message. |
| `ALICE_HOME` | Path to Alice checkout | The Alice checkout directory. Defaults to `../alice3-modernization` when not set. Used to locate the `starter-projects/` directory inside the packaged distribution. |

Unit tests in `a3p_content_coverage` and `a3p_roundtrip_coverage` use synthetic
in-memory ZIPs and always run regardless of environment variables.

The gate is a runtime `std::env::var` check via the shared `real_alice_enabled()`
helper, not a compile-time `cfg` attribute. This means `cargo test --no-run`
always compiles all tests, and `cargo test` skips gated tests gracefully.

## Module layout

All test files live in `crates/eatme-alice/tests/`:

| File | Category | Gate |
| --- | --- | --- |
| `a3p_content_support.rs` | Shared helper module | N/A (not a test binary) |
| `a3p_content_coverage.rs` | Unit tests | Always runs |
| `a3p_roundtrip_coverage.rs` | Unit tests | Always runs |
| `a3p_gallery_coverage.rs` | Integration tests | `EATME_REAL_ALICE=1` |

The shared helper module `a3p_content_support.rs` is imported by the test files
using `#[path]` or `mod` declarations. It is not a standalone test binary.

## Shared helper API

These helpers are test-only utilities in `a3p_content_support.rs`. They are
implementation details and are not part of the public crate API.

| Helper | Purpose |
| --- | --- |
| `real_alice_enabled()` | Returns `true` when `EATME_REAL_ALICE` is set to `1`. Integration tests call this at the top and return early when `false`. |
| `discover_a3p_files(dir)` | Recursively walks a directory and returns all paths ending in `.a3p`. Skips hidden directories and symlinks. |
| `extract_all_xml(path)` | Opens a `.a3p` ZIP file by path, iterates entries, collects all `.xml` file contents into a single concatenated string. Applies the path-traversal guard and 50 MB size cap. Used by integration tests. |
| `extract_all_xml_bytes(bytes)` | Same as `extract_all_xml` but accepts `&[u8]` via `Cursor`. Used by unit tests that build synthetic archives in memory. |
| `build_synthetic_a3p(entries)` | Builds an in-memory `.a3p` ZIP from a list of `(filename, content)` pairs. Returns `Vec<u8>` for passing directly to `extract_all_xml_bytes`. |
| `starter_projects_dir()` | Resolves the `starter-projects/` directory from `ALICE_HOME` or the default checkout path. |
| `JOINT_PATTERN` | Compiled regex: `JointedModel\|Joint(?:Id)?\|SkeletonVisual` |
| `BOUNDING_BOX_PATTERN` | Compiled regex: `BoundingBox\|boundingBox` |
| `CAMERA_PATTERN` | Compiled regex: `CameraMarker\|VantagePoint\|SymmetricPerspectiveCamera\|fieldOfView` |
| `AUDIO_PATTERN` | Compiled regex: `PlayAudio\|AudioSource\|\.mp3\|\.wav` |
| `BILLBOARD_PATTERN` | Compiled regex: `Billboard\|TextModel\|TextString` |
| `SCENE_ENTITY_PATTERN` | Compiled regex: `SScene\|SModel\|SGround` |
| `RESOURCE_DECL_PATTERN` | Compiled regex: `resourceReference\|ModelResourceReference` |

All regex patterns are compiled from string literals at test time using
`regex::Regex::new()`. They target Alice 3 XML element and attribute names
found inside `.a3p` project archives.

## Test categories

### Parser robustness (a3p_content_coverage)

**File:** `a3p_content_coverage.rs`

Core unit tests that validate the ZIP extraction pipeline handles edge cases
correctly. These always run — no environment gate required.

| Test | What it proves |
| --- | --- |
| `valid_extraction` | A well-formed synthetic `.a3p` with XML entries extracts correctly. |
| `empty_zip` | An empty ZIP archive produces no XML content without panicking. |
| `no_xml_zip` | A ZIP containing only non-XML entries (e.g. `.png`, `.txt`) returns empty XML. |
| `path_traversal_rejection` | Entries with `..` or absolute paths are silently skipped (security guard). |
| `oversized_content_cap` | XML concatenation exceeding the 50 MB cap is truncated, not failed. |
| `nested_directory_handling` | XML files inside nested subdirectories within the ZIP are discovered. |
| `filename_filtering` | Only `.xml` entries are extracted; other file types are ignored. |

### Content pattern matching (a3p_content_coverage)

**File:** `a3p_content_coverage.rs`

Additional unit tests that build synthetic `.a3p` archives containing specific
Alice XML element families, then verify the regex patterns match. These
complement the parser robustness tests and always run without gating.

| Test | What it proves |
| --- | --- |
| `synthetic_a3p_with_joints_extracts` | A synthetic `.a3p` containing joint XML can be extracted and matched. |
| `synthetic_a3p_with_bounding_box_extracts` | A synthetic `.a3p` containing bounding box XML can be extracted and matched. |
| `synthetic_a3p_with_resource_metadata_extracts` | A synthetic `.a3p` containing resource declarations can be extracted and matched. |
| `synthetic_a3p_with_camera_extracts` | A synthetic `.a3p` containing camera XML can be extracted and matched. |
| `synthetic_a3p_with_audio_extracts` | A synthetic `.a3p` containing audio references can be extracted and matched. |
| `synthetic_a3p_with_billboard_extracts` | A synthetic `.a3p` containing billboard XML can be extracted and matched. |
| `synthetic_a3p_with_scene_entities_extracts` | A synthetic `.a3p` containing scene entity XML (`SScene`, `SModel`) can be extracted and matched. |

### Round-trip (a3p_roundtrip_coverage)

**File:** `a3p_roundtrip_coverage.rs`

Unit tests that validate the `build_synthetic_a3p()` → `extract_all_xml_bytes()`
round-trip. These always run without gating.

| Test | What it proves |
| --- | --- |
| `round_trip_build_extract` | A synthetic `.a3p` built with `build_synthetic_a3p()` can be extracted back to the original XML content. |
| `multi_entry_zip` | A synthetic `.a3p` with multiple XML entries extracts all of them into the concatenated output. |
| `entry_ordering_stability` | Entries are extracted in a deterministic order regardless of insertion order. |

### Starter project gallery

**File:** `a3p_gallery_coverage.rs`

Integration tests that scan every `.a3p` file in the Alice starter-projects
directory. All gated behind `EATME_REAL_ALICE=1`.

| Test | What it proves |
| --- | --- |
| `gallery_is_not_empty` | At least one `.a3p` starter project exists in the distribution. |
| `every_a3p_contains_xml` | Every discovered `.a3p` archive contains at least one `.xml` entry. |
| `scene_entity_types_present` | At least one project contains `SScene`, `SModel`, or `SGround` XML elements. |
| `resource_declarations_present` | At least one project contains `resourceReference` or `ModelResourceReference` patterns. |
| `joint_hierarchy_in_gallery` | At least one project contains joint/skeleton XML patterns. |
| `bounding_box_in_gallery` | At least one project contains bounding box XML patterns. |
| `camera_controls_in_gallery` | At least one project contains camera transform XML patterns. |
| `audio_references_in_gallery` | At least one project contains audio resource references. |
| `billboard_elements_in_gallery` | At least one project contains billboard/text overlay XML patterns. |

These tests use **aggregate assertions** — they assert "at least N projects
across the full gallery contain element X" rather than requiring every project
to contain every element. This accommodates the variety of Alice starter
projects, where not every project uses cameras, audio, or billboards.

## Configuration

The content coverage tests add two dev-dependencies to `crates/eatme-alice/Cargo.toml`:

```toml
[dev-dependencies]
zip = "2"
regex = "1"
```

No runtime dependencies are added. No changes to shared library modules are
required.

| Setting | Required | Purpose |
| --- | --- | --- |
| `EATME_REAL_ALICE=1` | Integration tests only | Enables scanning of real `.a3p` starter projects. |
| `ALICE_HOME` | Integration tests only | Locates the Alice checkout. Defaults to `../alice3-modernization`. |
| `NODE_OPTIONS=--max-old-space-size=32768` | No | Safe to export for repository-wide workflows. Not required by these Rust tests. |

## Examples

### Run only unit tests (no Alice needed)

```bash
cargo test -p eatme-alice --test a3p_content_coverage
cargo test -p eatme-alice --test a3p_roundtrip_coverage
```

### Run gallery integration tests

```bash
export ALICE_HOME=../alice3-modernization
EATME_REAL_ALICE=1 cargo test -p eatme-alice --test a3p_gallery_coverage
```

### Run a single gallery test

```bash
EATME_REAL_ALICE=1 cargo test -p eatme-alice --test a3p_gallery_coverage \
  camera_controls_in_gallery
```

### Verify compilation without running tests

```bash
cargo test -p eatme-alice --no-run
```

### Run all eatme-alice tests with the integration gate enabled

```bash
EATME_REAL_ALICE=1 cargo test -p eatme-alice
```

### Check that unit tests pass in CI (no real Alice)

```bash
cargo test -p eatme-alice --test a3p_content_coverage --test a3p_roundtrip_coverage
```

Unit tests build synthetic `.a3p` archives in memory and validate extraction,
so they run on any machine with a Rust toolchain.

## Writing new content coverage tests

Use this workflow when adding a new Alice content pattern test.

1. **Choose the right file:**

   | Test type | File |
   | --- | --- |
   | Unit test with synthetic ZIP | `a3p_content_coverage.rs` |
   | Round-trip build→extract test | `a3p_roundtrip_coverage.rs` |
   | Integration test scanning real `.a3p` files | `a3p_gallery_coverage.rs` |

2. **Add a regex pattern** to `a3p_content_support.rs` if the new test targets
   a new XML element family. Use a compile-time string literal to avoid ReDoS.

3. **Use aggregate assertions** for integration tests. Assert that at least one
   project in the gallery matches, not that every project matches. Use the
   shared `GALLERY_CACHE` to avoid re-extracting ZIPs:

   ```rust
   assert!(
       GALLERY_CACHE
           .iter()
           .any(|(_, xml)| NEW_PATTERN.is_match(xml)),
       "expected at least one .a3p to contain new-element XML"
   );
   ```

4. **Write a companion unit test** with `build_synthetic_a3p()` so the pattern
   is validated even without real Alice:

   ```rust
   #[test]
   fn synthetic_a3p_with_new_element_extracts() {
       let xml = r#"<entry><NewElement attr="value"/></entry>"#;
       let zip_bytes = build_synthetic_a3p(vec![("project.xml", xml)]);
       let extracted = extract_all_xml_bytes(&zip_bytes);
       assert!(NEW_PATTERN.is_match(&extracted));
   }
   ```

5. **Gate integration tests** with `real_alice_enabled()`:

   ```rust
   #[test]
   fn new_element_in_gallery() {
       if !real_alice_enabled() {
           eprintln!("skipping: EATME_REAL_ALICE not set");
           return;
       }
       // ...
   }
   ```

6. **Keep each file under 500 lines** to satisfy the repository module-size gate.

7. **Run the focused tests and clippy:**

   ```bash
   cargo test -p eatme-alice --test a3p_content_coverage
   cargo test -p eatme-alice --test a3p_roundtrip_coverage
   EATME_REAL_ALICE=1 cargo test -p eatme-alice --test a3p_gallery_coverage
   cargo clippy -p eatme-alice -- -D warnings
   ```

## Regex pattern reference

All patterns target Alice 3 XML element and attribute names inside `.a3p`
archives. Patterns use alternation (`|`) for related elements.

| Pattern name | Regex | Matches |
| --- | --- | --- |
| `JOINT_PATTERN` | `JointedModel\|Joint(?:Id)?\|SkeletonVisual` | Character and prop skeletal joint hierarchies. |
| `BOUNDING_BOX_PATTERN` | `BoundingBox\|boundingBox` | Object spatial bounds used for collision and layout. |
| `CAMERA_PATTERN` | `CameraMarker\|VantagePoint\|SymmetricPerspectiveCamera\|fieldOfView` | Camera placement, orbit, and perspective settings. |
| `AUDIO_PATTERN` | `PlayAudio\|AudioSource\|\.mp3\|\.wav` | Audio playback actions and sound resource files. |
| `BILLBOARD_PATTERN` | `Billboard\|TextModel\|TextString` | 2D text overlays and billboard rendering elements. |
| `SCENE_ENTITY_PATTERN` | `SScene\|SModel\|SGround` | Top-level scene structure: scene root, models, ground plane. |
| `RESOURCE_DECL_PATTERN` | `resourceReference\|ModelResourceReference` | Resource declarations linking models to gallery entries. |

Patterns are intentionally broad to accommodate variations across Alice 3 XML
format versions. Aggregate assertions (at least one match across the gallery)
prevent false negatives when a specific pattern is absent from one project
but present in another.

## Security considerations

| Concern | Mitigation |
| --- | --- |
| ZIP path traversal | `extract_all_xml()` skips entries containing `..` or beginning with `/`. Entries with path-traversal components are silently dropped. |
| Oversized archives | XML concatenation is capped at 50 MB. Archives exceeding the cap are truncated with a warning, not failed. |
| Temp file cleanup | Unit tests use in-memory `Cursor` via `extract_all_xml_bytes` — no temp files. Integration test work directories follow the existing `TestFixture` RAII cleanup convention. |
| No unsafe code | All new test code is safe Rust. No `unsafe` blocks. |
| No ReDoS risk | All regex patterns are statically defined string literals with bounded alternation. No user-supplied input reaches the regex engine. |

## Maintenance checklist

Before merging a change that touches content coverage tests:

| Check | Command |
| --- | --- |
| Format Rust files | `cargo fmt --check` |
| Run unit tests | `cargo test -p eatme-alice --test a3p_content_coverage --test a3p_roundtrip_coverage` |
| Run integration tests | `EATME_REAL_ALICE=1 cargo test -p eatme-alice --test a3p_gallery_coverage` |
| Run all eatme-alice tests | `cargo test -p eatme-alice` |
| Clippy lint | `cargo clippy -p eatme-alice -- -D warnings` |
| Enforce module size | `find crates -name '*.rs' -not -path '*/target/*' -exec wc -l {} + \| awk '$2 != "total" && $1 > 500 { print; bad=1 } END { exit bad }'` |
| Full quality gate | `./scripts/quality-gates.sh` |

## Explicit non-claims

The content coverage tests prove that `.a3p` archives contain expected XML
structure. They do not prove:

- real Alice desktop rendering or visual correctness
- full UI automation or lesson completion
- creative assessment or learner-world grading
- model rendering fidelity, texture loading, or animation playback
- audio playback or camera movement at runtime
- production readiness or deployment suitability
- save/reopen/export workflow completion

The tests are structural XML validation. They confirm that bundled starter
projects contain the expected element families. Runtime behavior of those
elements requires real Alice desktop execution, which is the domain of the
existing launch-smoke integration tests documented in
[Deterministic Real-Alice Smoke Test](deterministic-real-alice-smoke-test.md).

## Related documentation

- [Alice Integration](alice-integration.md) — real Alice checkout, packaging, launch
- [Deterministic Real-Alice Smoke Test](deterministic-real-alice-smoke-test.md) — launch-smoke integration test
- [Outside-in Alice Test Modules](outside-in-alice-test-modules.md) — asset contract tests
- [Starter Project Preflight Evidence](starter-project-preflight-evidence.md) — starter project launch readiness
- [Alice Lesson Smoke](alice-lesson-smoke.md) — lesson scenario roster
- [Validation and Quality Gates](validation-quality-gates.md) — repository quality checks
