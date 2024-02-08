use mlua::{AnyUserData, Lua, LuaSerdeExt, UserDataFields};

pub(super) struct Tasks<'a, 'b> {
	scope: &'b mlua::Scope<'a, 'a>,

	inner: &'a yazi_core::tasks::Tasks,
}

impl<'a, 'b> Tasks<'a, 'b> {
	pub(super) fn register(lua: &Lua) -> mlua::Result<()> {
		lua.register_userdata_type::<yazi_core::tasks::Tasks>(|reg| {
			reg.add_field_method_get("progress", |lua, me| lua.to_value(&me.progress))
		})?;

		Ok(())
	}

	pub(crate) fn new(scope: &'b mlua::Scope<'a, 'a>, inner: &'a yazi_core::tasks::Tasks) -> Self {
		Self { scope, inner }
	}

	pub(crate) fn make(&self) -> mlua::Result<AnyUserData<'a>> {
		self.scope.create_any_userdata_ref(self.inner)
	}
}
