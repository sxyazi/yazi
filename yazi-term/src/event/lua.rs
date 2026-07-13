use std::mem;

use mlua::{IntoLua, Lua, LuaSerdeExt, UserData, UserDataFields, Value};
use yazi_shim::{mlua::{SER_OPT, UserDataFieldsExt}, strum::IntoStr};

use crate::event::{ClipboardEvent, ClipboardRead, DndDropArrive, DndEvent, KeyEvent, MouseButton, MouseEvent, MouseEventKind};

// --- DndEvent
impl UserData for DndEvent {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("type", |_, me| Ok(me.r#type()));

		fields.add_field_method_get("x", |_, me| Ok(me.x()));

		fields.add_field_method_get("y", |_, me| Ok(me.y()));

		fields.add_field_method_get("idx", |_, me| Ok(me.idx()));

		fields.add_field_method_get("op", |_, me| Ok(me.op().map(IntoStr::into_str)));

		fields.add_cached_field("mimes", |lua, me| {
			if let Some(mimes) = me.mimes() {
				lua.create_sequence_from(mimes.iter())?.into_lua(lua)
			} else {
				Ok(Value::Nil)
			}
		});

		fields.add_cached_field_mut("data", |lua, me| match me {
			Self::DropArrive(DndDropArrive { data, .. }) => {
				lua.create_external_string(mem::take(data))?.into_lua(lua)
			}
			_ => Ok(Value::Nil),
		});
	}
}

// --- ClipboardEvent
impl UserData for ClipboardEvent {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("type", |_, me| Ok(me.r#type()));

		fields.add_field_method_get("pw", |_, me| Ok(me.pw()));

		fields.add_field_method_get("primary", |_, me| Ok(me.primary()));

		fields.add_cached_field("mimes", |lua, me| {
			if let Some(mimes) = me.mimes() {
				lua.create_sequence_from(mimes.iter())?.into_lua(lua)
			} else {
				Ok(Value::Nil)
			}
		});

		fields.add_cached_field_mut("data", |lua, me| match me {
			Self::ReadData(ClipboardRead { data, .. }) => lua
				.create_table_from(
					data
						.iter()
						.map(|d| Ok((lua.create_string(&d.mime)?, lua.create_external_string(&*d.data)?)))
						.collect::<Result<Vec<_>, mlua::Error>>()?,
				)?
				.into_lua(lua),
			_ => Ok(Value::Nil),
		});
	}
}

// --- MouseEvent
impl UserData for MouseEvent {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field("type", "legacy");
		fields.add_field_method_get("x", |_, me| Ok(me.column));
		fields.add_field_method_get("y", |_, me| Ok(me.row));
		fields.add_field_method_get("is_left", |_, me| {
			use MouseEventKind as K;
			Ok(matches!(me.kind, K::Down(b) | K::Up(b) | K::Drag(b) if b == MouseButton::Left))
		});
		fields.add_field_method_get("is_right", |_, me| {
			use MouseEventKind as K;
			Ok(matches!(me.kind, K::Down(b) | K::Up(b) | K::Drag(b) if b == MouseButton::Right))
		});
		fields.add_field_method_get("is_middle", |_, me| {
			use MouseEventKind as K;
			Ok(matches!(me.kind, K::Down(b) | K::Up(b) | K::Drag(b) if b == MouseButton::Middle))
		});
	}
}

// --- KeyEvent
impl IntoLua for KeyEvent {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> { lua.to_value_with(&self, SER_OPT) }
}
