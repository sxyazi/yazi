use std::{fs, path::PathBuf};

use anyhow::Result;

use super::{Build, cargo_bin_dir, copy_bins};

pub(super) struct Install {
	bin_dir: PathBuf,
}

impl Install {
	pub(super) fn new(bin_dir: PathBuf) -> Self { Self { bin_dir } }

	pub(super) fn run(self) -> Result<()> {
		let build = Build::new(String::new());
		build.run()?;

		let bin_dir = self.bin_dir()?;
		fs::create_dir_all(&bin_dir)?;

		copy_bins(&build.target, build.profile(), &bin_dir)
	}

	fn bin_dir(self) -> Result<PathBuf> {
		if !self.bin_dir.as_os_str().is_empty() {
			return Ok(self.bin_dir);
		}
		cargo_bin_dir()
	}
}
