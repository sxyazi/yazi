use mlua::{AnyUserData, Lua, MetaMethod, UserDataFields, UserDataMethods, Value};

pub struct Tabs<'a, 'b> {
	scope: &'b mlua::Scope<'a, 'a>,

	inner: &'a yazi_core::manager::Tabs,
}

impl<'a, 'b> Tabs<'a, 'b> {
	pub(super) fn register(lua: &Lua) -> mlua::Result<()> {
		lua.register_userdata_type::<yazi_core::manager::Tabs>(|reg| {
			reg.add_field_method_get("idx", |_, me| Ok(me.idx));
			reg.add_meta_method(MetaMethod::Len, |_, me, ()| Ok(me.len()));
			reg.add_meta_function(MetaMethod::Index, |_, (me, index): (AnyUserData, usize)| {
				let items = me.named_user_value::<Vec<AnyUserData>>("items")?;
				Ok(items.get(index - 1).cloned())
			});
		})?;

		lua.register_userdata_type::<yazi_core::tab::Tab>(|reg| {
			reg.add_method("name", |lua, me, ()| {
				Some(lua.create_string(
					me.current.cwd.file_name().map_or_else(
						|| me.current.cwd.as_os_str().as_encoded_bytes(),
						|n| n.as_encoded_bytes(),
					),
				))
				.transpose()
			});

			reg.add_field_function_get("mode", |_, me| me.named_user_value::<AnyUserData>("mode"));
			reg.add_field_function_get("conf", |_, me| me.named_user_value::<AnyUserData>("conf"));
			reg.add_field_function_get("parent", |_, me| me.named_user_value::<Value>("parent"));
			reg.add_field_function_get("current", |_, me| me.named_user_value::<AnyUserData>("current"));
			reg.add_field_function_get("preview", |_, me| me.named_user_value::<AnyUserData>("preview"));
		})?;

		Ok(())
	}
}

impl<'a, 'b> Tabs<'a, 'b> {
	pub(crate) fn new(scope: &'b mlua::Scope<'a, 'a>, inner: &'a yazi_core::manager::Tabs) -> Self {
		Self { scope, inner }
	}

	pub(crate) fn make(&self) -> mlua::Result<AnyUserData<'a>> {
		let ud = self.scope.create_any_userdata_ref(self.inner)?;

		ud.set_named_user_value(
			"items",
			self.inner.iter().filter_map(|t| self.tab(t).ok()).collect::<Vec<_>>(),
		)?;

		Ok(ud)
	}

	fn tab(&self, inner: &'a yazi_core::tab::Tab) -> mlua::Result<AnyUserData<'a>> {
		let ud = self.scope.create_any_userdata_ref(inner)?;

		ud.set_named_user_value("parent", inner.parent.as_ref().and_then(|p| self.folder(p).ok()))?;
		ud.set_named_user_value("current", self.folder(&inner.current)?)?;

		Ok(ud)
	}

	pub(crate) fn folder(
		&self,
		inner: &'a yazi_core::folder::Folder,
	) -> mlua::Result<AnyUserData<'a>> {
		let ud = self.scope.create_any_userdata_ref(inner)?;
		ud.set_named_user_value("files", self.scope.create_any_userdata_ref(&inner.files)?)?;

		Ok(ud)
	}
}
