use core::Ctx;

use config::MANAGER;
use mlua::{AnyUserData, MetaMethod, UserDataFields, UserDataMethods, Value};

use super::Url;
use crate::LUA;

pub struct Active<'a, 'b> {
	scope: &'b mlua::Scope<'a, 'a>,

	cx:    &'a core::Ctx,
	inner: &'a core::manager::Tab,
}

impl<'a, 'b> Active<'a, 'b> {
	pub(crate) fn init() -> mlua::Result<()> {
		LUA.register_userdata_type::<core::manager::Mode>(|reg| {
			reg.add_field_method_get("is_select", |_, me| Ok(me.is_select()));
			reg.add_field_method_get("is_unset", |_, me| Ok(me.is_unset()));
			reg.add_field_method_get("is_visual", |_, me| Ok(me.is_visual()));
			reg.add_method("pending", |_, me, (idx, state): (usize, bool)| Ok(me.pending(idx, state)));

			reg.add_meta_method(MetaMethod::ToString, |_, me, ()| Ok(me.to_string()));
		})?;

		LUA.register_userdata_type::<core::manager::Folder>(|reg| {
			reg.add_field_method_get("cwd", |_, me| Ok(Url::from(&me.cwd)));
			reg.add_field_method_get("offset", |_, me| Ok(me.offset()));
			reg.add_field_method_get("cursor", |_, me| Ok(me.cursor()));

			reg.add_field_function_get("window", |_, me| me.named_user_value::<Value>("window"));
			reg.add_field_function_get("files", |_, me| me.named_user_value::<AnyUserData>("files"));
			reg.add_field_function_get("hovered", |_, me| me.named_user_value::<Value>("hovered"));
		})?;

		LUA.register_userdata_type::<core::manager::Preview>(|reg| {
			reg.add_field_function_get("folder", |_, me| me.named_user_value::<Value>("folder"));
		})?;

		Ok(())
	}

	pub(crate) fn new(scope: &'b mlua::Scope<'a, 'a>, cx: &'a Ctx) -> Self {
		Self { scope, cx, inner: cx.manager.active() }
	}

	pub(crate) fn make(&self) -> mlua::Result<AnyUserData<'a>> {
		let ud = self.scope.create_any_userdata_ref(self.inner)?;
		ud.set_named_user_value("mode", self.scope.create_any_userdata_ref(&self.inner.mode)?)?;
		ud.set_named_user_value(
			"parent",
			self.inner.parent.as_ref().and_then(|p| self.folder(p, None).ok()),
		)?;
		ud.set_named_user_value("current", self.folder(&self.inner.current, None)?)?;
		ud.set_named_user_value("preview", self.preview(self.inner)?)?;

		Ok(ud)
	}

	pub(crate) fn folder(
		&self,
		inner: &'a core::manager::Folder,
		window: Option<(usize, usize)>,
	) -> mlua::Result<AnyUserData<'a>> {
		let window = window.unwrap_or_else(|| (inner.offset(), MANAGER.layout.folder_height()));

		let ud = self.scope.create_any_userdata_ref(inner)?;
		ud.set_named_user_value(
			"window",
			inner
				.files
				.iter()
				.skip(window.0)
				.take(window.1)
				.enumerate()
				.filter_map(|(i, f)| self.file(i, f, inner).ok())
				.collect::<Vec<_>>(),
		)?;
		ud.set_named_user_value("files", self.files(&inner.files)?)?;
		// TODO: remove this
		ud.set_named_user_value(
			"hovered",
			inner.hovered.as_ref().and_then(|h| self.file(999, h, inner).ok()),
		)?;

		Ok(ud)
	}

	fn files(&self, inner: &'a core::files::Files) -> mlua::Result<AnyUserData<'a>> {
		self.scope.create_any_userdata_ref(inner)
	}

	fn file(
		&self,
		idx: usize,
		inner: &'a core::files::File,
		folder: &'a core::manager::Folder,
	) -> mlua::Result<AnyUserData<'a>> {
		let ud = self.scope.create_any_userdata_ref(inner)?;
		ud.set_named_user_value("idx", idx)?;
		ud.set_named_user_value("folder", self.scope.create_any_userdata_ref(folder)?)?;
		ud.set_named_user_value("manager", self.scope.create_any_userdata_ref(&self.cx.manager)?)?;

		Ok(ud)
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
				.and_then(|f| self.folder(f, Some((f.offset(), MANAGER.layout.preview_height()))).ok()),
		)?;

		Ok(ud)
	}
}
