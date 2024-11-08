use mlua::{AnyUserData, Lua, UserDataFields};

use super::Cast;
use crate::elements::Style;

pub struct Icon;

impl Icon {
	pub fn register(lua: &Lua) -> mlua::Result<()> {
		lua.register_userdata_type::<&yazi_shared::theme::Icon>(|reg| {
			reg.add_field_method_get("text", |lua, me| lua.create_string(&me.text));
			reg.add_field_method_get("style", |_, me| Ok(Style::from(me.style)));
		})?;

		Ok(())
	}
}

impl Cast<&'static yazi_shared::theme::Icon> for Icon {
	fn cast(lua: &Lua, data: &'static yazi_shared::theme::Icon) -> mlua::Result<AnyUserData> {
		lua.create_any_userdata(data)
	}
}
