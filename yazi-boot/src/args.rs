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
	pub cwd_file:     Option<PathBuf>,
	/// Write the selected files to this file on open fired
	#[arg(long)]
	pub chooser_file: Option<PathBuf>,

	/// Clear the cache directory
	#[arg(long, action)]
	pub clear_cache: bool,

	/// Report the specified local events to stdout
	#[arg(long, action)]
	pub local_events:  Option<String>,
	/// Report the specified remote events to stdout
	#[arg(long, action)]
	pub remote_events: Option<String>,

	/// Print debug information
	#[arg(long, action)]
	pub debug: bool,

	/// Print version
	#[arg(short = 'V', long)]
	pub version: bool,
}
