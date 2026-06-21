use std::{env, error::Error};

use vergen_gitcl::{Build, Emitter, Gitcl, Rustc};

fn main() -> Result<(), Box<dyn Error>> {
	Emitter::default()
		.add_instructions(&Build::builder().build_date(true).build())?
		.add_instructions(
			&Rustc::builder().commit_date(true).commit_hash(true).host_triple(true).semver(true).build(),
		)?
		.emit()?;

	if env::var_os("YAZI_NO_GITCL").is_none() {
		Emitter::default().add_instructions(&Gitcl::builder().sha(true).build())?.emit()?;
	} else {
		println!("cargo:rustc-env=VERGEN_GIT_SHA=no-gitcl");
	}

	Ok(())
}
