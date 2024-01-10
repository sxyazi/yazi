use mlua::{AnyUserData, Lua, UserDataFields};

use super::Cast;
use crate::elements::Style;

pub struct Icon;

impl Icon {
	pub fn register(lua: &Lua) -> mlua::Result<()> {
		lua.register_userdata_type::<&yazi_config::theme::Icon>(|reg| {
			reg.add_field_method_get("text", |lua, me| lua.create_string(&me.text));
			reg.add_field_method_get("style", |_, me| Ok(Style::from(me.style)));
		})?;

		Ok(())
	}
}

impl Cast<&'static yazi_config::theme::Icon> for Icon {
	fn cast<'lua>(
		lua: &'lua Lua,
		data: &'static yazi_config::theme::Icon,
	) -> mlua::Result<AnyUserData<'lua>> {
		lua.create_any_userdata(data)
	}
}
