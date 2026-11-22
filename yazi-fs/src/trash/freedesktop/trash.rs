use std::{env, ffi::OsStr, fs::{self, DirBuilder, OpenOptions}, hash::Hash, io::{self, ErrorKind, Write}, os::unix::{ffi::OsStrExt, fs::{DirBuilderExt, MetadataExt, OpenOptionsExt}}, path::{Path, PathBuf}};

use percent_encoding::percent_encode;
use scopeguard::{ScopeGuard, guard};
use trash::os_limited;
use yazi_macro::ok_or_not_found;
use yazi_shim::{OsStrExt2, Twox128, Uzers, path::OsPathExt};

use super::{super::{TrashEntries, TrashEntry, TrashId, TrashStat}, trash_info::{ENCODE, TrashInfo}};
use crate::{file::File, mounts::Partitions, stat::{Stat, StatSig}, unique_file};

pub struct Trash;

impl Trash {
	pub(crate) fn new() -> io::Result<Self> { Ok(Self) }

	pub(crate) fn create(&self, name: &OsStr, stat: Stat, original: &Path) -> io::Result<TrashEntry> {
		if !name.is_filename() {
			return Err(io::Error::new(ErrorKind::InvalidInput, "invalid trash entry name"));
		}

		let Some((parent, original_name)) = original.pair().filter(|_| original.is_absolute()) else {
			return Err(io::Error::new(ErrorKind::InvalidInput, "invalid trash original path"));
		};

		// Resolve parent links while allowing a missing put-back directory.
		let original = parent.canonicalize_partial()?.join(original_name);
		let home = [("XDG_DATA_HOME", "Trash"), ("HOME", ".local/share/Trash")]
			.into_iter()
			.find_map(|(key, suffix)| {
				env::var_os(key).filter(|s| !s.is_empty()).map(|p| PathBuf::from(p).join(suffix))
			})
			.ok_or_else(|| io::Error::other("cannot determine trash directory"))?;

		self.create_on_volume(&home, name, stat, &original)
	}

	fn create_on_volume(
		&self,
		home: &Path,
		name: &OsStr,
		stat: Stat,
		original: &Path,
	) -> io::Result<TrashEntry> {
		let mounts = Partitions::mounts()?;

		// Same mount: use the home trash root.
		// /home/alice/Downloads/cat.jpg -> /home/alice/.local/share/Trash
		let mp = mounts.mount_point(original).unwrap_or(Path::new("/"));
		if mp == mounts.mount_point(home.canonicalize_partial()?).unwrap_or(Path::new("/")) {
			return self.create_in(home, name, stat, original);
		}

		let uid = Uzers::uid();
		let root = if let parent = mp.join(".Trash")
			&& let Ok(meta) = fs::symlink_metadata(&parent)
			&& meta.is_dir()
			&& meta.mode() & 0o1000 != 0
			&& let child = parent.join(uid.to_string())
			&& child.is_dir()
		{
			// Cross-mount: use the existing user directory in the shared trash root.
			// /mnt/disk/cat.jpg -> /mnt/disk/.Trash/1000
			child
		} else {
			// No usable shared user trash: use this mount's private trash root.
			// /mnt/disk/cat.jpg -> /mnt/disk/.Trash-1000
			mp.join(format!(".Trash-{uid}"))
		};

		match self.create_in(&root, name, stat, original) {
			// Permission denied preparing the per-mount trash: retry at the home trash root.
			// /mnt/disk/cat.jpg -> /home/alice/.local/share/Trash
			Err(e) if e.kind() == ErrorKind::PermissionDenied => {
				self.create_in(home, name, stat, original)
			}
			result => result,
		}
	}

