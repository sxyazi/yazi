use mlua::{AnyUserData, MetaMethod, UserDataFields, UserDataMethods, Value};

use crate::LUA;

pub struct Tabs<'a, 'b> {
	scope: &'b mlua::Scope<'a, 'a>,

	inner: &'a core::manager::Tabs,
}

impl<'a, 'b> Tabs<'a, 'b> {
	pub(crate) fn init() -> mlua::Result<()> {
		LUA.register_userdata_type::<core::manager::Tabs>(|reg| {
			reg.add_field_method_get("idx", |_, me| Ok(me.idx()));
			reg.add_meta_method(MetaMethod::Len, |_, me, ()| Ok(me.len()));
			reg.add_meta_function(MetaMethod::Index, |_, (me, index): (AnyUserData, usize)| {
				let items = me.named_user_value::<Vec<AnyUserData>>("items")?;
				Ok(items.get(index - 1).cloned())
			});
		})?;

		LUA.register_userdata_type::<core::manager::Tab>(|reg| {
			reg.add_method("name", |_, me, ()| {
				Ok(
					me.current
						.cwd
						.file_name()
						.map(|n| n.to_string_lossy())
						.or_else(|| Some(me.current.cwd.to_string_lossy()))
						.unwrap_or_default()
						.into_owned(),
				)
			});

			reg.add_field_function_get("mode", |_, me| me.named_user_value::<AnyUserData>("mode"));
			reg.add_field_function_get("parent", |_, me| me.named_user_value::<Value>("parent"));
			reg.add_field_function_get("current", |_, me| me.named_user_value::<AnyUserData>("current"));
			reg.add_field_function_get("preview", |_, me| me.named_user_value::<AnyUserData>("preview"));
		})?;

		Ok(())
	}

	pub(crate) fn new(scope: &'b mlua::Scope<'a, 'a>, inner: &'a core::manager::Tabs) -> Self {
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

	fn tab(&self, inner: &'a core::manager::Tab) -> mlua::Result<AnyUserData<'a>> {
		let ud = self.scope.create_any_userdata_ref(inner)?;

		ud.set_named_user_value("parent", inner.parent.as_ref().and_then(|p| self.folder(p).ok()))?;
		ud.set_named_user_value("current", self.folder(&inner.current)?)?;
		ud.set_named_user_value("preview", self.preview(inner)?)?;

		Ok(ud)
	}

	pub(crate) fn folder(&self, inner: &'a core::manager::Folder) -> mlua::Result<AnyUserData<'a>> {
		let ud = self.scope.create_any_userdata_ref(inner)?;
		ud.set_named_user_value("files", self.files(&inner.files)?)?;

		Ok(ud)
	}

	fn files(&self, inner: &'a core::files::Files) -> mlua::Result<AnyUserData<'a>> {
		self.scope.create_any_userdata_ref(inner)
	}

	fn preview(&self, tab: &'a core::manager::Tab) -> mlua::Result<AnyUserData<'a>> {
		let inner = tab.preview();

		let ud = self.scope.create_any_userdata_ref(inner)?;
		ud.set_named_user_value(
			"folder",
			inner
				.lock
				.as_ref()
				.filter(|l| l.is_folder())
				.and_then(|l| tab.history(&l.url))
				.and_then(|f| self.folder(f).ok()),
		)?;

		Ok(ud)
	}
}
