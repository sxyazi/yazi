use std::time::SystemTime;

use object_store::ObjectMeta;
use yazi_fs::cha::{Cha, ChaKind, ChaMode};
use yazi_shared::strand::AsStrand;

pub(super) fn dir(_name: impl AsStrand) -> Cha {
	Cha {
		kind: ChaKind::empty(),
		mode: default_dir_mode(),
		len: 0,
		atime: None,
		btime: None,
		ctime: None,
		mtime: None,
		dev: 0,
		uid: 0,
		gid: 0,
		nlink: 0,
	}
}

pub(super) fn object(_name: impl AsStrand, meta: &ObjectMeta) -> Cha {
	Cha {
		kind: ChaKind::empty(),
		mode: default_file_mode(),
		len: meta.size,
		atime: None,
		btime: None,
		ctime: None,
		mtime: Some(SystemTime::from(meta.last_modified)),
		dev: 0,
		uid: 0,
		gid: 0,
		nlink: 0,
	}
}

#[inline]
const fn default_dir_mode() -> ChaMode {
	ChaMode::T_DIR
		.union(ChaMode::U_READ)
		.union(ChaMode::U_WRITE)
		.union(ChaMode::U_EXEC)
		.union(ChaMode::G_READ)
		.union(ChaMode::G_EXEC)
		.union(ChaMode::O_READ)
		.union(ChaMode::O_EXEC)
}

#[inline]
const fn default_file_mode() -> ChaMode {
	ChaMode::T_FILE
		.union(ChaMode::U_READ)
		.union(ChaMode::U_WRITE)
		.union(ChaMode::G_READ)
		.union(ChaMode::O_READ)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn object_uses_readable_file_permissions() {
		let cha = object(
			"producer.8-2.json",
			&ObjectMeta {
				location: "producer.8-2.json".into(),
				last_modified: chrono::Utc::now().into(),
				size: 61,
				e_tag: None,
				version: None,
			},
		);

		assert!(cha.mode.contains(ChaMode::T_FILE));
		assert!(cha.mode.contains(ChaMode::U_READ));
		assert!(cha.mode.contains(ChaMode::U_WRITE));
		assert!(cha.mode.contains(ChaMode::G_READ));
		assert!(cha.mode.contains(ChaMode::O_READ));
	}

	#[test]
	fn dir_uses_traversable_directory_permissions() {
		let cha = dir("e2e_test");

		assert!(cha.mode.contains(ChaMode::T_DIR));
		assert!(cha.mode.contains(ChaMode::U_READ));
		assert!(cha.mode.contains(ChaMode::U_WRITE));
		assert!(cha.mode.contains(ChaMode::U_EXEC));
		assert!(cha.mode.contains(ChaMode::G_READ));
		assert!(cha.mode.contains(ChaMode::G_EXEC));
		assert!(cha.mode.contains(ChaMode::O_READ));
		assert!(cha.mode.contains(ChaMode::O_EXEC));
	}
}
