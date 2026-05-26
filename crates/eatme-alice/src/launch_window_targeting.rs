const MAIN_ALICE_WINDOW_MARKERS: &[&str] = &["org.alice.stageide.entrypoint", "org.alice.stageide"];

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum AliceWindowSearch {
    Found { window_id: String, detail: String },
    WrongAliceLikeWindow { detail: String },
    NoAliceWindow { detail: String },
}

impl AliceWindowSearch {
    pub(crate) fn detected(&self) -> bool {
        matches!(self, Self::Found { .. })
    }

    pub(crate) fn detail(&self) -> &str {
        match self {
            Self::Found { detail, .. }
            | Self::WrongAliceLikeWindow { detail }
            | Self::NoAliceWindow { detail } => detail,
        }
    }

    pub(crate) fn failure_category(&self) -> Option<&'static str> {
        match self {
            Self::Found { .. } => None,
            Self::WrongAliceLikeWindow { .. } => Some("alice_like_window_not_main"),
            Self::NoAliceWindow { .. } => Some("alice_window_not_detected"),
        }
    }
}

#[cfg(test)]
pub(crate) fn alice_window_id(window_list: &str) -> Option<String> {
    match alice_window_search(window_list) {
        AliceWindowSearch::Found { window_id, .. } => Some(window_id),
        AliceWindowSearch::WrongAliceLikeWindow { .. }
        | AliceWindowSearch::NoAliceWindow { .. } => None,
    }
}

pub(crate) fn alice_window_search(window_list: &str) -> AliceWindowSearch {
    let mut alice_like_line = None;
    for line in window_list.lines() {
        let normalized = line.to_ascii_lowercase();
        if is_main_alice_window(&normalized)
            && let Some(window_id) = window_id(line)
        {
            return AliceWindowSearch::Found {
                detail: format!("wmctrl or xwininfo identified Alice main window {window_id}"),
                window_id,
            };
        }
        if alice_like_line.is_none() && is_alice_like_window(&normalized) {
            alice_like_line = Some(line.trim().to_string());
        }
    }

    if let Some(line) = alice_like_line {
        return AliceWindowSearch::WrongAliceLikeWindow {
            detail: format!(
                "Alice-like window was present, but no Alice main Stage IDE window matched: {line}"
            ),
        };
    }

    AliceWindowSearch::NoAliceWindow {
        detail: "no Alice main window was found in wmctrl/xwininfo output".into(),
    }
}

fn is_main_alice_window(normalized: &str) -> bool {
    MAIN_ALICE_WINDOW_MARKERS
        .iter()
        .any(|marker| normalized.contains(marker))
        || (has_alice_3_main_title(normalized) && !is_known_non_main_alice_window(normalized))
}

fn has_alice_3_main_title(normalized: &str) -> bool {
    normalized.contains("\"alice 3 \"") || normalized.trim_end().ends_with(" alice 3")
}

fn is_known_non_main_alice_window(normalized: &str) -> bool {
    ["license", "agreement", "dialog", "splash", "error"]
        .iter()
        .any(|marker| normalized.contains(marker))
}

fn is_alice_like_window(normalized: &str) -> bool {
    normalized.contains("alice")
}

fn window_id(line: &str) -> Option<String> {
    line.split_whitespace()
        .find(|token| token.starts_with("0x"))
        .map(|token| token.trim_matches('"').to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_stageide_window_from_wmctrl() {
        assert_eq!(
            alice_window_id("0x001 0 host org.alice.stageide.EntryPoint Alice 3"),
            Some("0x001".into())
        );
    }

    #[test]
    fn finds_main_alice_window_from_wmctrl_title() {
        assert_eq!(
            alice_window_id("0x001 0 host sun-awt-X11-XFramePeer Alice 3"),
            Some("0x001".into())
        );
    }

    #[test]
    fn finds_main_alice_window_from_xwininfo_tree() {
        assert_eq!(
            alice_window_id(
                r#"0x600007 "Alice 3 ": ("sun-launcher-LauncherHelper$FXHelper" "sun-launcher-LauncherHelper$FXHelper")"#
            ),
            Some("0x600007".into())
        );
    }

    #[test]
    fn distinguishes_alice_like_non_main_window() {
        let search = alice_window_search(
            r#"0x60002a "License Agreement (Part 1 of 2): Alice 3": ("sun-launcher-LauncherHelper$FXHelper" "sun-launcher-LauncherHelper$FXHelper")"#,
        );

        assert_eq!(
            search.failure_category(),
            Some("alice_like_window_not_main")
        );
        assert!(search.detail().contains("Alice-like window"));
    }

    #[test]
    fn distinguishes_absent_alice_window() {
        let search = alice_window_search("0x001 0 host firefox.Firefox Firefox");

        assert_eq!(search.failure_category(), Some("alice_window_not_detected"));
        assert!(search.detail().contains("no Alice main window"));
    }

    #[test]
    fn found_window_reports_detected_with_no_failure_category() {
        let search = alice_window_search("0x001 0 host org.alice.stageide Alice 3");

        assert!(search.detected());
        assert_eq!(search.failure_category(), None);
        assert!(search.detail().contains("Alice main window 0x001"));
    }

    #[test]
    fn splash_windows_are_alice_like_but_not_treated_as_main_windows() {
        let search = alice_window_search(
            r#"0x60002a \"Alice 3 splash\": ("sun-launcher-LauncherHelper$FXHelper" "sun-launcher-LauncherHelper$FXHelper")"#,
        );

        assert!(!search.detected());
        assert_eq!(
            search.failure_category(),
            Some("alice_like_window_not_main")
        );
    }
}
