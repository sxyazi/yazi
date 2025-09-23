use std::{ffi::OsStr, fs::Metadata, ops::Deref, time::{Duration, SystemTime, UNIX_EPOCH}};

use anyhow::bail;
use yazi_macro::{unix_either, win_either};
use yazi_shared::url::Url;

use super::ChaKind;
use crate::{cha::{ChaMode, ChaType}, provider};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cha {
	pub kind:  ChaKind,
	pub mode:  ChaMode,
	pub len:   u64,
	pub atime: Option<SystemTime>,
	pub btime: Option<SystemTime>,
	pub ctime: Option<SystemTime>,
	pub mtime: Option<SystemTime>,
	pub dev:   u64,
	pub uid:   u32,
	pub gid:   u32,
	pub nlink: u64,
}

impl Deref for Cha {
	type Target = ChaMode;

	fn deref(&self) -> &Self::Target { &self.mode }
}

impl Default for Cha {
	fn default() -> Self {
		Self {
			kind:  ChaKind::DUMMY,
			mode:  ChaMode::empty(),
			len:   0,
			atime: None,
			btime: None,
			ctime: None,
			mtime: None,
			dev:   0,
			uid:   0,
			gid:   0,
			nlink: 0,
		}
	}
}

impl Cha {
	#[inline]
	pub fn new(name: &OsStr, meta: Metadata) -> Self {
		Self::from_bare(&meta).attach(ChaKind::hidden(name, &meta))
	}

	#[inline]
	pub async fn from_url<'a>(url: impl Into<Url<'a>>) -> std::io::Result<Self> {
		let url = url.into();
		Ok(Self::from_follow(url, provider::symlink_metadata(url).await?).await)
	}

	pub async fn from_follow<'a, U>(url: U, mut cha: Self) -> Self
	where
		U: Into<Url<'a>>,
	{
		let url: Url = url.into();
		let mut retain = cha.kind & (ChaKind::HIDDEN | ChaKind::SYSTEM);

		if cha.is_link() {
			retain |= ChaKind::FOLLOW;
			cha = provider::metadata(url).await.unwrap_or(cha);
		}

		cha.attach(retain)
	}

	pub fn from_dummy<'a, U>(_url: U, r#type: Option<ChaType>) -> Self
	where
		U: Into<Url<'a>>,
	{
		let mut kind = ChaKind::DUMMY;
		let mode = r#type.map(ChaMode::from_bare).unwrap_or_default();

		#[cfg(unix)]
		if _url.into().urn().is_hidden() {
			kind |= ChaKind::HIDDEN;
		}

		Self { kind, mode, ..Default::default() }
	}

	fn from_bare(m: &Metadata) -> Self {
		#[cfg(unix)]
		use std::os::unix::fs::MetadataExt;

		#[cfg(unix)]
		let mode = {
			use std::os::unix::fs::PermissionsExt;
			ChaMode::from_bits_retain(m.permissions().mode() as u16)
		};

		#[cfg(windows)]
		let mode = {
			if m.is_file() {
				ChaMode::T_FILE
			} else if m.is_dir() {
				ChaMode::T_DIR
			} else if m.is_symlink() {
				ChaMode::T_LINK
			} else {
				ChaMode::empty()
			}
		};

		Self {
			kind: ChaKind::empty(),
			mode,
			len: m.len(),
			atime: m.accessed().ok(),
			btime: m.created().ok(),
			ctime: unix_either!(
				UNIX_EPOCH.checked_add(Duration::new(m.ctime() as u64, m.ctime_nsec() as u32)),
				None
			),
			mtime: m.modified().ok(),
			dev: unix_either!(m.dev(), 0) as _,
			uid: unix_either!(m.uid(), 0) as _,
			gid: unix_either!(m.gid(), 0) as _,
			nlink: unix_either!(m.nlink(), 0) as _,
		}
	}

	#[inline]
	pub fn hits(self, c: Self) -> bool {
		self.len == c.len
			&& self.mtime == c.mtime
			&& unix_either!(self.ctime == c.ctime, true)
			&& self.btime == c.btime
			&& self.kind == c.kind
			&& unix_either!(self.mode == c.mode, true)
	}

	#[inline]
	fn attach(mut self, kind: ChaKind) -> Self {
		self.kind |= kind;
		self
	}
}

impl Cha {
	#[inline]
	pub fn is_link(self) -> bool {
		self.kind.contains(ChaKind::FOLLOW) || *self.mode == ChaType::Link
	}

	#[inline]
	pub fn is_orphan(self) -> bool {
		*self.mode == ChaType::Link && self.kind.contains(ChaKind::FOLLOW)
	}

	#[inline]
	pub const fn is_hidden(self) -> bool {
		win_either!(
			self.kind.contains(ChaKind::HIDDEN) || self.kind.contains(ChaKind::SYSTEM),
			self.kind.contains(ChaKind::HIDDEN)
		)
	}

	#[inline]
	pub const fn is_dummy(self) -> bool { self.kind.contains(ChaKind::DUMMY) }

	pub fn atime_dur(self) -> anyhow::Result<Duration> {
		if let Some(atime) = self.atime {
			Ok(atime.duration_since(UNIX_EPOCH)?)
		} else {
			bail!("atime not available");
		}
	}

	pub fn mtime_dur(self) -> anyhow::Result<Duration> {
		if let Some(mtime) = self.mtime {
			Ok(mtime.duration_since(UNIX_EPOCH)?)
		} else {
			bail!("mtime not available");
		}
	}

	pub fn btime_dur(self) -> anyhow::Result<Duration> {
		if let Some(btime) = self.btime {
			Ok(btime.duration_since(UNIX_EPOCH)?)
		} else {
			bail!("btime not available");
		}
	}

	pub fn ctime_dur(self) -> anyhow::Result<Duration> {
		if let Some(ctime) = self.ctime {
			Ok(ctime.duration_since(UNIX_EPOCH)?)
		} else {
			bail!("ctime not available");
		}
	}
}
