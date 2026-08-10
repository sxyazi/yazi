use std::{fs, path::Path, process::Command};

use anyhow::{Context, Result};
use yazi_macro::ok_or_not_found;

use super::{Build, cargo, copy_bins, copy_file, is_windows_target, run, workspace_root};

pub(super) struct Dest {
	target: String,
}

impl Dest {
	pub(super) fn new(target: String) -> Self { Self { target } }

	pub(super) fn run(&self) -> Result<()> {
		let build = Build::new(&self.target).completions();
		build.run()?;

		let to = workspace_root()?.join("target/release");
		fs::create_dir_all(&to)?;
		copy_bins(&build.target, build.profile(), &to)?;

		self.deb()?;
		self.stage()?;
		self.archive()
	}

	fn deb(&self) -> Result<()> {
		if !self.target.contains("-linux-") {
			return Ok(());
		} else if !matches!(self.target.split('-').next(), Some("aarch64" | "x86_64")) {
			return Ok(());
		}

		run(Command::new(cargo()).args(["install", "cargo-deb"]))
			.context("failed to install cargo-deb")?;

		run(
			Command::new(cargo())
				.args(["deb", "-p", "yazi-packing", "--no-build", "--target"])
				.arg(&self.target)
				.args(["-o", &format!("yazi-{}.deb", self.target)]),
		)
		.context("failed to package the Debian archive")
	}

	fn stage(&self) -> Result<()> {
		let root = workspace_root()?;
		let dest = root.join(format!("yazi-{}", self.target));
		ok_or_not_found!(fs::remove_dir_all(&dest));

		let completions = dest.join("completions");
		fs::create_dir_all(&completions)?;

		let ext = if is_windows_target(&self.target) { ".exe" } else { "" };
		copy_file(&root.join(format!("target/release/ya{ext}")), &dest.join(format!("ya{ext}")))?;
		copy_file(&root.join(format!("target/release/yazi{ext}")), &dest.join(format!("yazi{ext}")))?;

		Self::copy_files(&root.join("yazi-cli/completions"), &completions)?;
		Self::copy_files(&root.join("yazi-boot/completions"), &completions)?;

		copy_file(&root.join("README.md"), &dest.join("README.md"))?;
		copy_file(&root.join("LICENSE"), &dest.join("LICENSE"))
	}

	fn archive(&self) -> Result<()> {
		let root = workspace_root()?;
		let name = format!("yazi-{}", self.target);

		let archive = root.join(format!("{name}.zip"));
		ok_or_not_found!(fs::remove_file(&archive));

		if cfg!(windows) {
			run(Command::new("tar").arg("-caf").arg(&archive).arg(&name))
		} else {
			run(Command::new("zip").arg("-r").arg(&archive).arg(&name))
		}
		.context("failed to create the release archive")
	}

	fn copy_files(from: &Path, to: &Path) -> Result<()> {
		for dent in fs::read_dir(from).with_context(|| format!("failed to read {}", from.display()))? {
			let dent = dent?;
			if dent.file_type()?.is_file() {
				copy_file(&dent.path(), &to.join(dent.file_name()))?;
			}
		}
		Ok(())
	}
}
