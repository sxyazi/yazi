use mlua::{AnyUserData, IntoLua, Lua, MetaMethod, UserDataFields, UserDataMethods, Value};
use yazi_config::{LAYOUT, THEME};
use yazi_plugin::{bindings::{Cast, File, Icon, Range, Url}, elements::Style};
use yazi_shared::MIME_DIR;

use super::{CtxRef, FolderRef};

pub struct Folder<'a, 'b> {
	scope: &'b mlua::Scope<'a, 'a>,

	inner: &'a yazi_core::folder::Folder,
}

impl<'a, 'b> Folder<'a, 'b> {
	pub(super) fn register(lua: &Lua) -> mlua::Result<()> {
		lua.register_userdata_type::<yazi_core::folder::Folder>(|reg| {
			reg.add_field_method_get("cwd", |lua, me| Url::cast(lua, me.cwd.clone()));
			reg.add_field_method_get("offset", |_, me| Ok(me.offset));
			reg.add_field_method_get("cursor", |_, me| Ok(me.cursor));

			reg.add_field_function_get("window", |_, me| me.named_user_value::<Value>("window"));
			reg.add_field_function_get("files", |_, me| me.named_user_value::<AnyUserData>("files"));
			reg.add_field_function_get("hovered", |_, me| me.named_user_value::<Value>("hovered"));
		})?;

		lua.register_userdata_type::<yazi_core::folder::Files>(|reg| {
			reg.add_meta_method(MetaMethod::Len, |_, me, ()| Ok(me.len()));

			reg.add_meta_function(MetaMethod::Pairs, |lua, me: AnyUserData| {
				let iter = lua.create_function(|lua, (me, i): (AnyUserData, usize)| {
					let files = me.borrow::<yazi_core::folder::Files>()?;
					let i = i + 1;
					Ok(if i > files.len() {
						mlua::Variadic::new()
					} else {
						mlua::Variadic::from_iter([
							i.into_lua(lua)?,
							File::cast(lua, files[i - 1].clone())?.into_lua(lua)?,
						])
					})
				})?;
				Ok((iter, me, 0))
			});
		})?;

		File::register(lua, |reg| {
			reg.add_function("size", |_, me: AnyUserData| {
				let file = me.borrow::<yazi_shared::fs::File>()?;
				if !file.is_dir() {
					return Ok(Some(file.len));
				}

				let folder = me.named_user_value::<FolderRef>("folder")?;
				Ok(folder.files.sizes.get(&file.url).copied())
			});
			reg.add_function("mime", |lua, me: AnyUserData| {
				let cx = lua.named_registry_value::<CtxRef>("cx")?;
				let file = me.borrow::<yazi_shared::fs::File>()?;
				Ok(cx.manager.mimetype.get(&file.url).cloned())
			});
			reg.add_function("prefix", |lua, me: AnyUserData| {
				let folder = me.named_user_value::<FolderRef>("folder")?;
				if !folder.cwd.is_search() {
					return Ok(None);
				}

				let file = me.borrow::<yazi_shared::fs::File>()?;
				let mut p = file.url.strip_prefix(&folder.cwd).unwrap_or(&file.url).components();
				p.next_back();
				Some(lua.create_string(p.as_path().as_os_str().as_encoded_bytes())).transpose()
			});
			reg.add_method("icon", |lua, me, ()| {
				THEME
					.icons
					.iter()
					.find(|&x| x.name.match_path(&me.url, me.is_dir()))
					.map(|x| Icon::cast(lua, x))
					.transpose()
			});
			reg.add_function("style", |lua, me: AnyUserData| {
				let cx = lua.named_registry_value::<CtxRef>("cx")?;
				let file = me.borrow::<yazi_shared::fs::File>()?;

				let mime = if file.is_dir() {
					Some(MIME_DIR)
				} else {
					cx.manager.mimetype.get(&file.url).map(|x| &**x)
				};

				Ok(
					THEME
						.filetypes
						.iter()
						.find(|&x| x.matches(&file, mime))
						.map(|x| Style::from(x.style)),
				)
			});
			reg.add_function("is_hovered", |_, me: AnyUserData| {
				let folder = me.named_user_value::<FolderRef>("folder")?;
				let file = me.borrow::<yazi_shared::fs::File>()?;
				Ok(matches!(folder.hovered(), Some(f) if f.url == file.url))
			});
			reg.add_function("is_yanked", |lua, me: AnyUserData| {
				let cx = lua.named_registry_value::<CtxRef>("cx")?;
				let file = me.borrow::<yazi_shared::fs::File>()?;
				Ok(if !cx.manager.yanked.1.contains(&file.url) {
					0u8
				} else if cx.manager.yanked.0 {
					2u8
				} else {
					1u8
				})
			});
			reg.add_function("is_selected", |lua, me: AnyUserData| {
				let cx = lua.named_registry_value::<CtxRef>("cx")?;
				let folder = me.named_user_value::<FolderRef>("folder")?;
				let file = me.borrow::<yazi_shared::fs::File>()?;

				let selected = folder.files.is_selected(&file.url);
				Ok(if !cx.manager.active().mode.is_visual() {
					selected
				} else {
					let idx: usize = me.named_user_value("idx")?;
					cx.manager.active().mode.pending(folder.offset + idx, selected)
				})
			});
			reg.add_function("found", |lua, me: AnyUserData| {
				let cx = lua.named_registry_value::<CtxRef>("cx")?;
				let Some(finder) = &cx.manager.active().finder else {
					return Ok(None);
				};

				let file = me.borrow::<yazi_shared::fs::File>()?;
				if let Some(idx) = finder.matched_idx(&file.url) {
					return Some(
						lua.create_sequence_from([idx.into_lua(lua)?, finder.matched().len().into_lua(lua)?]),
					)
					.transpose();
				}
				Ok(None)
			});
			reg.add_function("highlights", |lua, me: AnyUserData| {
				let cx = lua.named_registry_value::<CtxRef>("cx")?;
				let Some(finder) = &cx.manager.active().finder else {
					return Ok(None);
				};

				let file = me.borrow::<yazi_shared::fs::File>()?;
				let Some(h) = file.name().and_then(|n| finder.filter.highlighted(n)) else {
					return Ok(None);
				};

				Ok(Some(h.into_iter().map(Range::from).collect::<Vec<_>>()))
			});
		})?;

		Ok(())
	}
}

