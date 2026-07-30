use std::{io, vec};

use mlua::{FromLua, Lua, Table, Value};
use yazi_fs::{cha::{Cha, ChaType}, engine::{DirReader, FileHolder}, file::File};
use yazi_shared::{path::PathBufDyn, strand::StrandCow, url::{UrlBuf, UrlLike}};

pub struct ReadDir {
	pub(super) entries: vec::IntoIter<DirEntry>,
}

impl DirReader for ReadDir {
	type Entry = DirEntry;

	async fn next(&mut self) -> io::Result<Option<Self::Entry>> { Ok(self.entries.next()) }
}

// --- Entry
pub struct DirEntry {
	file: File,
	cha:  Cha,
}

impl FromLua for DirEntry {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		let t = Table::from_lua(value, lua)?;
		Ok(Self { file: t.raw_get("file")?, cha: t.raw_get("cha")? })
	}
}

impl FileHolder for DirEntry {
	async fn file(&self) -> io::Result<File> { Ok(self.file.clone()) }

	async fn file_type(&self) -> io::Result<ChaType> { Ok(**self.cha) }

	async fn metadata(&self) -> io::Result<Cha> { Ok(self.cha) }

	fn name(&self) -> StrandCow<'_> { self.file.name().unwrap_or_default().into() }

	fn path(&self) -> PathBufDyn { self.file.url.loc().into() }

	fn url(&self) -> UrlBuf { self.file.url.clone() }
}
