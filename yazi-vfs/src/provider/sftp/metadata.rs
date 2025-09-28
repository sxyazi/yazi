use std::{ffi::OsStr, io, time::{Duration, UNIX_EPOCH}};

use yazi_fs::cha::ChaKind;

pub(crate) struct Cha(pub(crate) yazi_fs::cha::Cha);

impl From<Cha> for yazi_sftp::fs::Attrs {
	fn from(cha: Cha) -> Self {
		Self {
			size:     Some(cha.0.len),
			uid:      Some(cha.0.uid),
			gid:      Some(cha.0.gid),
			perm:     Some(cha.0.mode.bits() as u32),
			atime:    cha.0.atime_dur().ok().map(|d| d.as_secs() as u32),
			mtime:    cha.0.mtime_dur().ok().map(|d| d.as_secs() as u32),
			extended: Default::default(),
		}
	}
}

impl TryFrom<&yazi_sftp::fs::DirEntry> for Cha {
	type Error = io::Error;

	fn try_from(ent: &yazi_sftp::fs::DirEntry) -> Result<Self, Self::Error> {
		let mut cha = Self::try_from((ent.name().as_ref(), ent.attrs()))?;
		cha.0.nlink = ent.nlink().unwrap_or_default();
		Ok(cha)
	}
}

impl TryFrom<(&OsStr, &yazi_sftp::fs::Attrs)> for Cha {
	type Error = io::Error;

	fn try_from((name, attrs): (&OsStr, &yazi_sftp::fs::Attrs)) -> Result<Self, Self::Error> {
		let kind =
			if name.as_encoded_bytes().starts_with(b".") { ChaKind::HIDDEN } else { ChaKind::empty() };

		Ok(Cha(yazi_fs::cha::Cha {
			kind,
			mode: ChaMode::try_from(attrs)?.0,
			len: attrs.size.unwrap_or(0),
			atime: attrs.atime.and_then(|t| UNIX_EPOCH.checked_add(Duration::from_secs(t as u64))),
			btime: None,
			ctime: None,
			mtime: attrs.mtime.and_then(|t| UNIX_EPOCH.checked_add(Duration::from_secs(t as u64))),
			dev: 0,
			uid: attrs.uid.unwrap_or(0),
			gid: attrs.gid.unwrap_or(0),
			nlink: 0,
		}))
	}
}

// --- ChaMode
pub(super) struct ChaMode(pub(super) yazi_fs::cha::ChaMode);

impl TryFrom<&yazi_sftp::fs::Attrs> for ChaMode {
	type Error = io::Error;

	fn try_from(attrs: &yazi_sftp::fs::Attrs) -> Result<Self, Self::Error> {
		yazi_fs::cha::ChaMode::try_from(attrs.perm.unwrap_or_default() as u16)
			.map(Self)
			.map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
	}
}
