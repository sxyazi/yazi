use mlua::{AnyUserData, ExternalError, FromLua, Lua, Table, UserData, UserDataMethods, UserDataRegistry, Value};
use yazi_binding::{impl_file_fields, impl_file_methods};

use crate::file::{File, FileInventory, FileRef};

const EXPECTED: &str = "expected a table, File, or fs::File";

impl File {
	pub fn install(lua: &Lua) -> mlua::Result<()> {
		lua.globals().raw_set("File", lua.create_function(|_, file: Self| Ok(file))?)
	}
}

impl TryFrom<Table> for File {
	type Error = mlua::Error;

	fn try_from(value: Table) -> Result<Self, Self::Error> {
		Ok(Self { url: value.raw_get("url")?, cha: value.raw_get("cha")?, ..Default::default() })
	}
}

impl TryFrom<AnyUserData> for File {
	type Error = mlua::Error;

	fn try_from(value: AnyUserData) -> Result<Self, Self::Error> {
		match value.take::<Self>() {
			Ok(me) => Ok(me),
			Err(mlua::Error::UserDataTypeMismatch) => FileRef(value).try_into(),
			Err(e) => Err(e),
		}
	}
}

impl FromLua for File {
	fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
		match value {
			Value::Table(tbl) => Self::try_from(tbl),
			Value::UserData(ud) => Self::try_from(ud),
			_ => Err(EXPECTED.into_lua_err())?,
		}
	}
}

impl UserData for File {
	fn register(registry: &mut UserDataRegistry<Self>) {
		impl_file_fields!(registry);
		impl_file_methods!(registry);

		for inv in inventory::iter::<FileInventory>() {
			(inv.register)(registry);
		}
	}
}
