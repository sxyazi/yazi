use mlua::{Lua, UserData, Value};
use yazi_binding::cached_field;

use super::Status;

pub struct Output {
	inner: std::process::Output,

	v_status: Option<Value>,
	v_stdout: Option<Value>,
	v_stderr: Option<Value>,
}

impl Output {
	pub fn new(inner: std::process::Output) -> Self {
		Self { inner, v_status: None, v_stdout: None, v_stderr: None }
	}
}

impl UserData for Output {
	fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
		cached_field!(fields, status, |_, me: &Self| Ok(Status::new(me.inner.status)));
		cached_field!(fields, stdout, |lua: &Lua, me: &Self| lua.create_string(&me.inner.stdout));
		cached_field!(fields, stderr, |lua: &Lua, me: &Self| lua.create_string(&me.inner.stderr));
	}
}
