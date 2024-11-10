#[path = "src/args.rs"]
mod args;

use std::{env, error::Error};

use clap::CommandFactory;
use clap_complete::{Shell, generate_to};
use vergen_gitcl::{BuildBuilder, Emitter, GitclBuilder, RustcBuilder};

fn main() -> Result<(), Box<dyn Error>> {
	Emitter::default()
		.add_instructions(&BuildBuilder::default().build_date(true).build()?)?
		.add_instructions(
			&RustcBuilder::default()
				.commit_date(true)
				.commit_hash(true)
				.host_triple(true)
				.semver(true)
				.build()?,
		)?
		.add_instructions(&GitclBuilder::default().commit_date(true).sha(true).build()?)?
		.emit()?;

	if env::var_os("YAZI_GEN_COMPLETIONS").is_none() {
		return Ok(());
	}

	let cmd = &mut args::Args::command();
	let bin = "yazi";
	let out = "completions";

	std::fs::create_dir_all(out)?;
	for sh in [Shell::Bash, Shell::Fish, Shell::Zsh, Shell::Elvish, Shell::PowerShell] {
		generate_to(sh, cmd, bin, out)?;
	}

	generate_to(clap_complete_nushell::Nushell, cmd, bin, out)?;
	generate_to(clap_complete_fig::Fig, cmd, bin, out)?;

	Ok(())
}
