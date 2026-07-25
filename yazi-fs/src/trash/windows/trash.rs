use std::{fs, io, os::windows::ffi::OsStrExt, path::{Path, PathBuf}, time::{Duration, SystemTime, UNIX_EPOCH}};

use windows::{Win32::{Foundation::*, System::Com::*, UI::Shell::*}, core::PCWSTR};
use yazi_ffi::Com;

use super::{super::{TrashEntries, TrashEntry, TrashId}, shell_item::ShellItem, trash_sig::TrashSig};
use crate::{cha::Cha, file::File};

thread_local! {
	static COM: io::Result<Com> = Com::new();
}

pub struct Trash;

impl Trash {
	pub(crate) fn new() -> io::Result<Self> {
		COM.with(|result| {
			result.as_ref().map(|_| Self).map_err(|e| io::Error::new(e.kind(), e.to_string()))
		})
	}

	pub(crate) fn list(&self, entry: Option<&TrashEntry>) -> io::Result<Vec<TrashEntry>> {
		let Some(entry) = entry else {
			return self.tops();
		};

		if !entry.lcha.is_dir() || entry.lcha.is_indirect() {
			return Err(io::Error::new(io::ErrorKind::InvalidInput, "trash item is not a directory"));
		}

		let original = entry.original.as_deref().ok_or_else(|| {
			io::Error::new(io::ErrorKind::NotFound, "trash item has no put-back location")
		})?;

		self
			.resolve(entry)?
			.children()?
			.into_iter()
			.map(|item| {
				let name = item.display_name(SIGDN_PARENTRELATIVE)?;
				item.entry(entry.id.child(&name)?, Some(original.join(&name)))
			})
			.collect()
	}

	pub(crate) fn entry(&self, id: &TrashId) -> io::Result<TrashEntry> {
		let top = ShellItem::top(id.top())?;
		let original = top.original()?.join(id.rel());
		if !id.has_rel() {
			return top.entry(id.clone(), Some(original));
		}

		let backing: PathBuf = top.display_name(SIGDN_FILESYSPATH)?.into();
		let cha = Cha::new(backing.file_name().unwrap_or_default(), fs::symlink_metadata(&backing)?);
		if cha.is_dir() && !cha.is_indirect() {
			ShellItem::new(backing.join(id.rel()))?.entry(id.clone(), Some(original))
		} else {
			Err(io::Error::new(io::ErrorKind::InvalidInput, "trash item is not a directory"))
		}
	}

	pub(crate) fn metadata(&self, entry: &TrashEntry, follow: bool) -> io::Result<Cha> {
		Ok(if follow { entry.cha } else { entry.lcha })
	}

	pub(crate) fn revalidate(
		&self,
		entry: Option<&TrashEntry>,
		current: &File,
	) -> io::Result<Option<File>> {
		let cha = if let Some(entry) = entry {
			TrashSig::item(&self.resolve(entry)?)?
		} else {
			TrashSig::root()?
		};

		Ok(if cha.hits(current.cha) { None } else { Some(File { cha, ..current.clone() }) })
	}

	pub(crate) fn remove_file(&self, entry: &TrashEntry) -> io::Result<()> {
		self.resolve(entry)?.delete()
	}

	pub(crate) fn remove_dir(&self, entry: &TrashEntry) -> io::Result<()> {
		let item = self.resolve(entry)?;
		if !entry.has_rel() {
			item.delete()
		} else if !item.children()?.is_empty() {
			Err(io::Error::new(io::ErrorKind::DirectoryNotEmpty, "trash directory is not empty"))
		} else {
			item.delete()
		}
	}

	pub(crate) fn restore(&self, entries: TrashEntries) -> io::Result<()> {
		for entry in entries {
			let to = entry.original.as_deref().ok_or_else(|| {
				io::Error::new(io::ErrorKind::NotFound, "trash item has no put-back location")
			})?;

			self.restore_do(&self.resolve(&entry)?, &to)?;
		}
		Ok(())
	}

	pub(crate) fn rename(&self, entry: &TrashEntry, path: &Path) -> io::Result<()> {
		fs::rename(self.resolve(entry)?.display_name(SIGDN_FILESYSPATH)?, path)
	}

	fn restore_do(&self, item: &ShellItem, to: &Path) -> io::Result<()> {
		match fs::symlink_metadata(to) {
			Ok(_) => {
				return Err(io::Error::new(
					io::ErrorKind::AlreadyExists,
					format!("restore target already exists: {to:?}"),
				));
			}
			Err(e) if e.kind() == io::ErrorKind::NotFound => {}
			Err(e) => return Err(e),
		}

		// Create parent directories
		let parent = to
			.parent()
			.ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid restore target"))?;
		fs::create_dir_all(parent)?;

		let parent = ShellItem::new(parent.as_os_str())?;
		let name: Vec<u16> = to
			.file_name()
			.ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid restore target"))?
			.encode_wide()
			.chain([0])
			.collect();

		operate(FOF_NO_UI | FOFX_EARLYFAILURE, |operation| unsafe {
			operation.MoveItem(&item.0, &parent.0, PCWSTR(name.as_ptr()), None)
		})
	}

	pub(crate) fn empty(&self) -> io::Result<()> {
		unsafe {
			SHEmptyRecycleBinW(
				None,
				PCWSTR::null(),
				SHERB_NOCONFIRMATION | SHERB_NOPROGRESSUI | SHERB_NOSOUND,
			)
		}
		.map_err(error)
	}

	fn resolve(&self, entry: &TrashEntry) -> io::Result<ShellItem> {
		if entry.has_rel() { ShellItem::new(&entry.backing) } else { ShellItem::top(entry.top()) }
	}

	fn tops(&self) -> io::Result<Vec<TrashEntry>> {
		ShellItem::root()?
			.children()?
			.into_iter()
			.map(|item| {
				let top = item.display_name(SIGDN_DESKTOPABSOLUTEPARSING)?;
				let original = item.original()?;
				item.entry(TrashId::new(top, PathBuf::new())?, Some(original))
			})
			.collect()
	}
}

pub(super) fn operate<F>(flags: FILEOPERATION_FLAGS, f: F) -> io::Result<()>
where
	F: FnOnce(&IFileOperation) -> windows::core::Result<()>,
{
	let aborted = unsafe {
		let operation: IFileOperation =
			CoCreateInstance(&FileOperation, None, CLSCTX_ALL).map_err(error)?;
		operation.SetOperationFlags(flags).map_err(error)?;
		f(&operation).map_err(error)?;
		operation.PerformOperations().map_err(error)?;
		operation.GetAnyOperationsAborted().map_err(error)?.as_bool()
	};

	if aborted { Err(io::Error::other("trash operation was aborted")) } else { Ok(()) }
}

pub(super) fn error(error: windows::core::Error) -> io::Error {
	if error.code() == ERROR_FILE_NOT_FOUND.to_hresult()
		|| error.code() == ERROR_PATH_NOT_FOUND.to_hresult()
	{
		io::Error::new(io::ErrorKind::NotFound, error)
	} else {
		io::Error::other(error)
	}
}

pub(super) fn system_time(time: FILETIME) -> Option<SystemTime> {
	let ticks = (u64::from(time.dwHighDateTime) << 32) | u64::from(time.dwLowDateTime);
	let ticks = ticks.checked_sub(116_444_736_000_000_000)?;
	Some(UNIX_EPOCH + Duration::new(ticks / 10_000_000, (ticks % 10_000_000) as u32 * 100))
}
