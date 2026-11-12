use std::{iter, ops::Range};

use mlua::{AnyUserData, ExternalError, FromLua, Function, Lua, MetaMethod, Table, Value};

use super::FilesInventory;
use crate::file::{File, FileRef};

pub enum FileIter<I = iter::Empty<&'static File>> {
	Iter(I),
	Table(Table),
	Live(AnyUserData),
	Static(AnyUserData),
	Empty,
}

impl<I> FileIter<I> {
	pub fn new(files: I) -> Self { Self::Iter(files) }

	pub fn is_empty(&self) -> mlua::Result<bool>
	where
		I: ExactSizeIterator,
	{
		Ok(match self {
			Self::Iter(it) => it.len() == 0,
			Self::Table(tbl) => tbl.raw_get::<Value>(1)?.is_nil(),
			Self::Live(ud) => {
				ud.metatable()?.get::<Function>(MetaMethod::Len.name())?.call::<usize>(ud)? == 0
			}
			Self::Static(_) => false,
			Self::Empty => true,
		})
	}
}

impl<'a, I: Iterator<Item = &'a File>> FileIter<I> {
	pub fn try_for_each(self, mut visit: impl FnMut(&File) -> mlua::Result<()>) -> mlua::Result<()> {
		self.visit_range(0..usize::MAX, &mut visit)
	}

	/// Visits a single file by its zero-based index, or none if out of bounds.
	pub fn try_visit(
		self,
		index: usize,
		mut visit: impl FnMut(&File) -> mlua::Result<()>,
	) -> mlua::Result<()> {
		self.visit_range(index..index.saturating_add(1), &mut visit)
	}

	fn visit_range(
		self,
		range: Range<usize>,
		visit: &mut dyn FnMut(&File) -> mlua::Result<()>,
	) -> mlua::Result<()> {
		match self {
			Self::Iter(it) => it.skip(range.start).take(range.len()).try_for_each(visit),
			Self::Table(tbl) => {
				for i in range {
					let Some(file) = tbl.raw_get::<Option<FileRef>>(i + 1)? else { break };
					file.borrow(&mut *visit)?;
				}
				Ok(())
			}
			Self::Live(ud) => {
				for inv in inventory::iter::<FilesInventory>() {
					if (inv.visit)(&ud, range.clone(), visit)? {
						return Ok(());
					}
				}
				Err("expected a file collection".into_lua_err())
			}
			Self::Static(ud) if range.contains(&0) => FileRef::from(ud).borrow(visit),
			Self::Static(_) | Self::Empty => Ok(()),
		}
	}
}

impl FromLua for FileIter<iter::Empty<&File>> {
	fn from_lua(files: Value, _: &Lua) -> mlua::Result<Self> {
		match files {
			Value::Nil => Ok(Self::Empty),
			Value::Table(tbl) => Ok(Self::Table(tbl)),
			Value::UserData(ud) => Ok(Self::Live(ud)),
			_ => Err("expected a file array or collection".into_lua_err()),
		}
	}
}
