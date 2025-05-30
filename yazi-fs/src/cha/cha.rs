use std::{fs::{FileType, Metadata}, path::Path, time::SystemTime};

use tokio::fs;
use yazi_macro::{unix_either, win_either};
use yazi_shared::url::Url;

use super::ChaKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cha {
	pub kind:  ChaKind,
	pub len:   u64,
	pub atime: Option<SystemTime>,
	pub btime: Option<SystemTime>,
	#[cfg(unix)]
	pub ctime: Option<SystemTime>,
	pub mtime: Option<SystemTime>,
	#[cfg(unix)]
	pub mode:  libc::mode_t,
	#[cfg(unix)]
	pub dev:   libc::dev_t,
	#[cfg(unix)]
	pub uid:   libc::uid_t,
	#[cfg(unix)]
	pub gid:   libc::gid_t,
	#[cfg(unix)]
	pub nlink: libc::nlink_t,
}

impl Default for Cha {
	fn default() -> Self {
		Self {
			kind:               ChaKind::DUMMY,
			len:                0,
			atime:              None,
			btime:              None,
			#[cfg(unix)]
			ctime:              None,
			mtime:              None,
			#[cfg(unix)]
			mode:               0,
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
	pub fn new(path: &Path, meta: Metadata) -> Self {
		Self::from_just_meta(&meta).attach(ChaKind::hidden(path, &meta))
	}

	#[inline]
	pub async fn from_url(url: &Url) -> std::io::Result<Self> {
		Ok(Self::from_follow(url, fs::symlink_metadata(url).await?).await)
	}

	pub async fn from_follow(path: &Path, mut meta: Metadata) -> Self {
		let mut attached = ChaKind::hidden(path, &meta);
		if meta.is_symlink() {
			attached |= ChaKind::LINK;
			meta = fs::metadata(path).await.unwrap_or(meta);
		}
		if meta.is_symlink() {
			attached |= ChaKind::ORPHAN;
		}

		Self::from_just_meta(&meta).attach(attached)
	}

	#[inline]
	pub fn from_dummy(url: &Url, ft: Option<FileType>) -> Self {
		let mut me = ft.map(Self::from_half_ft).unwrap_or_default();
		#[cfg(unix)]
		if yazi_shared::url::Urn::new(url).is_hidden() {
			me.kind |= ChaKind::HIDDEN;
		}
		me
	}

	fn from_half_ft(ft: FileType) -> Self {
		let mut kind = ChaKind::DUMMY;

		#[cfg(unix)]
		let mode = {
			use std::os::unix::fs::FileTypeExt;
			if ft.is_dir() {
				kind |= ChaKind::DIR;
				libc::S_IFDIR
			} else if ft.is_symlink() {
				kind |= ChaKind::LINK;
				libc::S_IFLNK
			} else if ft.is_block_device() {
				libc::S_IFBLK
			} else if ft.is_char_device() {
				libc::S_IFCHR
			} else if ft.is_fifo() {
				libc::S_IFIFO
			} else if ft.is_socket() {
				libc::S_IFSOCK
			} else {
				0
			}
		};

		#[cfg(windows)]
		{
			if ft.is_dir() {
				kind |= ChaKind::DIR;
			} else if ft.is_symlink() {
				kind |= ChaKind::LINK;
			}
		}

		Self {
			kind,
			#[cfg(unix)]
			mode,
			..Default::default()
		}
	}

	fn from_just_meta(m: &Metadata) -> Self {
		#[cfg(unix)]
		use std::{os::unix::{fs::MetadataExt, prelude::PermissionsExt}, time::{Duration, UNIX_EPOCH}};

		let mut kind = ChaKind::empty();
		if m.is_dir() {
			kind |= ChaKind::DIR;
		} else if m.is_symlink() {
			kind |= ChaKind::LINK;
		}

		Self {
			kind,
			len: m.len(),
			atime: m.accessed().ok(),
			btime: m.created().ok(),
			#[cfg(unix)]
			ctime: UNIX_EPOCH.checked_add(Duration::new(m.ctime() as u64, m.ctime_nsec() as u32)),
			mtime: m.modified().ok(),
			#[cfg(unix)]
			mode: m.permissions().mode() as _,
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
	pub const fn is_dir(&self) -> bool { self.kind.contains(ChaKind::DIR) }

	#[inline]
	pub const fn is_hidden(&self) -> bool {
		win_either!(self.kind.contains(ChaKind::SYSTEM), self.kind.contains(ChaKind::HIDDEN))
	}

	#[inline]
	pub const fn is_link(&self) -> bool { self.kind.contains(ChaKind::LINK) }

	#[inline]
	pub const fn is_orphan(&self) -> bool { self.kind.contains(ChaKind::ORPHAN) }

	#[inline]
	pub const fn is_dummy(&self) -> bool { self.kind.contains(ChaKind::DUMMY) }

	#[inline]
	pub const fn is_block(&self) -> bool {
		unix_either!(self.mode & libc::S_IFMT == libc::S_IFBLK, false)
	}

	#[inline]
	pub const fn is_char(&self) -> bool {
		unix_either!(self.mode & libc::S_IFMT == libc::S_IFCHR, false)
	}

	#[inline]
	pub const fn is_fifo(&self) -> bool {
		unix_either!(self.mode & libc::S_IFMT == libc::S_IFIFO, false)
	}

	#[inline]
	pub const fn is_sock(&self) -> bool {
		unix_either!(self.mode & libc::S_IFMT == libc::S_IFSOCK, false)
	}

	#[inline]
	pub const fn is_exec(&self) -> bool { unix_either!(self.mode & libc::S_IXUSR != 0, false) }

	#[inline]
	pub const fn is_sticky(&self) -> bool { unix_either!(self.mode & libc::S_ISVTX != 0, false) }
}
