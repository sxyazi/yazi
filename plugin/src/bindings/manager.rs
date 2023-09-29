use mlua::{AnyUserData, IntoLua, MetaMethod, UserDataFields, UserDataMethods};

use super::Url;
use crate::LUA;

pub struct Manager;

impl Manager {
	pub(crate) fn init() -> mlua::Result<()> {
		LUA.register_userdata_type::<core::manager::Manager>(|reg| {
			reg.add_field_method_get("mode", |_, me| Ok(me.active().mode().to_string()));
			reg.add_field_function_get("parent", |_, me| me.named_user_value::<AnyUserData>("parent"));
			reg.add_field_function_get("current", |_, me| me.named_user_value::<AnyUserData>("current"));
			reg.add_field_function_get("preview", |_, me| me.named_user_value::<AnyUserData>("preview"));
		})?;

		LUA.register_userdata_type::<core::manager::Folder>(|reg| {
			reg.add_field_method_get("cwd", |_, me| Ok(Url::from(&me.cwd)));
			reg.add_field_method_get("offset", |_, me| Ok(me.offset()));
			reg.add_field_method_get("cursor", |_, me| Ok(me.cursor()));

			reg.add_field_function_get("files", |_, me| me.named_user_value::<AnyUserData>("files"));
			reg.add_field_function_get("hovered", |_, me| me.named_user_value::<AnyUserData>("hovered"));
		})?;

		LUA.register_userdata_type::<core::files::Files>(|reg| {
			reg.add_meta_method(MetaMethod::Len, |_, me, ()| Ok(me.len()));

			reg.add_meta_function(MetaMethod::Pairs, |lua, me: AnyUserData| {
				let iter = lua.create_function(|lua, (me, i): (AnyUserData, usize)| {
					let items: Vec<AnyUserData> = me.named_user_value("items")?;

					let i = i + 1;
					Ok(if i > items.len() {
						mlua::Variadic::new()
					} else {
						let item = items[i - 1].clone().into_lua(lua)?;
						mlua::Variadic::from_iter([i.into_lua(lua)?, item])
					})
				})?;
				Ok((iter, me, 0))
			});
		})?;

		LUA.register_userdata_type::<core::files::File>(|reg| {
			reg.add_field_method_get("name", |_, me| {
				Ok(me.url().file_name().map(|n| n.to_string_lossy().to_string()))
			});
			reg.add_field_method_get("url", |_, me| Ok(Url::from(me.url())));
			reg.add_field_method_get("length", |_, me| Ok(me.length()));
			reg.add_field_method_get("link_to", |_, me| {
				Ok(me.link_to().map(|l| l.to_string_lossy().to_string()))
			});
			reg.add_field_method_get("is_link", |_, me| Ok(me.is_link()));
			reg.add_field_method_get("is_hidden", |_, me| Ok(me.is_hidden()));

			// Meta
			reg.add_field_method_get("permissions", |_, me| {
				Ok(shared::permissions(me.meta().permissions()))
			});
		})?;

		LUA.register_userdata_type::<core::manager::Preview>(|reg| {
			reg.add_field_function_get("folder", |_, me| me.named_user_value::<AnyUserData>("folder"));
		})?;

		Ok(())
	}

	pub(crate) fn make<'a>(
		scope: &mlua::Scope<'a, 'a>,
		inner: &'a core::manager::Manager,
	) -> mlua::Result<AnyUserData<'a>> {
		let ud = scope.create_any_userdata_ref(inner)?;
		ud.set_named_user_value("parent", inner.parent().and_then(|p| Self::folder(scope, p).ok()))?;
		ud.set_named_user_value("current", Self::folder(scope, inner.current())?)?;
		ud.set_named_user_value("preview", Self::preview(scope, inner.active())?)?;

		Ok(ud)
	}

	pub(crate) fn folder<'a>(
		scope: &mlua::Scope<'a, 'a>,
		inner: &'a core::manager::Folder,
	) -> mlua::Result<AnyUserData<'a>> {
		let ud = scope.create_any_userdata_ref(inner)?;
		ud.set_named_user_value("files", Self::files(scope, &inner.files)?)?;
		ud.set_named_user_value(
			"hovered",
			inner.hovered.as_ref().and_then(|h| Self::file(scope, h).ok()),
		)?;

		Ok(ud)
	}

	fn files<'a>(
		scope: &mlua::Scope<'a, 'a>,
		inner: &'a core::files::Files,
	) -> mlua::Result<AnyUserData<'a>> {
		let ud = scope.create_any_userdata_ref(inner)?;
		ud.set_named_user_value(
			"items",
			inner.iter().filter_map(|f| Self::file(scope, f).ok()).collect::<Vec<_>>(),
		)?;

		Ok(ud)
	}

	fn file<'a>(
		scope: &mlua::Scope<'a, 'a>,
		inner: &'a core::files::File,
	) -> mlua::Result<AnyUserData<'a>> {
		scope.create_any_userdata_ref(inner)
	}

	fn preview<'a>(
		scope: &mlua::Scope<'a, 'a>,
		tab: &'a core::manager::Tab,
	) -> mlua::Result<AnyUserData<'a>> {
		let inner = tab.preview();

		let ud = scope.create_any_userdata_ref(inner)?;
		ud.set_named_user_value(
			"folder",
			inner
				.lock
				.as_ref()
				.filter(|l| l.is_folder())
				.and_then(|l| tab.history(&l.url))
				.and_then(|f| Self::folder(scope, f).ok()),
		)?;

		Ok(ud)
	}
}
