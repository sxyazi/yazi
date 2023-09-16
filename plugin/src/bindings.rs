use mlua::{LuaSerdeExt, UserData, Value};
use shared::Url;

// Manager

pub struct Manager<'a>(&'a core::manager::Manager);

impl<'a> Manager<'a> {
	pub fn new(manager: &'a core::manager::Manager) -> Self { Self(manager) }
}

impl<'a> UserData for Manager<'a> {
	fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
		fields.add_field_method_get("mode", |_, this| Ok(this.0.active().mode().to_string()));

		fields.add_field_method_get("hovered", |_, this| {
			Ok(this.0.current().hovered.as_ref().map(File::from))
		})
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

pub struct File {
	pub(super) url:     Url,
	pub(super) length:  u64,
	pub(super) link_to: Option<Url>,
	pub(super) is_link: bool,
}

impl From<&core::files::File> for File {
	fn from(value: &core::files::File) -> Self {
		Self {
			url:     value.url_owned(),
			length:  value.length(),
			link_to: value.link_to().cloned(),
			is_link: value.is_link(),
		}
	}
}

impl UserData for File {
	fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
		fields.add_field_method_get("url", |_, this| Ok(this.url.to_string_lossy().to_string()));
		fields.add_field_method_get("length", |_, this| Ok(this.length));
		fields.add_field_method_get("link_to", |_, this| {
			Ok(this.link_to.as_ref().map(|l| l.to_string_lossy().to_string()))
		});
		fields.add_field_method_get("is_link", |_, this| Ok(this.is_link));
	}
}
