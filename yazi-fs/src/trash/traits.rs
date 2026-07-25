use std::fs;

use crate::cha::{Cha, ChaKind, ChaMode};

pub(super) trait TrashCha: Sized {
	fn from_mold(is_dir: bool) -> Self;

	fn from_trash(path: &std::path::Path, name: &std::ffi::OsStr) -> std::io::Result<(Self, Self)>;
}

impl TrashCha for Cha {
	fn from_mold(is_dir: bool) -> Self {
		let mut cha = Self::default();
		cha.kind.remove(ChaKind::DUMMY);
		cha.mode = if is_dir { ChaMode::T_DIR | ChaMode::U_EXEC } else { ChaMode::T_FILE };
		cha.mode |= ChaMode::U_READ | ChaMode::U_WRITE;
		cha
	}

	fn from_trash(path: &std::path::Path, name: &std::ffi::OsStr) -> std::io::Result<(Self, Self)> {
		let lcha = Cha::new(name, fs::symlink_metadata(path)?);
		let cha = if lcha.is_link() {
			lcha.follow(fs::metadata(path).ok().map(|meta| Cha::new(name, meta)))
		} else {
			lcha
		};
		Ok((lcha, cha))
	}
}
