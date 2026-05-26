use anyhow::Result;
use eatme_core::{CommandOutput, CommandRunner, CommandSpec};
use std::cell::RefCell;
use std::collections::VecDeque;

#[derive(Default)]
pub struct FakeCommandRunner {
    outputs: RefCell<VecDeque<CommandOutput>>,
    commands: RefCell<Vec<String>>,
}

impl FakeCommandRunner {
    pub fn push_output(&self, output: CommandOutput) {
        self.outputs.borrow_mut().push_back(output);
    }

    pub fn commands(&self) -> Vec<String> {
        self.commands.borrow().clone()
    }
}

impl CommandRunner for FakeCommandRunner {
    fn run(&self, spec: &CommandSpec) -> Result<CommandOutput> {
        self.commands.borrow_mut().push(spec.shell_display());
        Ok(self
            .outputs
            .borrow_mut()
            .pop_front()
            .unwrap_or(CommandOutput {
                command: spec.shell_display(),
                exit_status: Some(0),
                stdout: String::new(),
                stderr: String::new(),
            }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_queued_output_and_records_shell_display() {
        let runner = FakeCommandRunner::default();
        runner.push_output(CommandOutput {
            command: "queued".into(),
            exit_status: Some(7),
            stdout: "stdout".into(),
            stderr: "stderr".into(),
        });

        let output = runner
            .run(&CommandSpec::new("alice").args(["--headless", "--verify"]))
            .unwrap();

        assert_eq!(output.command, "queued");
        assert_eq!(output.exit_status, Some(7));
        assert_eq!(runner.commands(), vec!["alice --headless --verify"]);
    }

    #[test]
    fn falls_back_to_successful_empty_output_when_queue_is_empty() {
        let runner = FakeCommandRunner::default();

        let output = runner
            .run(&CommandSpec::new("java").args(["-version"]))
            .unwrap();

        assert_eq!(output.command, "java -version");
        assert_eq!(output.exit_status, Some(0));
        assert!(output.stdout.is_empty());
        assert!(output.stderr.is_empty());
        assert_eq!(runner.commands(), vec!["java -version"]);
    }

    #[test]
    fn consumes_queued_outputs_in_order_before_defaulting() {
        let runner = FakeCommandRunner::default();
        runner.push_output(CommandOutput {
            command: "first".into(),
            exit_status: Some(1),
            stdout: "one".into(),
            stderr: String::new(),
        });
        runner.push_output(CommandOutput {
            command: "second".into(),
            exit_status: Some(2),
            stdout: "two".into(),
            stderr: "warn".into(),
        });

        let first = runner
            .run(&CommandSpec::new("alice").args(["--first"]))
            .unwrap();
        let second = runner
            .run(&CommandSpec::new("alice").args(["--second"]))
            .unwrap();
        let fallback = runner
            .run(&CommandSpec::new("alice").args(["--third"]))
            .unwrap();

        assert_eq!(first.command, "first");
        assert_eq!(first.exit_status, Some(1));
        assert_eq!(second.command, "second");
        assert_eq!(second.exit_status, Some(2));
        assert_eq!(second.stderr, "warn");
        assert_eq!(fallback.command, "alice --third");
        assert_eq!(fallback.exit_status, Some(0));
        assert_eq!(
            runner.commands(),
            vec!["alice --first", "alice --second", "alice --third"]
        );
    }
}
