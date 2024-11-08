use std::ops::Deref;

use mlua::{FromLua, Lua, Table, UserData};

use super::{Padding, Position};

#[derive(Clone, Copy, Default, FromLua)]
pub struct Rect(ratatui::layout::Rect);

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
	pub fn install(lua: &Lua, ui: &Table) -> mlua::Result<()> {
		let new = lua.create_function(|_, (_, args): (Table, Table)| {
			Ok(Self(ratatui::layout::Rect {
				x:      args.raw_get("x")?,
				y:      args.raw_get("y")?,
				width:  args.raw_get("w")?,
				height: args.raw_get("h")?,
			}))
		})?;

		let rect = lua.create_table_from([("default", Self(ratatui::layout::Rect::default()))])?;

		rect.set_metatable(Some(lua.create_table_from([("__call", new)])?));

		ui.raw_set("Rect", rect)
	}
}

impl UserData for Rect {
	fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("x", |_, me| Ok(me.x));
		fields.add_field_method_get("y", |_, me| Ok(me.y));
		fields.add_field_method_get("w", |_, me| Ok(me.width));
		fields.add_field_method_get("h", |_, me| Ok(me.height));

		fields.add_field_method_get("left", |_, me| Ok(me.left()));
		fields.add_field_method_get("right", |_, me| Ok(me.right()));
		fields.add_field_method_get("top", |_, me| Ok(me.top()));
		fields.add_field_method_get("bottom", |_, me| Ok(me.bottom()));
	}

	fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
		methods.add_method("padding", |_, me, padding: Padding| {
			let mut r = **me;
			r.x = r.x.saturating_add(padding.left);
			r.y = r.y.saturating_add(padding.top);

			r.width = r.width.saturating_sub(padding.left + padding.right);
			r.height = r.height.saturating_sub(padding.top + padding.bottom);
			Ok(Self(r))
		});
		methods.add_method("position", |_, me, ()| Ok(Position::from(**me)));
		methods.add_method("contains", |_, me, position: Position| Ok(me.contains(*position)));
	}
}
