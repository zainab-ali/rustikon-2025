#![allow(dead_code)]

use futures_timer::Delay;
use std::io::{Error, ErrorKind};
use std::process::Child;
use std::process::Command as ProcCommand;
use std::process::ExitStatus;
use std::time::Duration;
// What can we acheive with only async programming?
// Can we put futures on the heap without pinning them?
// How can we run them?
// How is cancellation handled?
// Can we use a different runtime?

// This function makes a systemd unit that has not been run

// A definition that has not yet been run
#[derive(Debug, Clone)]
pub struct Command {
    pub prog: String,
    pub args: Vec<String>,
    pub delay: u64,
}

impl Command {
    pub async fn run(self) -> std::io::Result<()> {
        let delay = self.delay;
        let mut proc: ProcCommand = self.into();
        let () = Delay::new(Duration::from_secs(delay)).await;
        let status = proc.status().unwrap();
        if status.success() {
            Ok(())
        } else {
            Err(Error::new(ErrorKind::Other, "Oh no!"))
        }
    }
}

impl Into<ProcCommand> for Command {
    fn into(self) -> ProcCommand {
        let mut command = ProcCommand::new(self.prog);
        command.args(self.args);
        command
    }
}
