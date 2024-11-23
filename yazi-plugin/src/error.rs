use mlua::{IntoLua, Lua, MetaMethod, UserData, UserDataFields, UserDataMethods};

pub enum Error {
	Io(std::io::Error),
	Serde(serde_json::Error),
	Custom(String),
}

impl Error {
	pub fn install(lua: &Lua) -> mlua::Result<()> {
		let new = lua.create_function(|_, msg: String| Ok(Error::Custom(msg)))?;

		lua.globals().raw_set("Error", lua.create_table_from([("custom", new)])?)
	}
}

impl UserData for Error {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("code", |_, me| {
			Ok(match me {
				Error::Io(e) => e.raw_os_error(),
				_ => None,
			})
		});
	}

	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_method(MetaMethod::ToString, |lua, me, ()| {
			Ok(match me {
				Error::Io(e) => e.to_string().into_lua(lua),
				Error::Serde(e) => e.to_string().into_lua(lua),
				Error::Custom(s) => lua.create_string(s)?.into_lua(lua),
			})
		});
	}
}
