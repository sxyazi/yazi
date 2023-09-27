use mlua::{AnyUserData, LuaSerdeExt, UserDataFields};

use crate::LUA;

pub struct Tasks;

impl Tasks {
	pub(crate) fn init() -> mlua::Result<()> {
		LUA.register_userdata_type::<core::tasks::Tasks>(|reg| {
			reg.add_field_method_get("progress", |lua, me| lua.to_value(&me.progress))
		})?;

		Ok(())
	}

	pub(crate) fn make<'a>(
		scope: &mlua::Scope<'a, 'a>,
		inner: &'a core::tasks::Tasks,
	) -> mlua::Result<AnyUserData<'a>> {
		scope.create_any_userdata_ref(inner)
	}
}
