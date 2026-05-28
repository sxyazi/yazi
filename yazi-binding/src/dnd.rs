use std::{mem, ops::Deref};

use mlua::{UserData, UserDataFields, Value};
use yazi_shim::strum::IntoStr;
use yazi_term::event::{DndDropArrive, DndEvent as Inner};

use crate::{cached_field, cached_field_mut};

pub struct DndEvent {
	inner: Inner,

	v_mimes: Option<Value>,
	v_data:  Option<mlua::Result<Value>>,
}

impl Deref for DndEvent {
	type Target = Inner;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl From<Inner> for DndEvent {
	fn from(inner: Inner) -> Self { Self { inner, v_mimes: None, v_data: None } }
}

impl UserData for DndEvent {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("type", |_, me| Ok(me.inner.r#type()));

		fields.add_field_method_get("x", |_, me| Ok(me.inner.x()));

		fields.add_field_method_get("y", |_, me| Ok(me.inner.y()));

		fields.add_field_method_get("idx", |_, me| Ok(me.inner.idx()));

		fields.add_field_method_get("op", |_, me| Ok(me.inner.op().map(IntoStr::into_str)));

		cached_field!(fields, mimes, |lua, me| {
			if let Some(mimes) = me.inner.mimes() {
				lua.create_sequence_from(mimes.iter())?.into_lua(lua)
			} else {
				Ok(Value::Nil)
			}
		});

		cached_field_mut!(fields, data, |lua, me| {
			match &mut me.inner {
				Inner::DropArrive(DndDropArrive { data, .. }) => {
					lua.create_external_string(mem::take(data))?.into_lua(lua)
				}
				_ => Ok(Value::Nil),
			}
		});
	}
}
