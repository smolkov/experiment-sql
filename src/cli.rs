
use clap::{Parser, Subcommand};
use crate::command::Command;


#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Command to execute on destination host
    #[command(subcommand)]
    pub command: Command,
}

