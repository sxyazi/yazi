use std::env;

fn main() {
	println!("cargo:rerun-if-changed=build.rs");

	check();
}

fn check() {
	let manifest = env::var_os("CARGO_MANIFEST_DIR").unwrap().to_string_lossy().replace(r"\", "/");
	if manifest.contains("/git/checkouts/yazi-")
		|| manifest.contains("/registry/src/index.crates.io-")
	{
		panic!(
			"Due to Cargo's limitations, Yazi on crates.io must be built with `cargo install --force yazi-build`"
		);
	}
}
