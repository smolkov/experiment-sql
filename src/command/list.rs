use anyhow::Result;

use clap::{Parser};

#[derive(Debug, Parser)]
pub struct Cli{
	/// Offset
 	offset:Option<usize>,
	/// Limit
	limit: Option<usize>,
}


impl Cli {
	pub fn run(&self) -> Result<()> {
		println!("show full list of todo's");
		Ok(())
	}
}