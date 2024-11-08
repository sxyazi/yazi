use mlua::UserData;

use super::Status;

pub struct Output {
	inner: std::process::Output,
}

impl Output {
	pub fn new(inner: std::process::Output) -> Self { Self { inner } }
}

impl UserData for Output {
	fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("status", |_, me| Ok(Status::new(me.inner.status)));
		fields.add_field_method_get("stdout", |lua, me| lua.create_string(&me.inner.stdout));
		fields.add_field_method_get("stderr", |lua, me| lua.create_string(&me.inner.stderr));
	}
}
