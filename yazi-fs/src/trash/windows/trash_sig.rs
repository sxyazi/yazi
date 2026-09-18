use std::{ffi::OsString, hash::Hash, io, mem, time::{Duration, UNIX_EPOCH}};

use windows::{Win32::UI::Shell::*, core::PCWSTR};
use yazi_shim::Twox128;

use super::shell_item::ShellItem;
use crate::stat::Stat;

pub(super) struct TrashSig {
	names:      Option<Vec<OsString>>,
	count_size: Option<(i64, i64)>,
}

impl TrashSig {
	pub(super) fn root() -> io::Result<Stat> {
		let root = ShellItem::root()?;
		let stat = root.stat()?;

		let mut info =
			SHQUERYRBINFO { cbSize: mem::size_of::<SHQUERYRBINFO>() as u32, ..Default::default() };

		let sig = if unsafe { SHQueryRecycleBinW(PCWSTR::null(), &mut info) }.is_ok() {
			Self { names: None, count_size: Some((info.i64NumItems, info.i64Size)) }
		} else if stat.mtime.is_some() {
			Self { names: None, count_size: None }
		} else {
			Self { names: Some(Self::names(&root)?), count_size: None }
		};

		Ok(sig.into_stat(stat))
	}

	pub(super) fn item(item: &ShellItem) -> io::Result<Stat> {
		let stat = item.stat()?;
		let sig = if stat.mtime.is_none() && stat.is_dir() {
			Self { names: Some(Self::names(item)?), count_size: None }
		} else {
			Self { names: None, count_size: None }
		};

		Ok(sig.into_stat(stat))
	}

	fn names(item: &ShellItem) -> io::Result<Vec<OsString>> {
		let mut names: Vec<_> = item
			.children()?
			.into_iter()
			.map(|item| item.display_name(SIGDN_PARENTRELATIVEPARSING))
			.collect::<io::Result<_>>()?;
		names.sort_unstable();
		Ok(names)
	}

	fn into_stat(self, mut stat: Stat) -> Stat {
		let mut h = Twox128::default();
		if let Some((count, size)) = self.count_size {
			(size, count).hash(&mut h);
		} else if let Some(names) = self.names {
			names.hash(&mut h);
		} else {
			return stat;
		}

		let hash = h.finish_128();
		stat.ctime = UNIX_EPOCH.checked_add(Duration::from_nanos(hash as u64 ^ (hash >> 64) as u64));
		stat
	}
}
