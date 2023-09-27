use mlua::{LuaSerdeExt, UserData};

// Tasks
pub struct Tasks<'a>(&'a core::tasks::Tasks);

impl<'a> Tasks<'a> {
	pub fn new(tasks: &'a core::tasks::Tasks) -> Self { Self(tasks) }
}

impl<'a> UserData for Tasks<'a> {
	fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
		fields.add_field_method_get("progress", |lua, me| lua.to_value(&me.0.progress))
	}
}
