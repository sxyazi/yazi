use std::{fs::Metadata, ops::Deref, time::{Duration, SystemTime, UNIX_EPOCH}};

use anyhow::bail;
use mlua::FromLua;
use serde::{Deserialize, Serialize};
use yazi_macro::{unix_either, win_either};
use yazi_shared::{strand::AsStrand, url::AsUrl};

use super::StatKind;
use crate::stat::{StatMode, StatType};

#[derive(Clone, Copy, Debug, Deserialize, Eq, FromLua, PartialEq, Serialize)]
pub struct Stat {
	pub kind:  StatKind,
	pub mode:  StatMode,
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

impl Deref for Stat {
	type Target = StatMode;

	fn deref(&self) -> &Self::Target { &self.mode }
}

impl Default for Stat {
	fn default() -> Self {
		Self {
			kind:  StatKind::DUMMY,
			mode:  StatMode::empty(),
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

impl Stat {
	#[inline]
	pub fn new<T>(name: T, meta: Metadata) -> Self
	where
		T: AsStrand,
	{
		Self::from_bare(&meta).attach(StatKind::hidden(name, &meta) | StatKind::reparse(&meta))
	}

	pub(crate) fn from_dummy<U>(_url: U, r#type: Option<StatType>) -> Self
	where
		U: AsUrl,
	{
		#[allow(unused_mut)]
		let mut kind = StatKind::DUMMY;
		let mode = r#type.map(StatMode::from_bare).unwrap_or_default();

		#[cfg(unix)]
		if _url.as_url().urn().is_hidden() {
			kind |= StatKind::HIDDEN;
		}

		Self { kind, mode, ..Default::default() }
	}

	fn from_bare(m: &Metadata) -> Self {
		#[cfg(unix)]
		use std::os::unix::fs::MetadataExt;

		#[cfg(unix)]
		let mode = {
			use std::os::unix::fs::PermissionsExt;
			StatMode::from_bits_retain(m.permissions().mode() as u16)
		};

		#[cfg(windows)]
		let mut mode = if m.is_file() {
			StatMode::T_FILE
		} else if m.is_dir() {
			StatMode::T_DIR
		} else if m.is_symlink() {
			StatMode::T_LINK
		} else {
			StatMode::empty()
		};
		#[cfg(windows)]
		if !m.permissions().readonly() {
			mode |= StatMode::U_WRITE;
		}

		Self {
			kind: StatKind::empty(),
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
			&& self.ctime == c.ctime
			&& self.btime == c.btime
			&& self.kind == c.kind
			&& self.mode == c.mode
	}

	#[inline]
	fn attach(mut self, kind: StatKind) -> Self {
		self.kind |= kind;
		self
	}

	#[inline]
	pub fn follow(self, followed: Option<Self>) -> Self {
		if !self.is_link() {
			return self;
		}

		let retain = self.kind & (StatKind::HIDDEN | StatKind::SYSTEM | StatKind::REPARSE);
		followed.unwrap_or(self).attach(retain | StatKind::FOLLOW)
	}
}

impl Stat {
	#[inline]
	pub fn is_link(self) -> bool {
		self.kind.contains(StatKind::FOLLOW) || *self.mode == StatType::Link
	}

	#[inline]
	pub fn is_orphan(self) -> bool {
		*self.mode == StatType::Link && self.kind.contains(StatKind::FOLLOW)
	}

	#[inline]
	pub const fn is_hidden(self) -> bool {
		win_either!(
			self.kind.contains(StatKind::HIDDEN) || self.kind.contains(StatKind::SYSTEM),
			self.kind.contains(StatKind::HIDDEN)
		)
	}

	#[inline]
	pub const fn is_dummy(self) -> bool { self.kind.contains(StatKind::DUMMY) }

	#[inline]
	const fn is_reparse(self) -> bool { self.kind.contains(StatKind::REPARSE) }

	#[inline]
	pub fn is_indirect(self) -> bool { self.is_link() || self.is_reparse() }

	pub(crate) fn atime_dur(self) -> anyhow::Result<Duration> {
		if let Some(atime) = self.atime {
			Ok(atime.duration_since(UNIX_EPOCH)?)
		} else {
			bail!("atime not available");
		}
	}

	pub(crate) fn btime_dur(self) -> anyhow::Result<Duration> {
		if let Some(btime) = self.btime {
			Ok(btime.duration_since(UNIX_EPOCH)?)
		} else {
			bail!("btime not available");
		}
	}

	pub(crate) fn ctime_dur(self) -> anyhow::Result<Duration> {
		if let Some(ctime) = self.ctime {
			Ok(ctime.duration_since(UNIX_EPOCH)?)
		} else {
			bail!("ctime not available");
		}
	}

	pub fn mtime_dur(self) -> anyhow::Result<Duration> {
		if let Some(mtime) = self.mtime {
			Ok(mtime.duration_since(UNIX_EPOCH)?)
		} else {
			bail!("mtime not available");
		}
	}
}
