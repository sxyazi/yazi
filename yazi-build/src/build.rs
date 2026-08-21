use std::{env::{self, consts::EXE_SUFFIX}, path::PathBuf, process::Command};

use anyhow::{Context, Result, ensure};

use super::{cargo, is_linux_target, is_sparc64_target, is_windows_target, run};

pub(super) struct Build {
	pub(super) target: String,
	completions:       bool,
}

impl Build {
	pub(super) fn new(target: impl Into<String>) -> Self {
		Self { target: target.into(), completions: false }
	}

	pub(super) fn completions(mut self) -> Self {
		self.completions = true;
		self
	}

	pub(super) fn run(&self) -> Result<()> {
		let mut cmd = Command::new(cargo());
		cmd
			.env("RUSTC_BOOTSTRAP", "1")
			.env("CARGO_TARGET_DIR", "target")
			.args(["--config", ".cargo/release.toml"]);

		if is_linux_target(&self.target) && !is_sparc64_target(&self.target) && Self::has_rust_lld()? {
			cmd.args(["--config", r#"target.'cfg(all())'.rustflags=["-Clink-self-contained=+linker"]"#]);
		}

		cmd.args(["build", "--locked", "--profile", self.profile()]);

		if !self.target.is_empty() {
			cmd.arg("--target").arg(&self.target);
		}
		if self.completions {
			cmd.env("YAZI_GEN_COMPLETIONS", "1");
		}

		run(&mut cmd).context("failed to build Yazi")
	}

	pub(super) fn profile(&self) -> &'static str {
		if is_windows_target(&self.target) { "release-windows" } else { "release" }
	}

	fn has_rust_lld() -> Result<bool> {
		let output = Command::new(env::var_os("RUSTC").unwrap_or_else(|| "rustc".into()))
			.args(["--print", "target-libdir"])
			.output()
			.context("failed to locate the Rust target libdir")?;
		ensure!(output.status.success(), "`rustc --print target-libdir` failed");

		let libdir: PathBuf = String::from_utf8(output.stdout)
			.context("rustc returned an invalid target libdir")?
			.trim()
			.into();
		let rust_lld = libdir
			.parent()
			.context("Rust target libdir has no parent")?
			.join("bin")
			.join(format!("rust-lld{EXE_SUFFIX}"));

		Ok(rust_lld.is_file())
	}
}
