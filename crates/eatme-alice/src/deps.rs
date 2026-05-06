use anyhow::Result;
use eatme_core::{CommandRunner, CommandSpec};
use serde::Serialize;
use std::collections::BTreeMap;
use std::time::Duration;

pub const REQUIRED_TOOLS: &[&str] = &[
    "java", "mvn", "git", "Xvfb", "xdpyinfo", "wmctrl", "xwininfo", "xdotool",
];

pub const OPTIONAL_TOOLS: &[&str] = &["glxinfo"];

pub const SCREENSHOT_TOOLS: &[&str] = &["scrot", "import"];

#[derive(Clone, Debug, Serialize)]
pub struct DependencyReport {
    pub tools: BTreeMap<String, bool>,
    pub screenshot_available: bool,
    pub all_required_available: bool,
}

pub fn check_dependencies(runner: &impl CommandRunner) -> Result<DependencyReport> {
    let mut tools = BTreeMap::new();
    for tool in REQUIRED_TOOLS
        .iter()
        .chain(OPTIONAL_TOOLS.iter())
        .chain(SCREENSHOT_TOOLS.iter())
    {
        tools.insert((*tool).to_string(), command_exists(runner, tool)?);
    }
    let screenshot_available = SCREENSHOT_TOOLS
        .iter()
        .any(|tool| tools.get(*tool).copied().unwrap_or(false));
    let all_required_available = REQUIRED_TOOLS
        .iter()
        .all(|tool| tools.get(*tool).copied().unwrap_or(false))
        && screenshot_available;

    Ok(DependencyReport {
        tools,
        screenshot_available,
        all_required_available,
    })
}

fn command_exists(runner: &impl CommandRunner, tool: &str) -> Result<bool> {
    let output = runner.run(
        &CommandSpec::new("sh")
            .args(["-c", &format!("command -v {tool}")])
            .timeout(Duration::from_secs(2))
            .retries(2, Duration::from_millis(100)),
    )?;
    Ok(output.exit_status == Some(0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use eatme_core::CommandOutput;
    use eatme_test_support::FakeCommandRunner;

    #[test]
    fn reports_missing_required_tool() {
        let runner = FakeCommandRunner::default();
        for tool in REQUIRED_TOOLS.iter().chain(SCREENSHOT_TOOLS.iter()) {
            runner.push_output(CommandOutput {
                command: format!("command -v {tool}"),
                exit_status: if *tool == "Xvfb" { Some(1) } else { Some(0) },
                stdout: String::new(),
                stderr: String::new(),
            });
        }

        let report = check_dependencies(&runner).unwrap();

        assert!(!report.all_required_available);
        assert_eq!(report.tools.get("Xvfb"), Some(&false));
    }
}
