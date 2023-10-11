use mlua::{AnyUserData, LuaSerdeExt, UserDataFields};

use crate::LUA;

pub struct Tasks<'a, 'b> {
	scope: &'b mlua::Scope<'a, 'a>,

	inner: &'a core::tasks::Tasks,
}

impl<'a, 'b> Tasks<'a, 'b> {
	pub(crate) fn init() -> mlua::Result<()> {
		LUA.register_userdata_type::<core::tasks::Tasks>(|reg| {
			reg.add_field_method_get("progress", |lua, me| lua.to_value(&me.progress))
		})?;

		Ok(())
	}

	pub(crate) fn new(scope: &'b mlua::Scope<'a, 'a>, inner: &'a core::tasks::Tasks) -> Self {
		Self { scope, inner }
	}

	pub(crate) fn make(&self) -> mlua::Result<AnyUserData<'a>> {
		self.scope.create_any_userdata_ref(self.inner)
	}
}
