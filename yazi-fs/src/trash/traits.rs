use std::fs;

use crate::stat::{Stat, StatKind, StatMode};

pub(super) trait TrashStat: Sized {
	fn from_mold(is_dir: bool) -> Self;

	fn from_trash(path: &std::path::Path, name: &std::ffi::OsStr) -> std::io::Result<(Self, Self)>;
}

impl TrashStat for Stat {
	fn from_mold(is_dir: bool) -> Self {
		let mut stat = Self::default();
		stat.kind.remove(StatKind::DUMMY);
		stat.mode = if is_dir { StatMode::T_DIR | StatMode::U_EXEC } else { StatMode::T_FILE };
		stat.mode |= StatMode::U_READ | StatMode::U_WRITE;
		stat
	}

	fn from_trash(path: &std::path::Path, name: &std::ffi::OsStr) -> std::io::Result<(Self, Self)> {
		let lstat = Self::new(name, fs::symlink_metadata(path)?);
		let stat = if lstat.is_link() {
			lstat.follow(fs::metadata(path).ok().map(|meta| Self::new(name, meta)))
		} else {
			lstat
		};
		Ok((lstat, stat))
	}
}
