use std::borrow::Cow;

use mlua::{AnyUserData, ExternalError, FromLua, Lua, Value};

use crate::file::{File, FileInventory};

pub struct FileRef(pub(super) AnyUserData);

const EXPECTED: &str = "expected a table or File";

impl TryFrom<FileRef> for File {
	type Error = mlua::Error;

	fn try_from(value: FileRef) -> Result<Self, Self::Error> { value.borrow(|f| Ok(f.clone())) }
}

impl<'a> TryFrom<FileRef> for Cow<'a, File> {
	type Error = mlua::Error;

	fn try_from(value: FileRef) -> Result<Self, Self::Error> { File::try_from(value).map(Cow::Owned) }
}

impl FileRef {
	pub fn borrow<R>(&self, mut f: impl FnMut(&File) -> mlua::Result<R>) -> mlua::Result<R> {
		let mut result = None;
		for inv in inventory::iter::<FileInventory>() {
			match (inv.borrow)(&self.0, &mut |file| Ok(result = Some(f(file)?))) {
				Ok(()) => return Ok(result.unwrap()),
				Err(mlua::Error::UserDataTypeMismatch) => continue,
				Err(e) => return Err(e),
			}
		}

		match self.0.borrow::<File>() {
			Ok(file) => f(&file),
			Err(mlua::Error::UserDataTypeMismatch) => Err(mlua::Error::UserDataTypeMismatch),
			Err(e) => Err(e),
		}
	}
}

impl FromLua for FileRef {
	fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
		match value {
			Value::UserData(ud) => Ok(Self(ud)),
			_ => Err(EXPECTED.into_lua_err())?,
		}
	}
}
