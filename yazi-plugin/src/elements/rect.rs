use mlua::{AnyUserData, IntoLua, Lua, Table, UserDataFields, UserDataMethods, UserDataRef};

use super::PaddingRef;
use crate::bindings::Cast;

pub type RectRef<'lua> = UserDataRef<'lua, ratatui::layout::Rect>;

pub struct Rect;

impl Rect {
	pub fn install(lua: &Lua, ui: &Table) -> mlua::Result<()> {
		let new = lua.create_function(|lua, (_, args): (Table, Table)| {
			Rect::cast(lua, ratatui::layout::Rect {
				x:      args.get("x")?,
				y:      args.get("y")?,
				width:  args.get("w")?,
				height: args.get("h")?,
			})
		})?;

		let rect = lua.create_table_from([(
			"default",
			Rect::cast(lua, ratatui::layout::Rect::default())?.into_lua(lua)?,
		)])?;

		rect.set_metatable(Some(lua.create_table_from([("__call", new)])?));

		ui.set("Rect", rect)
	}

	pub fn register(lua: &Lua) -> mlua::Result<()> {
		lua.register_userdata_type::<ratatui::layout::Rect>(|reg| {
			reg.add_field_method_get("x", |_, me| Ok(me.x));
			reg.add_field_method_get("y", |_, me| Ok(me.y));
			reg.add_field_method_get("w", |_, me| Ok(me.width));
			reg.add_field_method_get("h", |_, me| Ok(me.height));

			reg.add_field_method_get("left", |_, me| Ok(me.left()));
			reg.add_field_method_get("right", |_, me| Ok(me.right()));
			reg.add_field_method_get("top", |_, me| Ok(me.top()));
			reg.add_field_method_get("bottom", |_, me| Ok(me.bottom()));

			reg.add_method("padding", |lua, me, padding: PaddingRef| {
				let mut r = *me;
				r.x = r.x.saturating_add(padding.left);
				r.y = r.y.saturating_add(padding.top);

				r.width = r.width.saturating_sub(padding.left + padding.right);
				r.height = r.height.saturating_sub(padding.top + padding.bottom);
				Rect::cast(lua, r)
			});
		})
	}
}

impl Cast<ratatui::layout::Rect> for Rect {
	fn cast(lua: &Lua, data: ratatui::layout::Rect) -> mlua::Result<AnyUserData> {
		lua.create_any_userdata(data)
	}
}
