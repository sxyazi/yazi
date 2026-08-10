use std::process::Command;

use anyhow::{Context, Result};

use super::{cargo, is_windows_target, run};

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
			.args(["-Z", "trim-paths", "--config", ".cargo/release.toml", "build", "--locked"])
			.args(["--profile", self.profile()]);

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
}
