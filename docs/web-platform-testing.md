# LookingGlass web-platform testing

Eatme can run the Alice curriculum against LookingGlass, the TypeScript web
port, as well as the Java desktop app.

## What this covers

The web-platform lane checks the same student-style workflows that matter in
Alice lessons: open a project, add objects, edit code, run the world, save,
and verify event-driven behavior.

The test file is:

```text
crates/eatme-alice/tests/web_platform_curriculum_e2e.rs
```

It has two layers:

- **offline checks** that always run and validate scenario shape
- **live REST API tests** gated behind `EATME_WEB_PLATFORM=1`

## Start LookingGlass

In the `alice-web-prototype` repository:

```bash
npm install
npm run build:server
node dist-server/cli.js serve --port 3099 --evidence-dir ./evidence
```

Eatme uses `http://localhost:3099` by default, so that command works without
extra environment variables.

For the dedicated save/reopen/export parity check, start LookingGlass with the
starter project path so the starter-project row proves an actual `.a3p` open:

```bash
node dist-server/cli.js serve --port 3099 --evidence-dir ./evidence \
  --project ../eatme/crates/eatme-alice/tests/fixtures/real/africaMinimum.a3p
```

Check the server before you run live tests:

```bash
curl http://127.0.0.1:3099/api/health
```

The health response includes `"runtime": "lookingglass"`.

If you need a different port, point eatme at it:

```bash
export ALICE_WEB_URL=http://127.0.0.1:3000
```

## Run live web-platform tests

Run the dedicated curriculum file:

```bash
EATME_WEB_PLATFORM=1 cargo test -p eatme-alice \
  --test web_platform_curriculum_e2e \
  -- --test-threads=1
```

Run the dedicated save/reopen/export parity file:

```bash
EATME_WEB_PLATFORM=1 cargo test -p eatme-alice \
  --test web_platform_save_reopen_export_e2e \
  -- --test-threads=1
```

Run the whole workspace with the web gate enabled:

```bash
EATME_WEB_PLATFORM=1 cargo test --workspace
```

Validate the RabbitHole-vs-LookingGlass journey matrix:

```bash
cargo test -p eatme-assets --test rabbithole_lookingglass_parity_matrix
```

Without `EATME_WEB_PLATFORM=1`, the live web tests skip cleanly and the
offline checks still run.

## Scenario coverage

The live lane covers these curriculum areas:

| Area | Representative workflow |
| --- | --- |
| Hello world | Create a project, add an object, run, save |
| Procedures | Edit a procedure and run it |
| Events and collision | Register handlers and fire events |
| Loops and conditionals | Build control-flow edits and run them |
| Functions | Add function-style behavior |
| Variables | Declare, update, and reuse data |
| Concurrency | Exercise `doTogether`-style behavior |
| Arrays | Work with collection-driven steps |
| Camera and viewpoint | Check camera actions and scene framing |
| Audio | Trigger sound-related flows |
| Parameters | Edit parameterized behavior |
| Inheritance and OOP | Use custom type patterns |
| Comments and code clarity | Check learner-friendly procedure edits |
| Project IO | Save, then verify synthetic in-memory reload state; there is no REST load endpoint yet |
| Game and narrative | Follow score, win, and story flows |
| Say and think | Exercise speech and thought bubbles |
| Design process | Track plan, build, playtest, and revision checkpoints |
| Vehicle parenting | Attach one object to another |
| Joint manipulation | Move named joints on character rigs |
| Scene transition | Switch between scenes |
| Property animation | Change opacity, color, and related properties |
| Nested control flow | Layer loops and branches together |
| Full student journey | Build, run, and save one lesson path |
| Instructor grading | Preserve save-path evidence only; no REST-backed reload/review round-trip |
| Error recovery | Prove expected failures and recovery paths |
| Full curriculum sweep | Run the combined scenario set |

Save/reopen scenarios must call a supported save and reload or review API before
they count as LookingGlass user-journey coverage. In-memory comparisons alone do
not count as coverage; those flows stay marked as not supported until the API
can prove the user result.

## Environment variables

| Variable | Default | Meaning |
| --- | --- | --- |
| `EATME_WEB_PLATFORM` | unset | Set to `1` to enable live web-platform tests |
| `ALICE_WEB_URL` | `http://localhost:3099` | Base URL for LookingGlass |

## What a passing run means

A passing live run proves that LookingGlass accepts the expected REST API calls
and returns curriculum-friendly results for the covered workflows.

It does **not** mean the full browser UI has been tested by a real student, and
it does **not** replace the desktop user-journey or grading lanes.
