use std::{env, error::Error};

use vergen_gitcl::{Emitter, GitclBuilder};

fn main() -> Result<(), Box<dyn Error>> {
	if env::var_os("YAZI_NO_GITCL").is_none() {
		Emitter::default().add_instructions(&GitclBuilder::default().sha(true).build()?)?.emit()?;
	} else {
		println!("cargo:rustc-env=VERGEN_GIT_SHA=no-gitcl");
	}

	Ok(())
}
