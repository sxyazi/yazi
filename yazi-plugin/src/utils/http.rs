use std::sync::LazyLock;

use mlua::{ExternalError, Function, IntoLuaMulti, Lua, Table, Value};
use reqwest::Client;
use yazi_binding::{HttpRequest, HttpTransport};

use super::Utils;

impl Utils {
	pub(super) fn http(lua: &Lua) -> mlua::Result<Table> {
		lua.create_table_from([("request", Self::request(lua)?)])
	}

	fn request(lua: &Lua) -> mlua::Result<Function> {
		lua.create_async_function(|lua, request: HttpRequest| async move {
			let result = match Self::client() {
				Ok(client) => HttpTransport::new(client).send(request).await,
				Err(e) => return (Value::Nil, e).into_lua_multi(&lua),
			};

			match result {
				Ok(response) => response.into_lua_multi(&lua),
				Err(e) => (Value::Nil, e.into_lua_err()).into_lua_multi(&lua),
			}
		})
	}

	fn client() -> mlua::Result<&'static Client> {
		static HTTP: LazyLock<Result<Client, reqwest::Error>> = LazyLock::new(|| {
			Client::builder()
				.user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/151.0.0.0 Safari/537.36")
				.build()
		});

		HTTP.as_ref().map_err(|e| e.into_lua_err())
	}
}
