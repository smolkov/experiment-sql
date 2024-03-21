use anyhow::Result;
use clap::{Parser};
use todo::cli::Args;


fn main( ) -> Result<()> {

	let cli = Args::parse();

	cli.command.run()?;

	Ok(())
}