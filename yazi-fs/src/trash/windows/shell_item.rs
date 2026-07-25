use std::{ffi::{OsStr, OsString, c_void}, io, os::windows::ffi::{OsStrExt, OsStringExt}, path::{Path, PathBuf}};

use windows::{Win32::{Foundation::*, Storage::EnhancedStorage::*, System::{Com::{StructuredStorage::PropVariantToBSTR, *}, SystemServices::*}, UI::Shell::*}, core::{Interface, PCWSTR}};

use super::{super::{TrashCha, TrashEntry, TrashId}, trash::{error, operate, system_time}};
use crate::cha::Cha;

pub(super) struct ShellItem(pub(super) IShellItem);

impl ShellItem {
	pub(super) fn new(name: impl AsRef<OsStr>) -> io::Result<Self> {
		let name: Vec<u16> = name.as_ref().encode_wide().chain([0]).collect();
		unsafe { SHCreateItemFromParsingName(PCWSTR(name.as_ptr()), None) }.map(Self).map_err(error)
	}

	pub(super) fn root() -> io::Result<Self> {
		unsafe { SHGetKnownFolderItem(&FOLDERID_RecycleBinFolder, KF_FLAG_DEFAULT, None) }
			.map(Self)
			.map_err(error)
	}

	pub(super) fn top(name: &Path) -> io::Result<Self> {
		Self::root()?
			.children()?
			.into_iter()
			.find(|item| item.display_name(SIGDN_DESKTOPABSOLUTEPARSING).is_ok_and(|s| s == name))
			.ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "item is not in the recycle bin"))
	}

	pub(super) fn children(&self) -> io::Result<Vec<Self>> {
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

	pub(super) fn cha(&self) -> io::Result<Cha> {
		if let Ok(backing) = self.display_name(SIGDN_FILESYSPATH).map(PathBuf::from) {
			let name = backing.file_name().unwrap_or_default();
			match Cha::from_trash(&backing, name) {
				Ok((_, cha)) => return Ok(cha),
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

	pub(super) fn original(&self) -> io::Result<PathBuf> {
		let item: IShellItem2 = self.0.cast().map_err(error)?;
		let value = unsafe {
			item.GetProperty(&PROPERTYKEY { fmtid: PSGUID_DISPLACED, pid: PID_DISPLACED_FROM })
		}
		.map_err(error)?;

		let parent = unsafe { PropVariantToBSTR(&value) }.map_err(error)?;
		Ok(PathBuf::from(OsString::from_wide(&parent)).join(self.display_name(SIGDN_PARENTRELATIVE)?))
	}

	pub(super) fn entry(&self, id: TrashId, original: Option<PathBuf>) -> io::Result<TrashEntry> {
		let backing: PathBuf = self.display_name(SIGDN_FILESYSPATH)?.into();
		let (lcha, cha) = Cha::from_trash(&backing, backing.file_name().unwrap_or_default())?;
		Ok(TrashEntry { id, cha, lcha, original, link_to: None, backing })
	}

	pub(super) fn delete(&self) -> io::Result<()> {
		operate(FOF_NO_UI, |operation| unsafe { operation.DeleteItem(&self.0, None) })
	}

	pub(super) fn display_name(&self, kind: SIGDN) -> io::Result<OsString> {
		unsafe {
			let name = self.0.GetDisplayName(kind).map_err(error)?;
			let result = OsString::from_wide(name.as_wide());
			CoTaskMemFree(Some(name.0.cast::<c_void>()));
			Ok(result)
		}
	}
}
