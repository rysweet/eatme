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
