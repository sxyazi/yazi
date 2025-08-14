#[path = "src/args.rs"]
mod args;

use std::{env, error::Error};

use clap::CommandFactory;
use clap_complete::{Shell, generate_to};
use vergen_gitcl::{BuildBuilder, Emitter, GitclBuilder};

fn main() -> Result<(), Box<dyn Error>> {
	let manifest = env::var_os("CARGO_MANIFEST_DIR").unwrap().to_string_lossy().replace(r"\", "/");
	if env::var_os("YAZI_CRATE_BUILD").is_none()
		&& (manifest.contains("/git/checkouts/yazi-")
			|| manifest.contains("/registry/src/index.crates.io-"))
	{
		panic!(
			"Due to Cargo's limitations, the `yazi-fm` and `yazi-cli` crates on crates.io must be built with `cargo install --force yazi-build`"
		);
	}

	generate()
}

fn generate() -> Result<(), Box<dyn Error>> {
	Emitter::default()
		.add_instructions(&BuildBuilder::default().build_date(true).build()?)?
		.add_instructions(&GitclBuilder::default().commit_date(true).sha(true).build()?)?
		.emit()?;

	if env::var_os("YAZI_GEN_COMPLETIONS").is_none() {
		return Ok(());
	}

	let cmd = &mut args::Args::command();
	let bin = "ya";
	let out = "completions";

	std::fs::create_dir_all(out)?;
	for sh in [Shell::Bash, Shell::Fish, Shell::Zsh, Shell::Elvish, Shell::PowerShell] {
		generate_to(sh, cmd, bin, out)?;
	}

	generate_to(clap_complete_nushell::Nushell, cmd, bin, out)?;
	generate_to(clap_complete_fig::Fig, cmd, bin, out)?;
	Ok(())
}
