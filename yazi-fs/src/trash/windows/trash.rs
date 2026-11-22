use std::{env, ffi::OsStr, fs, io::{self, ErrorKind, Read}, path::{Path, PathBuf}, time::Duration};

use scopeguard::{ScopeGuard, guard};
use windows::{Win32::{Foundation::*, System::Com::*, UI::Shell::*}, core::PCWSTR};
use yazi_binding::time::Time;
use yazi_ffi::Com;
use yazi_shim::{OsStrExt2, ToWide, bytes::BytesExt, path::PathExt, wtf8::FromWtf8};

use super::{super::{TrashEntries, TrashEntry, TrashId}, shell_item::ShellItem, trash_sig::TrashSig};
use crate::{file::File, stat::Stat, unique_dir};

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

	pub(crate) fn create(&self, name: &OsStr, stat: Stat, original: &Path) -> io::Result<TrashEntry> {
		if !name.is_filename() {
			return Err(io::Error::new(ErrorKind::InvalidInput, "invalid trash entry name"));
		} else if !original.is_absolute() || original.file_name().is_none() {
			return Err(io::Error::new(ErrorKind::InvalidInput, "invalid trash original path"));
		}

		let tmp =
			unique_dir(env::temp_dir().join(format!(".yazi-trash-{:016x}", rand::random::<u64>())))?;
		let tmp = guard(tmp, |p| _ = fs::remove_dir_all(p));

		let path = tmp.join(name);
		if stat.is_dir() {
			fs::create_dir(&path)?;
		} else {
			fs::File::create_new(&path)?;
		}

		let item = guard(ShellItem::new(&path)?.recycle()?, |i| _ = i.delete());
		let id = TrashId::new(item.display_name(SIGDN_PARENTRELATIVEPARSING)?, PathBuf::new())?;

		let entry = item.entry(id, Some(original.into()))?;
		Self::write_info(&entry, original, Some(stat.len))?;

		ScopeGuard::into_inner(item);
		Ok(entry)
	}

	pub(crate) fn list(&self, entry: Option<&TrashEntry>) -> io::Result<Vec<TrashEntry>> {
		let Some(entry) = entry else {
			return self.tops();
		};

		let lstat = entry.lstat()?;
		if !lstat.is_dir() || lstat.is_indirect() {
			return Err(io::Error::new(ErrorKind::InvalidInput, "trash item is not a directory"));
		}

		self
			.resolve(entry)?
			.children()?
			.into_iter()
			.map(|item| {
				let name = item.display_name(SIGDN_PARENTRELATIVEPARSING)?;
				item.entry(entry.id.child(&name)?, entry.original.as_ref().map(|p| p.join(&name)))
			})
			.collect()
	}

	pub(crate) fn entry(&self, id: &TrashId) -> io::Result<TrashEntry> {
		let top = ShellItem::top(id.top())?;
		let original = top.original()?;
		if !id.has_rel() {
			return top.entry(id.clone(), Some(original));
		}

		let original = original.join(id.rel());
		let backing: PathBuf = top.display_name(SIGDN_FILESYSPATH)?.into();

		let stat = Stat::new(backing.file_name().unwrap_or_default(), fs::symlink_metadata(&backing)?);
		if stat.is_dir() && !stat.is_indirect() {
			top.resolve(id.rel())?.entry(id.clone(), Some(original))
		} else {
			Err(io::Error::new(ErrorKind::InvalidInput, "trash item is not a directory"))
		}
	}

	pub(crate) fn revalidate(
		&self,
		entry: Option<&TrashEntry>,
		current: &File,
	) -> io::Result<Option<File>> {
		let latest = if let Some(entry) = entry {
			File { stat: TrashSig::item(&self.resolve(entry)?)?, ..entry.file(&current.url)? }
		} else {
			File { stat: TrashSig::root()?, ..current.clone() }
		};

		Ok(if latest.hits(current) { None } else { Some(latest) })
	}

	pub(crate) fn remove_file(&self, entry: &TrashEntry) -> io::Result<()> {
		self.resolve(entry)?.delete()
	}

	pub(crate) fn remove_dir(&self, entry: &TrashEntry) -> io::Result<()> {
		let item = self.resolve(entry)?;
		if entry.lstat()?.is_indirect() {
			item.delete()
		} else if item.is_empty()? {
			item.delete()
		} else {
			Err(io::Error::new(ErrorKind::DirectoryNotEmpty, "trash directory is not empty"))
		}
	}

	pub(crate) fn restore(&self, entries: TrashEntries) -> io::Result<()> {
		for entry in entries {
			let to = entry.original.as_deref().ok_or_else(|| {
				io::Error::new(ErrorKind::NotFound, "trash item has no put-back location")
			})?;

			self.restore_do(&self.resolve(&entry)?, to)?;
		}
		Ok(())
	}

	fn restore_do(&self, item: &ShellItem, to: &Path) -> io::Result<()> {
		match fs::symlink_metadata(to) {
			Ok(_) => {
				return Err(io::Error::new(
					ErrorKind::AlreadyExists,
					format!("restore target already exists: {to:?}"),
				));
			}
			Err(e) if e.kind() == ErrorKind::NotFound => {}
			Err(e) => return Err(e),
		}

		let (parent, name) =
			to.pair().ok_or_else(|| io::Error::new(ErrorKind::InvalidInput, "invalid restore target"))?;

		fs::create_dir_all(parent)?;
		let parent = ShellItem::new(parent)?;
		let name = name.to_wide();

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

	pub(crate) fn forget(entry: &TrashEntry) -> io::Result<()> {
		if !entry.has_rel() {
			fs::remove_file(Self::info(entry)?)?;
		}
		Ok(())
	}

	fn info(entry: &TrashEntry) -> io::Result<PathBuf> {
		let Some(name) = entry.backing.file_name().filter(|s| s.as_encoded_bytes().starts_with(b"$R"))
		else {
			return Err(io::Error::new(ErrorKind::InvalidInput, "invalid recycle bin item path"));
		};

		let name = name.as_encoded_bytes().replace_seq_once(b"$R", b"$I");
		Ok(entry.backing.with_file_name(OsStr::from_wtf8(&name).expect("valid WTF-8")))
	}

	pub(crate) fn write_info(
		entry: &TrashEntry,
		original: &Path,
		len: Option<u64>,
	) -> io::Result<()> {
		let info = Self::info(entry)?;

		let mut buf = vec![0; 24];
		fs::File::open(&info)?.read_exact(&mut buf)?;

		let mut original = original.to_wide();
		match u64::from_le_bytes(buf[..8].try_into().unwrap()) {
			// Version 1 stores 260 UTF-16 code units including NUL terminator, pad with zeros.
			1 if original.len() <= 260 => original.resize(260, 0),
			// Reject paths that cannot fit in the fixed-size version 1 field.
			1 => Err(io::Error::new(ErrorKind::InvalidInput, "trash original path is too long"))?,
			// Version 2 stores a 4-byte count of UTF-16 code units (including NUL), followed by the path.
			2 => buf.extend_from_slice(&(original.len() as u32).to_le_bytes()),
			_ => Err(io::Error::new(ErrorKind::Unsupported, "unsupported recycle bin metadata version"))?,
		}

		if let Some(len) = len {
			buf[8..16].copy_from_slice(&len.to_le_bytes());
		}

		buf.extend(original.into_iter().flat_map(u16::to_le_bytes));
		fs::write(info, buf)
	}

	fn resolve(&self, entry: &TrashEntry) -> io::Result<ShellItem> {
		let top = ShellItem::top(entry.top())?;
		if entry.has_rel() { top.resolve(entry.rel()) } else { Ok(top) }
	}

	fn tops(&self) -> io::Result<Vec<TrashEntry>> {
		ShellItem::root()?
			.children()?
			.into_iter()
			.map(|item| {
				let top = item.display_name(SIGDN_PARENTRELATIVEPARSING)?;
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
		io::Error::new(ErrorKind::NotFound, error)
	} else {
		io::Error::other(error)
	}
}

pub(super) fn time(time: FILETIME) -> Option<Time> {
	let ticks = (u64::from(time.dwHighDateTime) << 32) | u64::from(time.dwLowDateTime);
	let ticks = ticks.checked_sub(116_444_736_000_000_000)?;
	Time::try_from(Duration::new(ticks / 10_000_000, (ticks % 10_000_000) as u32 * 100)).ok()
}
