use mlua::{LuaSerdeExt, UserData};

// Manager

pub struct Manager<'a>(&'a core::manager::Manager);

impl<'a> Manager<'a> {
	pub fn new(manager: &'a core::manager::Manager) -> Self { Self(manager) }
}

impl<'a> UserData for Manager<'a> {
	fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
		fields.add_field_method_get("mode", |_, this| Ok(this.0.active().mode().to_string()));

		fields.add_field_method_get("current_cursor", |_, this| Ok(this.0.current().cursor()));
		fields.add_field_method_get("current_length", |_, this| Ok(this.0.current().files.len()));
		fields.add_field_method_get("current_hovered", |_, this| {
			Ok(this.0.current().hovered.as_ref().map(File::from))
		});
	}
}

// Tasks

pub struct Tasks<'a>(&'a core::tasks::Tasks);

impl<'a> Tasks<'a> {
	pub fn new(tasks: &'a core::tasks::Tasks) -> Self { Self(tasks) }
}

impl<'a> UserData for Tasks<'a> {
	fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
		fields.add_field_method_get("progress", |lua, this| lua.to_value(&this.0.progress))
	}
}

// File

pub struct File(core::files::File);

impl From<&core::files::File> for File {
	fn from(value: &core::files::File) -> Self { Self(value.clone()) }
}

impl UserData for File {
	fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
		fields.add_field_method_get("url", |_, this| Ok(this.0.url().to_string_lossy().to_string()));
		fields.add_field_method_get("length", |_, this| Ok(this.0.length()));
		fields.add_field_method_get("link_to", |_, this| {
			Ok(this.0.link_to().map(|l| l.to_string_lossy().to_string()))
		});
		fields.add_field_method_get("is_link", |_, this| Ok(this.0.is_link()));
		fields.add_field_method_get("is_hidden", |_, this| Ok(this.0.is_hidden()));

		// Meta
		fields.add_field_method_get("permissions", |_, this| {
			Ok(shared::permissions(this.0.meta().permissions()))
		});
	}
}
