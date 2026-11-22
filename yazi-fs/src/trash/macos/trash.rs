use std::{ffi::OsStr, fs::{self, OpenOptions}, io::{self, ErrorKind}, os::unix::fs::DirBuilderExt, path::{Path, PathBuf}};

use scopeguard::{ScopeGuard, guard};
use yazi_macro::ok_or_not_found;
use yazi_shim::{OptionExt, OsStrExt2, path::PathExt};

use super::{super::{TrashEntries, TrashEntry, TrashId, TrashStat}, DsStore};
use crate::{file::File, stat::Stat, unique_dir, unique_file};

pub struct Trash;

impl Trash {
	pub(crate) fn new() -> io::Result<Self> { Ok(Self) }

	pub(crate) fn create(&self, name: &OsStr, stat: Stat, original: &Path) -> io::Result<TrashEntry> {
		if !name.is_filename() {
			return Err(io::Error::new(ErrorKind::InvalidInput, "invalid trash entry name"));
		}

		let root = self.root()?;
		fs::DirBuilder::new().recursive(true).mode(0o700).create(&root)?;

		let name = name
			.to_str()
			.ok_or_else(|| io::Error::new(ErrorKind::InvalidInput, "invalid trash entry name"))?;

		let backing = if DsStore::is_internal(name.as_ref()) {
			root.join(format!("{name}_1"))
		} else {
			root.join(name)
		};

		let backing = if stat.is_dir() {
			unique_dir(backing)?
		} else {
			unique_file(backing, OpenOptions::new().write(true))?.0
		};

		let entry = TrashEntry::top(backing.clone(), backing, original.non_empty().owned())?;
		let entry =
			guard(entry, |e| _ = if stat.is_dir() { self.remove_dir(&e) } else { self.remove_file(&e) });

		Self::write_info(&entry, original, None)?;
		Ok(ScopeGuard::into_inner(entry))
	}

	pub(crate) fn list(&self, entry: Option<&TrashEntry>) -> io::Result<Vec<TrashEntry>> {
		if let Some(ent) = entry
			&& !ent.file_type()?.is_dir()
		{
			return Err(io::Error::new(ErrorKind::InvalidInput, "trash item is not a directory"));
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
			Err(e) if e.kind() == ErrorKind::NotFound && entry.is_none() => return Ok(vec![]),
			Err(e) => return Err(e),
		};

		let mut entries = Vec::new();
		for dent in it {
			let dent = dent?;
			if let Some(entry) = entry {
				entries.push(entry.child(dent.file_name())?);
				continue;
			}

			let path = dent.path();
			let name = path.file_name().unwrap_or_default();
			if DsStore::is_internal(name) {
				continue;
			}

			let original = store.get(name).and_then(|ds| ds.join(Path::new("")).ok());
			entries.push(TrashEntry::top(path.clone(), path, original)?);
		}
		Ok(entries)
	}

	pub(crate) fn entry(&self, id: &TrashId) -> io::Result<TrashEntry> {
		let root = self.root()?;
		if id.top().parent() != Some(&root) {
			return Err(io::Error::new(ErrorKind::InvalidInput, "item is not in the trash"));
		}
		if id.has_rel() && !fs::symlink_metadata(id.top())?.file_type().is_dir() {
			return Err(io::Error::new(ErrorKind::InvalidInput, "trash item is not a directory"));
		}

		let store = DsStore::parse(&root.join(".DS_Store")).unwrap_or_default();
		let name = id.top().file_name().unwrap_or_default();
		let original = store.get(name).and_then(|ds| ds.join(id.rel()).ok());

		TrashEntry::new(id.clone(), id.path(), original)
	}

	pub(crate) fn revalidate(
		&self,
		entry: Option<&TrashEntry>,
		current: &File,
	) -> io::Result<Option<File>> {
		let latest = if let Some(entry) = entry {
			entry.file(&current.url)?
		} else {
			let path = self.root()?;
			let stat = match fs::symlink_metadata(&path) {
				Ok(meta) => Stat::new(path.file_name().unwrap_or_default(), meta),
				Err(e) if e.kind() == ErrorKind::NotFound => Stat::from_mold(true),
				Err(e) => return Err(e),
			};
			File { stat, ..current.clone() }
		};

		let changed = !latest.hits(current)
			|| latest.extra.link_to() != current.extra.link_to()
			|| latest.extra.backing() != current.extra.backing();

		Ok(changed.then_some(latest))
	}

	pub(crate) fn remove_file(&self, entry: &TrashEntry) -> io::Result<()> {
		fs::remove_file(&entry.backing)?;
		Self::forget(entry)
	}

	pub(crate) fn remove_dir(&self, entry: &TrashEntry) -> io::Result<()> {
		fs::remove_dir(&entry.backing)?;
		Self::forget(entry)
	}

	pub(crate) fn restore(&self, entries: TrashEntries) -> io::Result<()> {
		for entry in entries {
			let to = entry.original.as_ref().ok_or_else(|| {
				io::Error::new(ErrorKind::NotFound, "trash item has no put-back location")
			})?;

			Self::restore_item(&entry.backing, to)?;
		}
		Ok(())
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

	pub(crate) fn forget(entry: &TrashEntry) -> io::Result<()> {
		// .DS_Store record names are Unicode, so non-UTF-8 filenames have no matching record.
		if !entry.has_rel()
			&& let Some((root, name)) = entry.backing.pair()
			&& let Some(name) = name.to_str()
		{
			DsStore::set(root, name, Path::new(""))?;
		}
		Ok(())
	}

	pub(crate) fn write_info(
		entry: &TrashEntry,
		original: &Path,
		_len: Option<u64>,
	) -> io::Result<()> {
		let Some((root, name)) = entry.backing.pair() else {
			return Err(io::Error::new(ErrorKind::InvalidData, "invalid trash entry path"));
		};

		let name = name
			.to_str()
			.ok_or_else(|| io::Error::new(ErrorKind::InvalidInput, "invalid trash entry name"))?;

		DsStore::set(root, name, original)
	}

	fn root(&self) -> io::Result<PathBuf> {
		dirs::home_dir()
			.filter(|p| p.is_absolute())
			.ok_or_else(|| io::Error::other("cannot determine home directory for trash root resolution"))
			.map(|home| home.join(".Trash"))
	}
}
