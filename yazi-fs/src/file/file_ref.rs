use std::borrow::Cow;

use mlua::{AnyUserData, ExternalError, FromLua, Lua, Value};

use crate::file::{File, FileInventory};

pub enum FileRef<'a> {
	Raw(&'a File),
	Lua(AnyUserData),
}

const EXPECTED: &str = "expected a table or File";

impl<'a> From<&'a File> for FileRef<'a> {
	fn from(file: &'a File) -> Self { Self::Raw(file) }
}

impl From<AnyUserData> for FileRef<'_> {
	fn from(ud: AnyUserData) -> Self { Self::Lua(ud) }
}

impl TryFrom<FileRef<'_>> for File {
	type Error = mlua::Error;

	fn try_from(value: FileRef<'_>) -> Result<Self, Self::Error> { value.borrow(|f| Ok(f.clone())) }
}

impl<'a> TryFrom<FileRef<'a>> for Cow<'a, File> {
	type Error = mlua::Error;

	fn try_from(value: FileRef<'a>) -> Result<Self, Self::Error> {
		match value {
			FileRef::Raw(file) => Ok(Cow::Borrowed(file)),
			value => File::try_from(value).map(Cow::Owned),
		}
	}
}

impl FileRef<'_> {
	pub fn borrow<R>(&self, mut f: impl FnMut(&File) -> mlua::Result<R>) -> mlua::Result<R> {
		let ud = match self {
			Self::Raw(file) => return f(file),
			Self::Lua(ud) => ud,
		};

		let mut result = None;
		for inv in inventory::iter::<FileInventory>() {
			match (inv.borrow)(ud, &mut |file| Ok(result = Some(f(file)?))) {
				Ok(()) => return Ok(result.unwrap()),
				Err(mlua::Error::UserDataTypeMismatch) => continue,
				Err(e) => return Err(e),
			}
		}

		f(&*ud.borrow::<File>()?)
	}
}

impl FromLua for FileRef<'_> {
	fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
		match value {
			Value::UserData(ud) => Ok(ud.into()),
			_ => Err(EXPECTED.into_lua_err())?,
		}
	}
}
