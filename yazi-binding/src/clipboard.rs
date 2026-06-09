use std::ops::Deref;

use mlua::{IntoLua, UserData, UserDataFields, Value};
use yazi_shim::mlua::UserDataFieldsExt;
use yazi_term::event::{ClipboardEvent as Inner, ClipboardRead};

pub struct ClipboardEvent {
	inner: Inner,
}

impl Deref for ClipboardEvent {
	type Target = Inner;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl From<Inner> for ClipboardEvent {
	fn from(inner: Inner) -> Self { Self { inner } }
}

impl UserData for ClipboardEvent {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("type", |_, me| Ok(me.inner.r#type()));

		fields.add_field_method_get("pw", |_, me| Ok(me.inner.pw()));

		fields.add_field_method_get("primary", |_, me| Ok(me.inner.primary()));

		fields.add_cached_field("mimes", |lua, me| {
			if let Some(mimes) = me.inner.mimes() {
				lua.create_sequence_from(mimes.iter())?.into_lua(lua)
			} else {
				Ok(Value::Nil)
			}
		});

		fields.add_cached_field_mut("data", |lua, me| match &mut me.inner {
			Inner::ReadData(ClipboardRead { data, .. }) => lua
				.create_table_from(data.iter().map(|d| {
					(
						lua.create_string(&d.mime).ok(),
						// TODO !!5522!! is this the best way
						lua.create_external_string(&*d.data).ok(),
					)
				}))?
				.into_lua(lua),
			_ => Ok(Value::Nil),
		});
	}
}
