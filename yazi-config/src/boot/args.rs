use std::path::PathBuf;

use clap::{command, Parser};

#[derive(Debug, Parser)]
#[command(name = "yazi")]
pub struct Args {
	/// Set the current working entry
	#[arg(index = 1)]
	pub entry: Option<PathBuf>,

	/// Write the cwd on exit to this file
	#[arg(long)]
	pub cwd_file: Option<PathBuf>,
	/// Write the selected files on open emitted by the chooser mode
	#[arg(long)]
	pub chooser_file: Option<PathBuf>,
	/// Write the selected files on open to stdout
	#[arg(long, action)]
	pub chooser_stdout: bool,

	/// Clear the cache directory
	#[arg(long, action)]
	pub clear_cache: bool,

	/// Print version
	#[arg(short = 'V', long)]
	pub version: bool,
}
