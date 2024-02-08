use mlua::{AnyUserData, Lua, MetaMethod, UserDataFields, UserDataMethods, Value};

use super::Folder;

pub(super) struct Active<'a, 'b> {
	scope: &'b mlua::Scope<'a, 'a>,

	inner: &'a yazi_core::tab::Tab,
}

impl<'a, 'b> Active<'a, 'b> {
	pub(super) fn register(lua: &Lua) -> mlua::Result<()> {
		lua.register_userdata_type::<yazi_core::tab::Mode>(|reg| {
			reg.add_field_method_get("is_select", |_, me| Ok(me.is_select()));
			reg.add_field_method_get("is_unset", |_, me| Ok(me.is_unset()));
			reg.add_field_method_get("is_visual", |_, me| Ok(me.is_visual()));
			reg.add_method("pending", |_, me, (idx, state): (usize, bool)| Ok(me.pending(idx, state)));

			reg.add_meta_method(MetaMethod::ToString, |_, me, ()| Ok(me.to_string()));
		})?;

		lua.register_userdata_type::<yazi_core::tab::Config>(|reg| {
			reg.add_field_method_get("sort_by", |_, me| Ok(me.sort_by.to_string()));
			reg.add_field_method_get("sort_sensitive", |_, me| Ok(me.sort_sensitive));
			reg.add_field_method_get("sort_reverse", |_, me| Ok(me.sort_reverse));
			reg.add_field_method_get("sort_dir_first", |_, me| Ok(me.sort_dir_first));

			reg.add_field_method_get("linemode", |_, me| Ok(me.linemode.to_owned()));
			reg.add_field_method_get("show_hidden", |_, me| Ok(me.show_hidden));
		})?;

		lua.register_userdata_type::<yazi_core::tab::Preview>(|reg| {
			reg.add_field_method_get("skip", |_, me| Ok(me.skip));
			reg.add_field_function_get("folder", |_, me| me.named_user_value::<Value>("folder"));
		})?;

		Ok(())
	}
}

impl<'a, 'b> Active<'a, 'b> {
	pub(crate) fn new(scope: &'b mlua::Scope<'a, 'a>, inner: &'a yazi_core::tab::Tab) -> Self {
		Self { scope, inner }
	}

	pub(crate) fn make(&self) -> mlua::Result<AnyUserData<'a>> {
		let ud = self.scope.create_any_userdata_ref(self.inner)?;
		ud.set_named_user_value("mode", self.scope.create_any_userdata_ref(&self.inner.mode)?)?;
		ud.set_named_user_value("conf", self.scope.create_any_userdata_ref(&self.inner.conf)?)?;
		ud.set_named_user_value(
			"parent",
			self.inner.parent.as_ref().and_then(|p| Folder::make(self.scope, p).ok()),
		)?;
		ud.set_named_user_value("current", Folder::make(self.scope, &self.inner.current)?)?;
		ud.set_named_user_value("preview", self.preview(self.inner)?)?;

		Ok(ud)
	}

	fn preview(&self, tab: &'a yazi_core::tab::Tab) -> mlua::Result<AnyUserData<'a>> {
		let inner = &tab.preview;

		let ud = self.scope.create_any_userdata_ref(inner)?;
		ud.set_named_user_value(
			"folder",
			tab
				.current
				.hovered()
				.filter(|&f| f.is_dir())
				.and_then(|f| tab.history(&f.url))
				.and_then(|f| Folder::make(self.scope, f).ok()),
		)?;

		Ok(ud)
	}
}
