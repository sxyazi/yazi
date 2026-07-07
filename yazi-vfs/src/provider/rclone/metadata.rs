use std::{io, time::SystemTime};

use serde::Deserialize;
use yazi_fs::cha::{Cha, ChaKind, ChaMode};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(super) struct Item {
	#[serde(default)]
	pub name:     String,
	#[serde(default)]
	pub size:     i64,
	#[serde(default)]
	pub mod_time: Option<String>,
	#[serde(default)]
	pub is_dir:   bool,
}

impl Item {
	pub(super) fn cha(&self) -> io::Result<Cha> {
		let kind = if self.name.starts_with('.') { ChaKind::HIDDEN } else { ChaKind::empty() };

		// Object stores have no POSIX permissions; synthesize a sensible mode
		let mode = ChaMode::try_from(if self.is_dir { 0o040755u16 } else { 0o100644u16 })
			.map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

		let mtime = self
			.mod_time
			.as_deref()
			.and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
			.map(SystemTime::from);

		Ok(Cha {
			kind,
			mode,
			len: self.size.max(0) as u64,
			atime: None,
			btime: None,
			ctime: None,
			mtime,
			dev: 0,
			uid: 0,
			gid: 0,
			nlink: 0,
		})
	}
}
