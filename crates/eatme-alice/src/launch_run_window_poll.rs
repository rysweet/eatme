// TDD red phase: stubs only — called by tests, not yet by production code.
#![allow(dead_code)]

use eatme_core::CommandRunner;
use std::time::Duration;

const POLL_DEADLINE: Duration = Duration::from_secs(10);
const POLL_INTERVAL: Duration = Duration::from_millis(500);
const WMCTRL_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum RunWindowPollResult {
    Found {
        window_id: String,
        poll_count: u32,
        elapsed: Duration,
    },
    NotFound {
        poll_count: u32,
        elapsed: Duration,
        excluded_main_id: Option<String>,
    },
}

pub(crate) fn poll_for_run_window(
    runner: &impl CommandRunner,
    display: &str,
    main_window_id: Option<&str>,
) -> RunWindowPollResult {
    poll_for_run_window_inner(
        runner,
        display,
        main_window_id,
        POLL_DEADLINE,
        POLL_INTERVAL,
    )
}

fn poll_for_run_window_inner(
    _runner: &impl CommandRunner,
    _display: &str,
    _main_window_id: Option<&str>,
    _deadline: Duration,
    _interval: Duration,
) -> RunWindowPollResult {
    todo!("implementation pending — polls wmctrl -lx every interval until deadline")
}

fn find_new_run_window(_wmctrl_output: &str, _main_window_id: Option<&str>) -> Option<String> {
    todo!("implementation pending — scans lines for alice run window excluding main id")
}

fn line_is_alice_run_window(_line: &str) -> bool {
    todo!("implementation pending — per-line heuristic matching has_run_window_evidence logic")
}

fn extract_window_id(_line: &str) -> Option<String> {
    todo!("implementation pending — extracts 0x-prefixed hex window id from wmctrl line")
}

#[cfg(test)]
mod tests {
    use super::*;
    use eatme_core::CommandOutput;
    use eatme_test_support::FakeCommandRunner;

    const SHORT_DEADLINE: Duration = Duration::from_millis(100);
    const SHORT_INTERVAL: Duration = Duration::from_millis(10);

    // ── Test 1: immediate return when wmctrl shows a new Run window ──

    #[test]
    fn finds_new_run_window_on_first_poll() {
        let runner = FakeCommandRunner::default();
        runner.push_output(wmctrl_output(
            "0x600007 0 sun-awt-X11-XFramePeer.org.alice.stageide.EntryPoint host Alice 3\n\
             0x600042 0 sun-awt-X11-XFramePeer.org.alice.stageide.EntryPoint host Run - Alice 3\n",
        ));

        let result = poll_for_run_window_inner(
            &runner,
            ":99",
            Some("0x600007"),
            SHORT_DEADLINE,
            SHORT_INTERVAL,
        );

        match result {
            RunWindowPollResult::Found {
                window_id,
                poll_count,
                ..
            } => {
                assert_eq!(window_id, "0x600042");
                assert_eq!(poll_count, 1);
            }
            other => panic!("expected Found, got {other:?}"),
        }
    }

    // ── Test 2: main window ID is excluded from results ──

    #[test]
    fn excludes_main_window_id() {
        let runner = FakeCommandRunner::default();
        // Only run-window-like line has the SAME id as the main window
        runner.push_output(wmctrl_output(
            "0x600007 0 sun-awt-X11-XFramePeer.org.alice.stageide.EntryPoint host Run - Alice 3\n",
        ));

        let result = poll_for_run_window_inner(
            &runner,
            ":99",
            Some("0x600007"),
            SHORT_DEADLINE,
            SHORT_INTERVAL,
        );

        match result {
            RunWindowPollResult::NotFound {
                excluded_main_id, ..
            } => {
                assert_eq!(excluded_main_id.as_deref(), Some("0x600007"));
            }
            other => panic!("expected NotFound, got {other:?}"),
        }
    }

    // ── Test 3: NotFound after deadline with no matching output ──

