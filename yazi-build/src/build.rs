use std::{env, fs, iter, path::{Path, PathBuf}, process::Command};

use anyhow::{Context, Result, ensure};
use yazi_macro::ok_or_not_found;

use super::{cargo, is_linux_target, is_sparc64_target, is_windows_target, run, workspace_root};

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
		if is_linux_target(&self.target) && !is_sparc64_target(&self.target) {
			self.expose_lld(&mut cmd)?;
		}

		run(&mut cmd).context("failed to build Yazi")
	}

	pub(super) fn profile(&self) -> &'static str {
		if is_windows_target(&self.target) { "release-windows" } else { "release" }
	}

	fn expose_lld(&self, cmd: &mut Command) -> Result<()> {
		let rust_lld = Self::rustc_libdir()?
			.parent()
			.context("Rust target libdir has no parent")?
			.join("bin/rust-lld");
		ensure!(rust_lld.is_file(), "{} does not exist", rust_lld.display());

		let temp_dir = workspace_root()?.join("target/.rust-lld");
		fs::create_dir_all(&temp_dir).context("failed to create `target/.rust-lld` directory")?;
		for name in ["ld.lld".to_owned()].into_iter().chain(self.lld_alias()) {
			let link = temp_dir.join(name);
			ok_or_not_found!(fs::remove_file(&link));
			#[cfg(unix)]
			std::os::unix::fs::symlink(&rust_lld, link)?;
			#[cfg(not(unix))]
			anyhow::bail!("bundled rust-lld setup requires a Unix host");
		}

		let path = env::var_os("PATH").unwrap_or_default();
		cmd.env("PATH", env::join_paths(iter::once(temp_dir).chain(env::split_paths(&path)))?);
		Ok(())
	}

	fn rustc_libdir() -> Result<PathBuf> {
		let output = Command::new(env::var_os("RUSTC").unwrap_or_else(|| "rustc".into()))
			.args(["--print", "target-libdir"])
			.output()
			.context("failed to locate the Rust target libdir")?;

		ensure!(output.status.success(), "`rustc --print target-libdir` failed");
		Ok(String::from_utf8(output.stdout)?.trim().into())
	}

	// Cross GCC resolves `-fuse-ld=lld` as `<toolchain-prefix>ld.lld`.
	// For example:
	//   `aarch64-linux-musl-gcc.sh` => `aarch64-linux-musl-ld.lld`
	// if Cargo has no configured linker:
	//   `CROSS_TOOLCHAIN_PREFIX=x86_64-linux-musl-` => `x86_64-linux-musl-ld.lld`
	fn lld_alias(&self) -> Option<String> {
		let var = env::var_os(format!(
			"CARGO_TARGET_{}_LINKER",
			self.target.replace('-', "_").to_ascii_uppercase()
		))
		.or_else(|| env::var_os("CROSS_TOOLCHAIN_PREFIX"))?;

		let prefix = Path::new(&var).file_stem()?.to_str()?.trim_end_matches("gcc");

		Some(format!("{prefix}ld.lld"))
	}
}
