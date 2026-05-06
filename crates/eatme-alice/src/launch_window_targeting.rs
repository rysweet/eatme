const ALICE_WINDOW_MARKERS: &[&str] = &[
    "org.alice.stageide.entrypoint",
    "org.alice.stageide",
    "org.alice.ide",
    "\"alice 3",
];

pub(crate) fn alice_window_id(window_list: &str) -> Option<String> {
    window_list.lines().find_map(|line| {
        let normalized = line.to_ascii_lowercase();
        if !ALICE_WINDOW_MARKERS
            .iter()
            .any(|marker| normalized.contains(marker))
        {
            return None;
        }
        line.split_whitespace()
            .next()
            .filter(|id| id.starts_with("0x"))
            .map(str::to_string)
    })
}
