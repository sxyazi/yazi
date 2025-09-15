use std::{fs::{FileType, Metadata}, ops::Deref, time::SystemTime};

use yazi_macro::{unix_either, win_either};
use yazi_shared::url::Url;

use super::ChaKind;
use crate::{cha::ChaMode, provider};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cha {
	pub kind:  ChaKind,
	pub mode:  ChaMode,
	pub len:   u64,
	pub atime: Option<SystemTime>,
	pub btime: Option<SystemTime>,
	#[cfg(unix)]
	pub ctime: Option<SystemTime>,
	pub mtime: Option<SystemTime>,
	#[cfg(unix)]
	pub dev:   libc::dev_t,
	#[cfg(unix)]
	pub uid:   u32,
	#[cfg(unix)]
	pub gid:   u32,
	#[cfg(unix)]
	pub nlink: u64,
}

impl Deref for Cha {
	type Target = ChaMode;

	fn deref(&self) -> &Self::Target { &self.mode }
}

impl Default for Cha {
	fn default() -> Self {
		Self {
			kind:               ChaKind::DUMMY,
			mode:               ChaMode::empty(),
			len:                0,
			atime:              None,
			btime:              None,
			#[cfg(unix)]
			ctime:              None,
			mtime:              None,
			#[cfg(unix)]
			dev:                0,
			#[cfg(unix)]
			uid:                0,
			#[cfg(unix)]
			gid:                0,
			#[cfg(unix)]
			nlink:              0,
		}
	}
}

impl Cha {
	#[inline]
	pub fn new<'a, U>(url: U, meta: Metadata) -> Self
	where
		U: Into<Url<'a>>,
	{
		Self::from_bare(&meta).attach(ChaKind::hidden(url, &meta))
	}

	#[inline]
	pub async fn from_url<'a>(url: impl Into<Url<'a>>) -> std::io::Result<Self> {
		let url = url.into();
		Ok(Self::from_follow(url, provider::symlink_metadata(url).await?).await)
	}

	pub async fn from_follow<'a, U>(url: U, mut meta: Metadata) -> Self
	where
		U: Into<Url<'a>>,
	{
		let url = url.into();
		let mut attached = ChaKind::hidden(url, &meta);

		if meta.is_symlink() {
			attached |= ChaKind::LINK;
			meta = provider::metadata(url).await.unwrap_or(meta);
		}
		if meta.is_symlink() {
			attached |= ChaKind::ORPHAN;
		}

		Self::from_bare(&meta).attach(attached)
	}

	pub fn from_dummy<'a, U>(_url: U, ft: Option<FileType>) -> Self
	where
		U: Into<Url<'a>>,
	{
		let mode = ft.map(ChaMode::from_bare).unwrap_or_default();

		let mut kind = ChaKind::DUMMY;
		if mode.is_link() {
			kind |= ChaKind::LINK;
		}

		#[cfg(unix)]
		if _url.into().urn().is_hidden() {
			kind |= ChaKind::HIDDEN;
		}

		Self { kind, mode, ..Default::default() }
	}

	fn from_bare(m: &Metadata) -> Self {
		#[cfg(unix)]
		use std::{os::unix::fs::MetadataExt, time::{Duration, UNIX_EPOCH}};

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
			#[cfg(unix)]
			ctime: UNIX_EPOCH.checked_add(Duration::new(m.ctime() as u64, m.ctime_nsec() as u32)),
			mtime: m.modified().ok(),
			#[cfg(unix)]
			dev: m.dev() as _,
			#[cfg(unix)]
			uid: m.uid() as _,
			#[cfg(unix)]
			gid: m.gid() as _,
			#[cfg(unix)]
			nlink: m.nlink() as _,
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
	pub const fn is_hidden(&self) -> bool {
		win_either!(
			self.kind.contains(ChaKind::HIDDEN) || self.kind.contains(ChaKind::SYSTEM),
			self.kind.contains(ChaKind::HIDDEN)
		)
	}

	#[inline]
	pub const fn is_orphan(&self) -> bool { self.kind.contains(ChaKind::ORPHAN) }

	#[inline]
	pub const fn is_dummy(&self) -> bool { self.kind.contains(ChaKind::DUMMY) }
}
