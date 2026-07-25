use std::{fs, io, path::{Path, PathBuf}};

use yazi_macro::ok_or_not_found;

use super::{super::{TrashCha, TrashEntries, TrashEntry, TrashId, restore_item}, DsStore};
use crate::{cha::Cha, file::File};

pub struct Trash;

impl Trash {
	pub(crate) fn new() -> io::Result<Self> { Ok(Self) }

	pub(crate) fn list(&self, entry: Option<&TrashEntry>) -> io::Result<Vec<TrashEntry>> {
		if entry.is_some_and(|entry| !entry.lcha.is_dir()) {
			return Err(io::Error::new(io::ErrorKind::InvalidInput, "trash item is not a directory"));
		}

		let root = self.root()?;
		let store = if entry.is_none() {
			DsStore::parse(&root.join(".DS_Store")).unwrap_or_default()
		} else {
			Default::default()
		};

		let path = entry.map_or(&root, |entry| &entry.backing);
		let it = match fs::read_dir(path) {
			Ok(it) => it,
			Err(e) if e.kind() == io::ErrorKind::NotFound && entry.is_none() => return Ok(vec![]),
			Err(e) => return Err(e),
		};

		it.map(|dent| {
			let dent = dent?;
			if let Some(entry) = entry {
				entry.child(dent.file_name())
			} else {
				let path = dent.path();
				let original = store
					.get(path.file_name().unwrap_or_default())
					.and_then(|ds| ds.join(Path::new("")).ok());
				TrashEntry::top(path.clone(), path, original)
			}
		})
		.collect()
	}

	pub(crate) fn entry(&self, id: &TrashId) -> io::Result<TrashEntry> {
		let root = self.root()?;
		if id.top().parent() != Some(&root) {
			return Err(io::Error::new(io::ErrorKind::InvalidInput, "item is not in the trash"));
		}
		if id.has_rel() && !fs::symlink_metadata(id.top())?.file_type().is_dir() {
			return Err(io::Error::new(io::ErrorKind::InvalidInput, "trash item is not a directory"));
		}

		let store = DsStore::parse(&root.join(".DS_Store")).unwrap_or_default();
		let name = id.top().file_name().unwrap_or_default();
		let original = store.get(name).and_then(|ds| ds.join(id.rel()).ok());

		TrashEntry::new(id.clone(), id.path(), original)
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
			let path = self.root()?;
			let cha = match fs::symlink_metadata(&path) {
				Ok(meta) => Cha::new(path.file_name().unwrap_or_default(), meta),
				Err(e) if e.kind() == io::ErrorKind::NotFound => Cha::from_mold(true),
				Err(e) => return Err(e),
			};
			File { cha, ..current.clone() }
		};

		let changed = !latest.cha.hits(current.cha)
			|| latest.extra.link_to() != current.extra.link_to()
			|| latest.extra.backing() != current.extra.backing();

		Ok(changed.then_some(latest))
	}

	pub(crate) fn remove_file(&self, entry: &TrashEntry) -> io::Result<()> {
		fs::remove_file(&entry.backing)
	}

	pub(crate) fn remove_dir(&self, entry: &TrashEntry) -> io::Result<()> {
		fs::remove_dir(&entry.backing)
	}

	pub(crate) fn restore(&self, entries: TrashEntries) -> io::Result<()> {
		for entry in entries {
			let to = entry.original.as_ref().ok_or_else(|| {
				io::Error::new(io::ErrorKind::NotFound, "trash item has no put-back location")
			})?;

			restore_item(&entry.backing, &to)?;
		}
		Ok(())
	}

	pub(crate) fn rename(&self, entry: &TrashEntry, path: &Path) -> io::Result<()> {
		fs::rename(&entry.backing, path)
	}

	pub(crate) fn empty(&self) -> io::Result<()> {
		let root = self.root()?;
		for dent in ok_or_not_found!(fs::read_dir(root), return Ok(())) {
			let dent = dent?;
			if dent.file_type()?.is_dir() {
				fs::remove_dir_all(dent.path())?;
			} else {
				fs::remove_file(dent.path())?;
			}
		}
		Ok(())
	}

	fn root(&self) -> io::Result<PathBuf> {
		dirs::home_dir()
			.filter(|p| p.is_absolute())
			.ok_or_else(|| io::Error::other("cannot determine home directory for trash root resolution"))
			.map(|home| home.join(".Trash"))
	}
}
