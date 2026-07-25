use std::{fs::{self}, hash::Hash, io, path::Path};

use trash::os_limited;
use yazi_macro::ok_or_not_found;
use yazi_shim::Twox128;

use super::{super::{TrashCha, TrashEntries, TrashEntry, TrashId, restore_item}, TrashInfo};
use crate::{cha::{Cha, ChaSig}, file::File};

pub struct Trash;

impl Trash {
	pub(crate) fn new() -> io::Result<Self> { Ok(Self) }

	pub(crate) fn list(&self, entry: Option<&TrashEntry>) -> io::Result<Vec<TrashEntry>> {
		let Some(entry) = entry else {
			return self.tops();
		};

		// TODO
		if !entry.lcha.is_dir() {
			return Err(io::Error::new(io::ErrorKind::InvalidInput, "trash item is not a directory"));
		}

		fs::read_dir(&entry.backing)?
			.map(|dent| {
				let dent = dent?;
				entry.child(dent.file_name())
			})
			.collect()
	}

	pub(crate) fn entry(&self, id: &TrashId) -> io::Result<TrashEntry> {
		let info = TrashInfo::parse(id.top())?;
		if !os_limited::trash_folders()
			.map_err(io::Error::other)?
			.iter()
			.any(|folder| folder == &info.root)
		{
			return Err(io::Error::new(io::ErrorKind::NotFound, "trash item outside of trash folders"));
		}

		if id.has_rel() && !fs::symlink_metadata(&info.backing)?.file_type().is_dir() {
			return Err(io::Error::new(io::ErrorKind::InvalidInput, "trash item is not a directory"));
		}

		let backing = info.backing.join(id.rel());
		TrashEntry::new(id.clone(), backing, Some(info.original.join(id.rel())))
	}

	pub(crate) fn metadata(&self, entry: &TrashEntry, follow: bool) -> io::Result<Cha> {
		Ok(if follow { entry.cha } else { entry.lcha })
	}

	pub(crate) fn revalidate(
		&self,
		entry: Option<&TrashEntry>,
		current: &File,
	) -> io::Result<Option<File>> {
		let latest = if let Some(entry) = entry {
			entry.clone().into_file(&current.url)
		} else {
			let mut roots: Vec<_> = os_limited::trash_folders().unwrap_or_default().into_iter().collect();
			roots.sort_unstable();

			let mut h = Twox128::default();
			for root in roots {
				let meta = ok_or_not_found!(fs::metadata(root.join("info")), continue);
				let cha = Cha::new(root.file_name().unwrap_or_default(), meta);

				root.hash(&mut h);
				ChaSig(cha).hash(&mut h);
			}

			let hash = h.finish_128();
			File {
				cha: Cha { len: hash as u64 ^ (hash >> 64) as u64, ..Cha::from_mold(true) },
				..current.clone()
			}
		};

		let changed = !latest.cha.hits(current.cha)
			|| latest.extra.link_to() != current.extra.link_to()
			|| latest.extra.backing() != current.extra.backing();

		Ok(changed.then_some(latest))
	}

	pub(crate) fn remove_file(&self, entry: &TrashEntry) -> io::Result<()> {
		fs::remove_file(&entry.backing)?;
		if !entry.has_rel() {
			fs::remove_file(entry.top())?;
		}
		Ok(())
	}

	pub(crate) fn remove_dir(&self, entry: &TrashEntry) -> io::Result<()> {
		fs::remove_dir(&entry.backing)?;
		if !entry.has_rel() {
			fs::remove_file(entry.top())?;
		}
		Ok(())
	}

	pub(crate) fn restore(&self, entries: TrashEntries) -> io::Result<()> {
		for entry in entries {
			let to = entry.original.clone().ok_or_else(|| {
				io::Error::new(io::ErrorKind::NotFound, "trash item has no put-back location")
			})?;

			restore_item(&entry.backing, &to)?;

			if !entry.has_rel() {
				fs::remove_file(entry.top())?;
			}
		}
		Ok(())
	}

	// FIXME: also rename or remove the .trashinfo file in the info folder
	pub(crate) fn rename(&self, entry: &TrashEntry, path: &Path) -> io::Result<()> {
		fs::rename(&entry.backing, path)
	}

	pub(crate) fn empty(&self) -> io::Result<()> {
		for entry in self.tops()? {
			if entry.lcha.is_dir() {
				fs::remove_dir_all(&entry.backing)?;
			} else {
				fs::remove_file(&entry.backing)?;
			}
			fs::remove_file(entry.top())?;
		}
		Ok(())
	}

	fn tops(&self) -> io::Result<Vec<TrashEntry>> {
		let mut tops = Vec::new();
		for root in os_limited::trash_folders().map_err(io::Error::other)? {
			for dent in ok_or_not_found!(fs::read_dir(root.join("info")), continue) {
				let dent = dent?;
				let info = dent.path();
				if let Ok(parsed) = TrashInfo::parse(&info) {
					tops.push(TrashEntry::top(info, parsed.backing, Some(parsed.original))?);
				}
			}
		}
		Ok(tops)
	}
}
