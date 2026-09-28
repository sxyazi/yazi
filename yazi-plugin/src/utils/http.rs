use mlua::{BorrowedBytes, Function, Lua, MetaMethod, Table};
use yazi_binding::{HttpBuilder, HttpPart};

use super::Utils;

impl Utils {
	pub(super) fn http(lua: &Lua) -> mlua::Result<Table> {
		let new = lua.create_function(|_, (_, method, url): (Table, BorrowedBytes, String)| {
			Ok(HttpBuilder::new(&method, url)?)
		})?;

		let http = lua.create_table_from([("part", Self::http_part(lua)?)])?;
		http.set_metatable(Some(lua.create_table_from([(MetaMethod::Call.name(), new)])?))?;

		Ok(http)
	}

	fn http_part(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|_, (name, options): (String, Table)| {
			Ok(HttpPart::new(name, options.raw_get("filename")?, options.raw_get("content_type")?))
		})
	}
}
