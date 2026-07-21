use std::{ffi::{OsStr, OsString, c_void}, fs, hash::Hash, io, mem, os::windows::ffi::{OsStrExt, OsStringExt}, path::{Path, PathBuf}, time::{Duration, SystemTime, UNIX_EPOCH}};

use hashbrown::HashMap;
use trash::{TrashItem, os_limited};
use windows::{Win32::{Foundation::*, Storage::EnhancedStorage::*, System::{Com::*, SystemServices::*}, UI::Shell::*}, core::{Interface, PCWSTR}};
use yazi_ffi::Com;
use yazi_shim::Twox128;

use super::{TrashCha, TrashEntry, TrashNode, TrashNodes};
use crate::{cha::Cha, file::File};

thread_local! {
	static COM: io::Result<Com> = Com::new();
}

pub struct Trash;

impl Trash {
	pub fn new() -> io::Result<Self> {
		COM.with(|result| {
			result.as_ref().map(|_| Self).map_err(|e| io::Error::new(e.kind(), e.to_string()))
		})
	}

	pub fn list(&self, node: Option<&TrashNode>) -> io::Result<Vec<TrashEntry>> {
		let Some(node) = node else {
			return os_limited::list()
				.map_err(io::Error::other)?
				.into_iter()
				.map(|item| ShellItem::new(&item.id)?.entry(item.name, item.id))
				.collect();
		};

		self
			.resolve(node)?
			.children()?
			.into_iter()
			.map(|item| {
				let name = item.display_name(SIGDN_PARENTRELATIVE)?;
				item.entry(name.clone(), name)
			})
			.collect()
	}

	pub fn entry(&self, node: &TrashNode) -> io::Result<TrashEntry> {
		let item = self.resolve(node)?;
		item.entry(item.display_name(SIGDN_PARENTRELATIVE)?, &node.key)
	}

	pub fn metadata(&self, node: &TrashNode, _: bool) -> io::Result<Cha> { self.resolve(node)?.cha() }

	pub(super) fn revalidate(
		&self,
		node: Option<&TrashNode>,
		current: &File,
	) -> io::Result<Option<File>> {
		let cha =
			if let Some(node) = node { TrashSig::item(&self.resolve(node)?)? } else { TrashSig::root()? };

		Ok(if cha.hits(current.cha) { None } else { Some(File { cha, ..current.clone() }) })
	}

	pub fn remove_file(&self, node: &TrashNode) -> io::Result<()> {
		if node.rel.as_os_str().is_empty() {
			os_limited::purge_all([self.top_item(&node.top)?]).map_err(io::Error::other)
		} else {
			self.resolve(node)?.delete()
		}
	}

	pub fn remove_dir(&self, node: &TrashNode) -> io::Result<()> {
		if node.rel.as_os_str().is_empty() {
			return os_limited::purge_all([self.top_item(&node.top)?]).map_err(io::Error::other);
		}

		let item = self.resolve(node)?;
		if !item.children()?.is_empty() {
			return Err(io::Error::new(io::ErrorKind::DirectoryNotEmpty, "trash directory is not empty"));
		}
		item.delete()
	}

	pub fn restore(&self, nodes: TrashNodes) -> io::Result<()> {
		let mut tops = Vec::new();
		let items: HashMap<_, _> = os_limited::list()
			.map_err(io::Error::other)?
			.into_iter()
			.map(|item| (item.id.clone(), item))
			.collect();

		for node in nodes {
			let item = items
				.get(&node.top)
				.ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "trash item no longer exists"))?;
			if node.rel.as_os_str().is_empty() {
				tops.push(item.clone());
			} else {
				let to = item.original_path().join(&node.rel);
				self.restore_do(&self.resolve(&node)?, &to)?;
			}
		}
		os_limited::restore_all(tops).map_err(io::Error::other)
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

	pub fn empty(&self) -> io::Result<()> {
		os_limited::purge_all(os_limited::list().map_err(io::Error::other)?).map_err(io::Error::other)
	}

	fn resolve(&self, node: &TrashNode) -> io::Result<ShellItem> {
		let top = ShellItem::new(&node.top)?;
		if node.rel.as_os_str().is_empty() {
			return Ok(top);
		}

		let path = PathBuf::from(top.display_name(SIGDN_FILESYSPATH)?).join(&node.rel);
		ShellItem::new(path.as_os_str())
	}

	fn top_item(&self, key: &OsStr) -> io::Result<TrashItem> {
		os_limited::list()
			.map_err(io::Error::other)?
			.into_iter()
			.find(|item| item.id == key)
			.ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "trash item no longer exists"))
	}
}

// --- TrashSig
struct TrashSig {
	names:      Option<Vec<OsString>>,
	count_size: Option<(i64, i64)>,
}

impl TrashSig {
	fn root() -> io::Result<Cha> {
		let root = ShellItem::root()?;
		let cha = root.cha()?;

		let mut info =
			SHQUERYRBINFO { cbSize: mem::size_of::<SHQUERYRBINFO>() as u32, ..Default::default() };

		let sig = if unsafe { SHQueryRecycleBinW(PCWSTR::null(), &mut info) }.is_ok() {
			Self { names: None, count_size: Some((info.i64NumItems, info.i64Size)) }
		} else if cha.mtime.is_some() {
			Self { names: None, count_size: None }
		} else {
			Self { names: Some(Self::names(&root)?), count_size: None }
		};

		Ok(sig.into_cha(cha))
	}

