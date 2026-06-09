use std::mem;

use mlua::{UserData, UserDataFields};
use yazi_shim::mlua::UserDataFieldsExt;

use super::Status;

pub struct Output {
	inner: std::process::Output,
}

impl Output {
	pub fn new(inner: std::process::Output) -> Self { Self { inner } }
}

impl UserData for Output {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_cached_field("status", |_, me| Ok(Status::new(me.inner.status)));
		fields.add_cached_field_mut("stdout", |lua, me| {
			lua.create_external_string(mem::take(&mut me.inner.stdout))
		});
		fields.add_cached_field_mut("stderr", |lua, me| {
			lua.create_external_string(mem::take(&mut me.inner.stderr))
		});
	}
}
