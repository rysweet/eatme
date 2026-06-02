# PASS 1: `cli-usage.md` contradicts the grading command's six-step contract

- **Checklist:** stale documentation (docs spot-check)
- **Verdict:** FAIL

## Finding
`docs/cli-usage.md` says `assets grading-report` evaluates only three steps, but the detailed grading docs and the implementation both describe six steps.

## Evidence
- `docs/cli-usage.md:96-99` says the report evaluates only `validate-assets`, `check-dependencies`, and `launch-smoke`.
- `docs/first-lesson-grading-report.md:3-7` says the command checks launch-smoke preconditions plus three deeper lesson interaction steps.
- `docs/first-lesson-grading-report.md:35-49` enumerates six steps: the three preconditions plus `place-object`, `edit-code`, and `run-world`.
- `crates/eatme-assets/src/grading_report.rs:76-107` hardcodes those three interaction steps into the returned `GradingReport`.

## Why this is a bug
Two docs pages disagree on the same CLI contract, and the shorter page is the one most likely to be used as a quick reference.

## Impact
Readers can misread `assets grading-report` as a pure three-step preflight and miss the blocked/not-yet-tested interaction stages that appear in the real JSON.

## Suggested fix
Update `docs/cli-usage.md` so its summary matches the six-step contract and links cleanly to the detailed grading-report page.
