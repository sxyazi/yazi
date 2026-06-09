use std::{mem, ops::Deref};

use mlua::{IntoLua, UserData, UserDataFields, Value};
use yazi_shim::{mlua::UserDataFieldsExt, strum::IntoStr};
use yazi_term::event::{DndDropArrive, DndEvent as Inner};

pub struct DndEvent {
	inner: Inner,
}

impl Deref for DndEvent {
	type Target = Inner;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl From<Inner> for DndEvent {
	fn from(inner: Inner) -> Self { Self { inner } }
}

impl UserData for DndEvent {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("type", |_, me| Ok(me.inner.r#type()));

		fields.add_field_method_get("x", |_, me| Ok(me.inner.x()));

		fields.add_field_method_get("y", |_, me| Ok(me.inner.y()));

		fields.add_field_method_get("idx", |_, me| Ok(me.inner.idx()));

		fields.add_field_method_get("op", |_, me| Ok(me.inner.op().map(IntoStr::into_str)));

		fields.add_cached_field("mimes", |lua, me| {
			if let Some(mimes) = me.inner.mimes() {
				lua.create_sequence_from(mimes.iter())?.into_lua(lua)
			} else {
				Ok(Value::Nil)
			}
		});

		fields.add_cached_field_mut("data", |lua, me| match &mut me.inner {
			Inner::DropArrive(DndDropArrive { data, .. }) => {
				lua.create_external_string(mem::take(data))?.into_lua(lua)
			}
			_ => Ok(Value::Nil),
		});
	}
}
