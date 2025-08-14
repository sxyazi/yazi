use std::{env, error::Error};

fn main() -> Result<(), Box<dyn Error>> {
	let dir = env::var("OUT_DIR").unwrap();

	// cargo build
	//   C:\Users\Ika\Desktop\yazi\target\release\build\yazi-fm-cfc94820f71daa30\out
	// cargo install
	//   C:\Users\Ika\AppData\Local\Temp\cargo-installTFU8cj\release\build\
	// yazi-fm-45dffef2500eecd0\out

	if dir.contains(r"\release\build\yazi-fm-") {
		panic!(
			"Unwinding must be enabled for Windows. Please use `cargo build --profile release-windows --locked` instead to build Yazi."
		);
	}

	let manifest = env::var_os("CARGO_MANIFEST_DIR").unwrap().to_string_lossy().replace(r"\", "/");
	if env::var_os("YAZI_CRATE_BUILD").is_none()
		&& (manifest.contains("/git/checkouts/yazi-")
			|| manifest.contains("/registry/src/index.crates.io-"))
	{
		panic!(
			"Due to Cargo's limitations, the `yazi-fm` and `yazi-cli` crates on crates.io must be built with `cargo install --force yazi-build`"
		);
	}

	Ok(())
}
