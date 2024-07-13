use crossterm::event::MouseButton;
use mlua::{AnyUserData, Lua, UserDataFields};

use super::Cast;

pub struct MouseEvent;

impl MouseEvent {
	pub fn register(lua: &Lua) -> mlua::Result<()> {
		lua.register_userdata_type::<crossterm::event::MouseEvent>(|reg| {
			reg.add_field_method_get("x", |_, me| Ok(me.column));
			reg.add_field_method_get("y", |_, me| Ok(me.row));
			reg.add_field_method_get("is_left", |_, me| {
				use crossterm::event::MouseEventKind as K;
				Ok(matches!(me.kind, K::Down(b) | K::Up(b) | K::Drag(b) if b == MouseButton::Left))
			});
			reg.add_field_method_get("is_right", |_, me| {
				use crossterm::event::MouseEventKind as K;
				Ok(matches!(me.kind, K::Down(b) | K::Up(b) | K::Drag(b) if b == MouseButton::Right))
			});
			reg.add_field_method_get("is_middle", |_, me| {
				use crossterm::event::MouseEventKind as K;
				Ok(matches!(me.kind, K::Down(b) | K::Up(b) | K::Drag(b) if b == MouseButton::Middle))
			});
		})?;

		Ok(())
	}
}

impl Cast<crossterm::event::MouseEvent> for MouseEvent {
	fn cast(lua: &Lua, data: crossterm::event::MouseEvent) -> mlua::Result<AnyUserData> {
		lua.create_any_userdata(data)
	}
}
