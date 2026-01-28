use std::mem;

use mlua::{UserData, Value};
use yazi_binding::{cached_field, cached_field_mut};

use super::Status;

pub struct Output {
	inner: std::process::Output,

	v_status: Option<Value>,
	v_stdout: Option<mlua::Result<Value>>,
	v_stderr: Option<mlua::Result<Value>>,
}

impl Output {
	pub fn new(inner: std::process::Output) -> Self {
		Self { inner, v_status: None, v_stdout: None, v_stderr: None }
	}
}

impl UserData for Output {
	fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
		cached_field!(fields, status, |_, me| Ok(Status::new(me.inner.status)));
		cached_field_mut!(fields, stdout, |lua, me| {
			lua.create_external_string(mem::take(&mut me.inner.stdout))
		});
		cached_field_mut!(fields, stderr, |lua, me| {
			lua.create_external_string(mem::take(&mut me.inner.stderr))
		});
	}
}
