use std::{ffi::OsStr, io, time::{Duration, UNIX_EPOCH}};

use crate::cha::{Cha, ChaKind, ChaMode};

impl TryFrom<&yazi_sftp::fs::DirEntry> for Cha {
	type Error = io::Error;

	fn try_from(ent: &yazi_sftp::fs::DirEntry) -> Result<Self, Self::Error> {
		let mut cha = Self::try_from((ent.name().as_ref(), ent.attrs()))?;
		cha.nlink = ent.nlink().unwrap_or_default();
		Ok(cha)
	}
}

impl TryFrom<(&OsStr, &yazi_sftp::fs::Attrs)> for Cha {
	type Error = io::Error;

	fn try_from((name, attrs): (&OsStr, &yazi_sftp::fs::Attrs)) -> Result<Self, Self::Error> {
		let kind =
			if name.as_encoded_bytes().starts_with(b".") { ChaKind::HIDDEN } else { ChaKind::empty() };

		Ok(Cha {
			kind,
			mode: attrs.try_into()?,
			len: attrs.size.unwrap_or(0),
			atime: attrs.atime.and_then(|t| UNIX_EPOCH.checked_add(Duration::from_secs(t as u64))),
			btime: None,
			ctime: None,
			mtime: attrs.mtime.and_then(|t| UNIX_EPOCH.checked_add(Duration::from_secs(t as u64))),
			dev: 0,
			uid: attrs.uid.unwrap_or(0),
			gid: attrs.gid.unwrap_or(0),
			nlink: 0,
		})
	}
}

impl TryFrom<&yazi_sftp::fs::Attrs> for ChaMode {
	type Error = io::Error;

	fn try_from(attrs: &yazi_sftp::fs::Attrs) -> Result<Self, Self::Error> {
		ChaMode::try_from(attrs.perm.unwrap_or_default() as u16)
			.map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
	}
}

impl From<Cha> for yazi_sftp::fs::Attrs {
	fn from(cha: Cha) -> Self {
		Self {
			size:     Some(cha.len),
			uid:      Some(cha.uid),
			gid:      Some(cha.gid),
			perm:     Some(cha.mode.bits() as u32),
			atime:    cha.atime_dur().ok().map(|d| d.as_secs() as u32),
			mtime:    cha.mtime_dur().ok().map(|d| d.as_secs() as u32),
			extended: Default::default(),
		}
	}
}
