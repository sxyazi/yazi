use std::error::Error;

use vergen_gitcl::{Emitter, GitclBuilder};

fn main() -> Result<(), Box<dyn Error>> {
	Emitter::default()
		.add_instructions(&GitclBuilder::default().commit_date(true).sha(true).build()?)?
		.emit()?;

	Ok(())
}
