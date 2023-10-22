use std::path::PathBuf;

use clap::{command, Parser};

#[derive(Debug, Parser)]
#[command(name = "yazi", version)]
pub(super) struct Args {
	/// Set the current working directory
	#[arg(index = 1)]
	pub cwd: Option<PathBuf>,

	/// Write the cwd on exit to this file
	#[arg(long)]
	pub cwd_file:     Option<PathBuf>,
	/// Write the selected files on open emitted by the chooser mode
	#[arg(long)]
	pub chooser_file: Option<PathBuf>,

	/// Clear the cache directory
	#[arg(long, action)]
	pub clear_cache: bool,
}
