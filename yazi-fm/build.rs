use std::{env, error::Error};

fn main() -> Result<(), Box<dyn Error>> {
	let dir = env::var("OUT_DIR").unwrap();
	if dir.contains("\\target\\release\\build\\yazi-fm-") {
		panic!(
			"Unwinding must be enabled for Windows. Please use `cargo build --profile release-windows --locked` instead to build Yazi."
		);
	}

	Ok(())
}
