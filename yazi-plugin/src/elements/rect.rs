use std::ops::Deref;

use mlua::{FromLua, IntoLua, Lua, MetaMethod, Table, UserData, Value};

use super::Pad;

#[derive(Clone, Copy, Debug, Default, FromLua)]
pub struct Rect(pub(super) ratatui::layout::Rect);

impl Deref for Rect {
	type Target = ratatui::layout::Rect;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl From<ratatui::layout::Rect> for Rect {
	fn from(rect: ratatui::layout::Rect) -> Self { Self(rect) }
}

impl From<ratatui::layout::Size> for Rect {
	fn from(size: ratatui::layout::Size) -> Self {
		Self(ratatui::layout::Rect { x: 0, y: 0, width: size.width, height: size.height })
	}
}

impl Rect {
	pub fn compose(lua: &Lua) -> mlua::Result<Value> {
		let new = lua.create_function(|_, (_, args): (Table, Table)| {
			Ok(Self(ratatui::layout::Rect {
				x:      args.raw_get("x").unwrap_or_default(),
				y:      args.raw_get("y").unwrap_or_default(),
				width:  args.raw_get("w").unwrap_or_default(),
				height: args.raw_get("h").unwrap_or_default(),
			}))
		})?;

		let rect = lua.create_table_from([("default", Self(ratatui::layout::Rect::default()))])?;

		rect.set_metatable(Some(lua.create_table_from([(MetaMethod::Call.name(), new)])?));
		rect.into_lua(lua)
	}

	pub(super) fn pad(self, pad: Pad) -> Self {
		let mut r = *self;
		r.x = r.x.saturating_add(pad.left);
		r.y = r.y.saturating_add(pad.top);

		r.width = r.width.saturating_sub(pad.left + pad.right);
		r.height = r.height.saturating_sub(pad.top + pad.bottom);
		Self(r)
	}
}

impl UserData for Rect {
	fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("x", |_, me| Ok(me.x));
		fields.add_field_method_get("y", |_, me| Ok(me.y));
		fields.add_field_method_get("w", |_, me| Ok(me.width));
		fields.add_field_method_get("h", |_, me| Ok(me.height));

		fields.add_field_method_set("x", |_, me, x| Ok(me.0.x = x));
		fields.add_field_method_set("y", |_, me, y| Ok(me.0.y = y));
		fields.add_field_method_set("w", |_, me, w| Ok(me.0.width = w));
		fields.add_field_method_set("h", |_, me, h| Ok(me.0.height = h));

		fields.add_field_method_get("left", |_, me| Ok(me.left()));
		fields.add_field_method_get("right", |_, me| Ok(me.right()));
		fields.add_field_method_get("top", |_, me| Ok(me.top()));
		fields.add_field_method_get("bottom", |_, me| Ok(me.bottom()));
	}

	fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
		methods.add_method("pad", |_, me, pad: Pad| Ok(me.pad(pad)));
		methods.add_method("contains", |_, me, Rect(rect)| Ok(me.contains(rect.into())));
	}
}
