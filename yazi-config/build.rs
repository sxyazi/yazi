#[path = "src/boot/cli.rs"]
mod cli;

use std::{env, fs, io};

use clap::CommandFactory;
use clap_complete::{generate_to, Shell};

fn main() -> io::Result<()> {
	if env::var_os("YAZI_GEN_COMPLETIONS").is_none() {
		return Ok(());
	}

	let cmd = &mut cli::Args::command();
	let bin = "yazi";
	let out = "completions";

	fs::create_dir_all(out)?;
	generate_to(Shell::Bash, cmd, bin, out)?;
	generate_to(Shell::Fish, cmd, bin, out)?;
	generate_to(Shell::Zsh, cmd, bin, out)?;
	generate_to(Shell::Elvish, cmd, bin, out)?;
	generate_to(Shell::PowerShell, cmd, bin, out)?;
	generate_to(clap_complete_nushell::Nushell, cmd, bin, out)?;
	generate_to(clap_complete_fig::Fig, cmd, bin, out)?;
	Ok(())
}
