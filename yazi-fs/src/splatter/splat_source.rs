use std::iter;

use mlua::{AnyUserData, ExternalError, FromLua, Lua, Table, Value};

use super::Splatable;
use crate::{file::{File, FileRef}, files::FileIter};

pub struct SplatSource {
	table: Table,
	tab:   usize,
}

impl SplatSource {
	fn tab(&self, tab: usize) -> mlua::Result<Option<Table>> {
		self.table.raw_get::<Table>("tabs")?.raw_get(tab)
	}
}

impl Splatable for SplatSource {
	fn tab(&self) -> usize { self.tab }

	fn selected(&self, tab: usize) -> mlua::Result<FileIter<impl Iterator<Item = &File>>> {
		let Some(tab) = self.tab(tab)? else { return Ok(FileIter::Empty) };

		let files: FileIter<iter::Empty<&File>> = tab.raw_get("selected")?;
		if !files.is_empty()? {
			return Ok(files);
		}

		Ok(
			tab
				.raw_get::<Option<AnyUserData>>("hovered")?
				.map(FileIter::Static)
				.unwrap_or(FileIter::Empty),
		)
	}

	fn hovered(&self, tab: usize) -> mlua::Result<Option<FileRef<'_>>> {
		self.tab(tab)?.map(|t| t.raw_get("hovered")).transpose().map(Option::flatten)
	}

	fn yanked(&self) -> mlua::Result<FileIter<impl Iterator<Item = &File>>> {
		let files: FileIter<iter::Empty<&File>> = self.table.raw_get("yanked")?;
		Ok(files)
	}
}

impl FromLua for SplatSource {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		let table = Table::from_lua(value, lua)?;

		let tab = table.raw_get("tab").unwrap_or_default();
		if tab == 0 {
			return Err("tab must be greater than 0".into_lua_err());
		}

		Ok(Self { table, tab })
	}
}