impl<'a, 'b> Folder<'a, 'b> {
	pub(crate) fn new(scope: &'b mlua::Scope<'a, 'a>, inner: &'a yazi_core::folder::Folder) -> Self {
		Self { scope, inner }
	}

	pub(crate) fn make(&self, window: Option<(usize, usize)>) -> mlua::Result<AnyUserData<'a>> {
		let window =
			window.unwrap_or_else(|| (self.inner.offset, LAYOUT.load().preview.height as usize));

		let ud = self.scope.create_any_userdata_ref(self.inner)?;
		ud.set_named_user_value(
			"window",
			self
				.inner
				.files
				.iter()
				.skip(window.0)
				.take(window.1)
				.enumerate()
				.filter_map(|(i, f)| self.file(i, f).ok())
				.collect::<Vec<_>>(),
		)?;
		ud.set_named_user_value("files", self.scope.create_any_userdata_ref(&self.inner.files)?)?;
		ud.set_named_user_value(
			"hovered",
			self.inner.hovered().and_then(|h| self.file(self.inner.cursor - window.0, h).ok()),
		)?;

		Ok(ud)
	}

	fn file(&self, idx: usize, inner: &'a yazi_shared::fs::File) -> mlua::Result<AnyUserData<'a>> {
		let ud = self.scope.create_any_userdata_ref(inner)?;
		ud.set_named_user_value("idx", idx)?;
		ud.set_named_user_value("folder", self.scope.create_any_userdata_ref(self.inner)?)?;

		Ok(ud)
	}
}
