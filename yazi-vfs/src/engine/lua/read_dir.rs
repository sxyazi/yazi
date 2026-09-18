use std::io;

use mlua::{FromLua, Lua, Value};
use tokio::sync::mpsc;
use yazi_fs::{engine::{DirReader, FileHolder}, file::File, stat::{Stat, StatType}};
use yazi_shared::{path::PathBufDyn, strand::StrandCow, url::{UrlBuf, UrlLike}};

pub struct ReadDir(pub(super) mpsc::Receiver<io::Result<DirEntry>>);

impl DirReader for ReadDir {
	type Entry = DirEntry;

	async fn next(&mut self) -> io::Result<Option<Self::Entry>> { self.0.recv().await.transpose() }
}

// --- Entry
pub struct DirEntry(File);

impl FromLua for DirEntry {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		Ok(Self(File::from_lua(value, lua)?))
	}
}

impl FileHolder for DirEntry {
	async fn file(&self) -> io::Result<File> { Ok(self.0.clone()) }

	async fn file_type(&self) -> io::Result<StatType> { Ok(**self.0.lstat()) }

	async fn metadata(&self) -> io::Result<Stat> { Ok(self.0.lstat()) }

	fn name(&self) -> StrandCow<'_> { self.0.name().unwrap_or_default().into() }

	fn path(&self) -> PathBufDyn { self.0.url.loc().into() }

	fn url(&self) -> UrlBuf { self.0.url.clone() }
}
