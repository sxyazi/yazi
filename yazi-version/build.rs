use std::{env, error::Error};

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
		.emit()?;

	if env::var_os("YAZI_NO_GITCL").is_none() {
		Emitter::default().add_instructions(&GitclBuilder::default().sha(true).build()?)?.emit()?;
	} else {
		println!("cargo:rustc-env=VERGEN_GIT_SHA=no-gitcl");
	}

	Ok(())
}
