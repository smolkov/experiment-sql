use anyhow::Result;

use clap::{Parser};

#[derive(Debug, Parser)]
pub struct Cli{
	/// New todo title
	title: Vec<String>,
}


impl Cli {
	pub fn run(&self) -> Result<()> {
		println!("create new task {}",self.title.iter().map(|x|x.to_string()).collect::<String>());
		Ok(())
	}
}