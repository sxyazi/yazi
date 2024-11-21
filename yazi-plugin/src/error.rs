use mlua::{Lua, MetaMethod, UserData, UserDataMethods};

pub enum Error {
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
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_method(MetaMethod::ToString, |_, me, ()| {
			Ok(match me {
				Error::Serde(e) => e.to_string(),
				Error::Custom(s) => s.clone(),
			})
		});
	}
}
