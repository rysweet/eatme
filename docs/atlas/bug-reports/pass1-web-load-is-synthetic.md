# PASS 1: web-platform reload coverage is synthetic, not REST-backed

- **Checklist:** web platform adapter completeness (`user-journeys` × `api-contracts`)
- **Verdict:** FAIL

## Finding
The web-platform docs claim project reload and instructor review coverage, but the test harness does not call a load endpoint. It fakes reload by replaying locally remembered save state.

## Evidence
- `docs/web-platform-testing.md:86-97` says `Project IO` will "Save, reload, and verify project state" and `Instructor grading` will "Round-trip the saved project structure for review".
- `crates/eatme-alice/tests/web_platform_curriculum_e2e.rs:196-221` sends `POST /api/project/save` for `Step::Save`, but `Step::Load` only compares `saved_path`/`saved_count` in memory and returns `load({path})` without any HTTP request.
- `docs/atlas/api-contracts/README.md:28-34` documents `POST /api/project/save` but no load/reopen endpoint.
- `docs/atlas/user-journeys/web-platform-test.mmd:10-17` documents only the save-side REST calls.

## Why this is a bug
The documented journey overclaims what the adapter verifies. Reload behavior is currently local bookkeeping inside the Rust test, not a contract exercised against the TypeScript server.

## Impact
A broken server-side load/reopen path would not be caught by this suite even though the docs suggest it is covered.

## Suggested fix
Either add and document a real load/reopen REST contract, or downgrade the docs to say that current web tests only verify save plus local post-save bookkeeping.
