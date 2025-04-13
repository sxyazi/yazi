use std::{env, error::Error};

fn main() -> Result<(), Box<dyn Error>> {
	let dir = env::var("OUT_DIR").unwrap();

	// cargo build
	//   C:\Users\Ika\Desktop\yazi\target\release\build\yazi-fm-cfc94820f71daa30\out
	// cargo install
	//   C:\Users\Ika\AppData\Local\Temp\cargo-installTFU8cj\release\build\
	// yazi-fm-45dffef2500eecd0\out

	if dir.contains("\\release\\build\\yazi-fm-") {
		panic!(
			"Unwinding must be enabled for Windows. Please use `cargo build --profile release-windows --locked` instead to build Yazi."
		);
	}

	Ok(())
}
