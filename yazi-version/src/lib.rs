use std::{env::consts::{ARCH, OS}, fmt::Write};

pub fn version() -> &'static str { concat!(env!("CARGO_PKG_VERSION"), " ", env!("VERGEN_GIT_SHA")) }

pub fn version_long() -> &'static str {
	concat!(
		env!("CARGO_PKG_VERSION"),
		" (",
		env!("VERGEN_GIT_SHA"),
		" ",
		env!("VERGEN_BUILD_DATE"),
		")"
	)
}

pub fn version_full() -> String {
	let mut s = String::new();

	writeln!(s, "    Version: {}", version_long()).ok();
	writeln!(s, "    Debug  : {}", cfg!(debug_assertions)).ok();
	#[rustfmt::skip]
	writeln!(s, "    Triple : {} ({OS}-{ARCH})", env!("VERGEN_RUSTC_HOST_TRIPLE")).ok();
	#[rustfmt::skip]
	writeln!(s, "    Rustc  : {} ({} {})", env!("VERGEN_RUSTC_SEMVER"), &env!("VERGEN_RUSTC_COMMIT_HASH")[..8], env!("VERGEN_RUSTC_COMMIT_DATE")).ok();

	s
}