    #[test]
    fn returns_not_found_when_no_run_window_appears() {
        let runner = FakeCommandRunner::default();
        // FakeCommandRunner returns empty stdout by default once queue is exhausted,
        // which won't match any run-window heuristic.

        let result = poll_for_run_window_inner(
            &runner,
            ":99",
            Some("0x600007"),
            SHORT_DEADLINE,
            SHORT_INTERVAL,
        );

        match result {
            RunWindowPollResult::NotFound { poll_count, .. } => {
                assert!(poll_count > 0, "should have polled at least once");
            }
            other => panic!("expected NotFound, got {other:?}"),
        }
    }

    // ── Test 4: any matching run window accepted when main_id is None ──

    #[test]
    fn accepts_any_run_window_when_main_id_is_none() {
        let runner = FakeCommandRunner::default();
        runner.push_output(wmctrl_output(
            "0x600042 0 sun-awt-X11-XFramePeer.org.alice.stageide.EntryPoint host Run - Alice 3\n",
        ));

        let result =
            poll_for_run_window_inner(&runner, ":99", None, SHORT_DEADLINE, SHORT_INTERVAL);

        match result {
            RunWindowPollResult::Found { window_id, .. } => {
                assert_eq!(window_id, "0x600042");
            }
            other => panic!("expected Found, got {other:?}"),
        }
    }

    // ── Test 5: line heuristic matches org.alice + run patterns ──

    #[test]
    fn line_heuristic_matches_org_alice_run() {
        // " run" pattern (space before "run")
        assert!(line_is_alice_run_window(
            "0x600042 0 sun-awt-X11-XFramePeer.org.alice.stageide.EntryPoint host Run - Alice 3"
        ));
        // "\"run" pattern (quote before "run") — xwininfo tree format
        assert!(line_is_alice_run_window(
            r#"0x600042 "Run - Alice 3": ("org.alice.stageide" "org.alice.stageide")"#
        ));
    }

    // ── Test 6: firefox windows excluded even when matching org.alice ──

    #[test]
    fn line_heuristic_rejects_firefox() {
        assert!(!line_is_alice_run_window(
            "0x600099 0 Navigator.Firefox host org.alice Run Something Firefox"
        ));
    }

    // ── Test 7: main window title without "run" excluded ──

    #[test]
    fn line_heuristic_rejects_main_window_title() {
        assert!(!line_is_alice_run_window(
            "0x600007 0 sun-awt-X11-XFramePeer.org.alice.stageide.EntryPoint host Alice 3"
        ));
    }

    // ── Test 8: 0x-prefixed hex extraction ──

    #[test]
    fn extracts_window_id_from_wmctrl_line() {
        assert_eq!(
            extract_window_id(
                "0x600042 0 sun-awt-X11-XFramePeer.org.alice.stageide.EntryPoint host Run"
            ),
            Some("0x600042".into())
        );
        assert_eq!(extract_window_id("no hex token here"), None);
    }

    // ── Test 9: find_new_run_window scans multi-line output correctly ──

    #[test]
    fn find_new_run_window_returns_first_new_run_window_id() {
        let output = "0x600007 0 sun-awt-X11-XFramePeer.org.alice.stageide.EntryPoint host Alice 3\n\
                      0x600042 0 sun-awt-X11-XFramePeer.org.alice.stageide.EntryPoint host Run - Alice 3\n\
                      0x600099 0 Navigator.Firefox host Mozilla Firefox\n";

        let result = find_new_run_window(output, Some("0x600007"));

        assert_eq!(result, Some("0x600042".into()));
    }

    // ── Test 10: find_new_run_window returns None when all run windows match main ──

    #[test]
    fn find_new_run_window_returns_none_when_only_main_matches() {
        let output =
            "0x600007 0 sun-awt-X11-XFramePeer.org.alice.stageide.EntryPoint host Run - Alice 3\n";

        let result = find_new_run_window(output, Some("0x600007"));

        assert_eq!(result, None);
    }

    // ── helpers ──

    fn wmctrl_output(stdout: &str) -> CommandOutput {
        CommandOutput {
            command: "wmctrl -lx".into(),
            exit_status: Some(0),
            stdout: stdout.into(),
            stderr: String::new(),
        }
    }
}
