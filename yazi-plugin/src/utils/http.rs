use mlua::{ExternalError, Function, IntoLuaMulti, Lua, Table, Value};
use yazi_binding::{HttpRequest, HttpTransport};

use super::Utils;
use crate::HTTP;

impl Utils {
	pub(super) fn http(lua: &Lua) -> mlua::Result<Table> {
		lua.create_table_from([("request", Self::request(lua)?)])
	}

	fn request(lua: &Lua) -> mlua::Result<Function> {
		lua.create_async_function(|lua, request: HttpRequest| async move {
			match HttpTransport::new(&HTTP).send(request).await {
				Ok(response) => response.into_lua_multi(&lua),
				Err(e) => (Value::Nil, e.into_lua_err()).into_lua_multi(&lua),
			}
		})
	}
}
