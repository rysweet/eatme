# PASS 1: atlas invents an A3P grading pipeline that the CLI does not run

- **Checklist:** data-flow inconsistency (`data-flow` × `api-contracts` × `user-journeys`)
- **Verdict:** FAIL

## Finding
The atlas shows instructor/student grading as `saved .a3p -> program.xml parser -> AST -> grading report`, but the shipped `assets grading-report` command is only a readiness preflight built from asset validation plus dependency checks.

## Evidence
- `docs/atlas/data-flow/data-flow.mmd:6-10` shows `Student .a3p file -> ZipArchive reads program.xml -> Program / Procedure / Statement AST -> grade_* pipeline -> GradingReport`.
- `docs/atlas/user-journeys/student-lesson-e2e.mmd:19-22` shows `CLI -> Parser -> Grade -> CLI` after save.
- `docs/atlas/user-journeys/instructor-grading.mmd:10-18` shows the instructor loading a saved `.a3p` and receiving grading JSON.
- `crates/eatme-cli/src/grading.rs:19-63` builds the report from `validate_assets(Path::new(&args.path))` plus `check_dependencies(runner)` and never opens a project file.
- `docs/first-lesson-grading-report.md:3-10` explicitly says the command is a readiness preflight, not a lesson grade.

## Why this is a bug
Layers 5, 6, and 8 tell a stronger story than the code implements. The current CLI has no end-to-end path from a saved student project into parser-driven grading for the first-lesson report.

## Impact
Engineers reading the atlas can believe there is already a learner-project grading pipeline when there is not. That can hide missing implementation work and distort test planning.

## Suggested fix
Either implement a real `.a3p`/`program.xml` ingestion path for the grading command, or rewrite the affected atlas diagrams to describe the current readiness-only flow.
