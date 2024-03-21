use std::ops::Neg;

use anyhow::Result;
use clap::Subcommand;

pub mod list;
pub mod new;

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Create new task
    New(new::Cli),
	/// Show todo list
    List(list::Cli),
}

impl Command {
    pub fn run(&self) -> Result<()> {
		match self {
			Command::New(cli) => cli.run()?,
			Command::List(cli) => cli.run()?,
		}
		Ok(())
	}
}
