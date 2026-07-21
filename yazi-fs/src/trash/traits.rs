use crate::cha::{Cha, ChaKind, ChaMode};

pub(super) trait TrashCha: Sized {
	fn from_mold(is_dir: bool) -> Self;

	#[cfg(all(unix, not(target_os = "android"), not(target_os = "ios")))]
	fn from_trash(
		path: &std::path::Path,
		name: &std::ffi::OsStr,
		follow: bool,
	) -> std::io::Result<Self>;
}

impl TrashCha for Cha {
	fn from_mold(is_dir: bool) -> Self {
		let mut cha = Self::default();
		cha.kind.remove(ChaKind::DUMMY);
		cha.mode = if is_dir { ChaMode::T_DIR | ChaMode::U_EXEC } else { ChaMode::T_FILE };
		cha.mode |= ChaMode::U_READ | ChaMode::U_WRITE;
		cha
	}

	#[cfg(all(unix, not(target_os = "android"), not(target_os = "ios")))]
	fn from_trash(
		path: &std::path::Path,
		name: &std::ffi::OsStr,
		follow: bool,
	) -> std::io::Result<Self> {
		let cha = Cha::new(name, std::fs::symlink_metadata(path)?);
		Ok(if cha.is_link() && follow {
			cha.follow(std::fs::metadata(path).ok().map(|meta| Cha::new(name, meta)))
		} else {
			cha
		})
	}
}