	fn create_in(
		&self,
		root: &Path,
		name: &OsStr,
		stat: Stat,
		original: &Path,
	) -> io::Result<TrashEntry> {
		for dir in [root.join("files"), root.join("info")] {
			DirBuilder::new().recursive(true).mode(0o700).create(dir)?;
		}

		let mut info = root.join("info").join(name);
		info.add_extension("trashinfo");

		let mut skipped = Vec::new();
		loop {
			let (info, mut file) = unique_file(info.clone(), OpenOptions::new().write(true).mode(0o600))?;
			let backing = root.join("files").join(info.file_stem().unwrap());

			let entry = TrashEntry::top(info, backing, Some(original.into()))?;
			let entry = guard(entry, |ent| _ = fs::remove_file(ent.top()));

			writeln!(
				file,
				"[Trash Info]\nPath={}\nDeletionDate={}",
				percent_encode(original.as_os_str().as_bytes(), ENCODE),
				chrono::Local::now().format("%Y-%m-%dT%H:%M:%S")
			)?;

			match if stat.is_dir() {
				fs::create_dir(&entry.backing)
			} else {
				fs::File::create_new(&entry.backing).map(drop)
			} {
				Ok(()) => return Ok(ScopeGuard::into_inner(entry)),
				Err(e) if e.kind() == ErrorKind::AlreadyExists => skipped.push(entry),
				Err(e) => return Err(e),
			}
		}
	}

	pub(crate) fn list(&self, entry: Option<&TrashEntry>) -> io::Result<Vec<TrashEntry>> {
		let Some(entry) = entry else {
			return self.tops();
		};

		if !entry.file_type()?.is_dir() {
			return Err(io::Error::new(ErrorKind::InvalidInput, "trash item is not a directory"));
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
		if !os_limited::trash_folders().map_err(io::Error::other)?.contains(&info.root) {
			return Err(io::Error::new(ErrorKind::NotFound, "trash item outside of trash folders"));
		}

		if id.has_rel() && !fs::symlink_metadata(&info.backing)?.file_type().is_dir() {
			return Err(io::Error::new(ErrorKind::InvalidInput, "trash item is not a directory"));
		}

		let (backing, original) = if id.has_rel() {
			(info.backing.join(id.rel()), info.original.join(id.rel()))
		} else {
			(info.backing, info.original)
		};
		TrashEntry::new(id.clone(), backing, Some(original))
	}

	pub(crate) fn revalidate(
		&self,
		entry: Option<&TrashEntry>,
		current: &File,
	) -> io::Result<Option<File>> {
		let latest = if let Some(entry) = entry {
			entry.file(&current.url)?
		} else {
			let mut roots: Vec<_> = os_limited::trash_folders().unwrap_or_default().into_iter().collect();
			roots.sort_unstable();

			let mut h = Twox128::default();
			for root in roots {
				let meta = ok_or_not_found!(fs::metadata(root.join("info")), continue);
				let stat = Stat::new(root.file_name().unwrap_or_default(), meta);

				root.hash(&mut h);
				StatSig(stat).hash(&mut h);
			}

			let hash = h.finish_128();
			File {
				stat: Stat { len: hash as u64 ^ (hash >> 64) as u64, ..Stat::from_mold(true) },
				..current.clone()
			}
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
			let to = entry.original.as_deref().ok_or_else(|| {
				io::Error::new(ErrorKind::NotFound, "trash item has no put-back location")
			})?;

			Self::restore_item(&entry.backing, to)?;

			Self::forget(&entry)?;
		}
		Ok(())
	}

	pub(crate) fn empty(&self) -> io::Result<()> {
		for entry in self.tops()? {
			if entry.file_type()?.is_dir() {
				fs::remove_dir_all(&entry.backing)?;
			} else {
				fs::remove_file(&entry.backing)?;
			}
			fs::remove_file(entry.top())?;
		}
		Ok(())
	}

	pub(crate) fn forget(entry: &TrashEntry) -> io::Result<()> {
		if !entry.has_rel() {
			fs::remove_file(entry.top())?;
		}
		Ok(())
	}

	pub(crate) fn write_info(
		entry: &TrashEntry,
		original: &Path,
		_len: Option<u64>,
	) -> io::Result<()> {
		TrashInfo::set_original(entry.top(), original)
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
