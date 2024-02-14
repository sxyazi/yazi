use mlua::{Lua, MetaMethod, UserDataFields, UserDataMethods};

pub(super) struct Yanked;

impl Yanked {
	pub(super) fn register(lua: &Lua) -> mlua::Result<()> {
		lua.register_userdata_type::<yazi_core::manager::Yanked>(|reg| {
			reg.add_field_method_get("is_cut", |_, me| Ok(me.cut));

			reg.add_meta_method(MetaMethod::Len, |_, me, ()| Ok(me.len()));
		})
	}
}
