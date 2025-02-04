use std::{env, error::Error};

fn main() -> Result<(), Box<dyn Error>> {
	if env::var_os("CARGO_CFG_TARGET_FAMILY").unwrap() != "windows" {
		return Ok(());
	}
	if env::var_os("CARGO_CFG_PANIC").unwrap() == "unwind" {
		return Ok(());
	}

	panic!(
		"Unwinding must be enabled for Windows. Please use `cargo build --profile release-windows --locked` instead to build Yazi."
	);

	Ok(())
}
