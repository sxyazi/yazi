use mlua::{UserData, UserDataFields};

pub struct Status {
	inner: std::process::ExitStatus,
}

impl Status {
	pub fn new(inner: std::process::ExitStatus) -> Self { Self { inner } }
}

impl UserData for Status {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("success", |_, me| Ok(me.inner.success()));
		fields.add_field_method_get("code", |_, me| Ok(me.inner.code()));
	}
}
