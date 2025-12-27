use std::{io, time::{Duration, UNIX_EPOCH}};

use yazi_fs::cha::ChaKind;

// --- Attrs
pub(crate) struct Attrs(pub(crate) yazi_fs::provider::Attrs);

impl TryFrom<Attrs> for yazi_sftp::fs::Attrs {
	type Error = ();

	fn try_from(value: Attrs) -> Result<Self, Self::Error> {
		let attrs = Self {
			size:     None,
			uid:      None,
			gid:      None,
			perm:     value.0.mode.map(|m| m.bits() as u32),
			atime:    value.0.atime_dur().map(|d| d.as_secs() as u32),
			mtime:    value.0.mtime_dur().map(|d| d.as_secs() as u32),
			extended: Default::default(),
		};

		if attrs.is_empty() { Err(()) } else { Ok(attrs) }
	}
}

// --- Cha
pub(crate) struct Cha(pub(crate) yazi_fs::cha::Cha);

impl TryFrom<&yazi_sftp::fs::DirEntry> for Cha {
	type Error = io::Error;

	fn try_from(ent: &yazi_sftp::fs::DirEntry) -> Result<Self, Self::Error> {
		let mut cha = Self::try_from((ent.name(), ent.attrs()))?;
		cha.0.nlink = ent.nlink().unwrap_or_default();
		Ok(cha)
	}
}

impl TryFrom<(&[u8], &yazi_sftp::fs::Attrs)> for Cha {
	type Error = io::Error;

	fn try_from((name, attrs): (&[u8], &yazi_sftp::fs::Attrs)) -> Result<Self, Self::Error> {
		let kind = if name.starts_with(b".") { ChaKind::HIDDEN } else { ChaKind::empty() };

		Ok(Self(yazi_fs::cha::Cha {
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