	fn item(item: &ShellItem) -> io::Result<Cha> {
		let cha = item.cha()?;
		let sig = if cha.mtime.is_none() && cha.is_dir() {
			Self { names: Some(Self::names(item)?), count_size: None }
		} else {
			Self { names: None, count_size: None }
		};

		Ok(sig.into_cha(cha))
	}

	fn names(item: &ShellItem) -> io::Result<Vec<OsString>> {
		let mut names: Vec<_> = item
			.children()?
			.into_iter()
			.map(|item| item.display_name(SIGDN_DESKTOPABSOLUTEPARSING))
			.collect::<io::Result<_>>()?;
		names.sort_unstable();
		Ok(names)
	}

	fn into_cha(self, mut cha: Cha) -> Cha {
		let mut h = Twox128::default();
		if let Some((count, size)) = self.count_size {
			(size, count).hash(&mut h);
		} else if let Some(names) = self.names {
			names.hash(&mut h);
		} else {
			return cha;
		}

		let hash = h.finish_128();
		cha.ctime = UNIX_EPOCH.checked_add(Duration::from_nanos(hash as u64 ^ (hash >> 64) as u64));
		cha
	}
}

// --- ShellItem
struct ShellItem(IShellItem);

impl ShellItem {
	fn new(name: &OsStr) -> io::Result<Self> {
		let name: Vec<u16> = name.encode_wide().chain([0]).collect();
		unsafe { SHCreateItemFromParsingName(PCWSTR(name.as_ptr()), None) }.map(Self).map_err(error)
	}

	fn root() -> io::Result<Self> {
		unsafe { SHGetKnownFolderItem(&FOLDERID_RecycleBinFolder, KF_FLAG_DEFAULT, None) }
			.map(Self)
			.map_err(error)
	}

	fn children(&self) -> io::Result<Vec<Self>> {
		let items: IEnumShellItems =
			unsafe { self.0.BindToHandler(None, &BHID_EnumItems).map_err(error)? };

		let mut result = Vec::new();
		loop {
			let mut fetched = 0;
			let mut next = [None];
			unsafe { items.Next(&mut next, Some(&mut fetched)).map_err(error)? };
			if fetched == 0 {
				break;
			} else if let Some(item) = next[0].take() {
				result.push(Self(item));
			}
		}
		Ok(result)
	}

	fn cha(&self) -> io::Result<Cha> {
		if let Ok(backing) = self.display_name(SIGDN_FILESYSPATH).map(PathBuf::from) {
			match fs::metadata(&backing) {
				Ok(meta) => return Ok(Cha::new(backing.file_name().unwrap_or_default(), meta)),
				Err(e) if e.kind() == io::ErrorKind::NotFound => return Err(e),
				Err(_) => {}
			}
		}

		let item: IShellItem2 = self.0.cast().map_err(error)?;
		let is_dir = unsafe { self.0.GetAttributes(SFGAO_FOLDER) }.map_err(error)? == SFGAO_FOLDER;

		let mut cha = Cha::from_mold(is_dir);
		cha.len = unsafe { item.GetUInt64(&PKEY_Size) }.unwrap_or_default();
		cha.mtime = unsafe { item.GetFileTime(&PKEY_DateModified) }.ok().and_then(system_time);
		Ok(cha)
	}

	fn entry<N, K>(&self, name: N, key: K) -> io::Result<TrashEntry>
	where
		N: Into<OsString>,
		K: Into<OsString>,
	{
		Ok(TrashEntry {
			name:    name.into(),
			key:     key.into(),
			cha:     self.cha()?,
			link_to: None,
			backing: self.display_name(SIGDN_FILESYSPATH)?.into(),
		})
	}

	fn delete(&self) -> io::Result<()> {
		operate(FOF_NO_UI, |operation| unsafe { operation.DeleteItem(&self.0, None) })
	}

	fn display_name(&self, kind: SIGDN) -> io::Result<OsString> {
		unsafe {
			let name = self.0.GetDisplayName(kind).map_err(error)?;
			let result = OsString::from_wide(name.as_wide());
			CoTaskMemFree(Some(name.0.cast::<c_void>()));
			Ok(result)
		}
	}
}

fn operate<F>(flags: FILEOPERATION_FLAGS, f: F) -> io::Result<()>
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

fn error(error: windows::core::Error) -> io::Error {
	if error.code() == ERROR_FILE_NOT_FOUND.to_hresult()
		|| error.code() == ERROR_PATH_NOT_FOUND.to_hresult()
	{
		io::Error::new(io::ErrorKind::NotFound, error)
	} else {
		io::Error::other(error)
	}
}

fn system_time(time: FILETIME) -> Option<SystemTime> {
	let ticks = (u64::from(time.dwHighDateTime) << 32) | u64::from(time.dwLowDateTime);
	let ticks = ticks.checked_sub(116_444_736_000_000_000)?;
	Some(UNIX_EPOCH + Duration::new(ticks / 10_000_000, (ticks % 10_000_000) as u32 * 100))
}
