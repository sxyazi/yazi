use std::ops::Deref;

use mlua::{AnyUserData, IntoLua, Lua, UserDataFields, UserDataMethods};
use yazi_config::THEME;
use yazi_plugin::{bindings::{Cast, Cha, Icon, Range}, elements::Style, url::Url};
use yazi_shared::MIME_DIR;

use super::{CtxRef, SCOPE};

pub(super) struct File {
	idx:    usize,
	folder: *const yazi_core::folder::Folder,
	tab:    *const yazi_core::tab::Tab,
}

impl Deref for File {
	type Target = yazi_shared::fs::File;

	fn deref(&self) -> &Self::Target { &self.folder().files[self.idx] }
}

impl File {
	#[inline]
	pub(super) fn make(
		idx: usize,
		folder: &yazi_core::folder::Folder,
		tab: &yazi_core::tab::Tab,
	) -> mlua::Result<AnyUserData<'static>> {
		SCOPE.create_any_userdata(Self { idx, folder, tab })
	}

	pub(super) fn register(lua: &Lua) -> mlua::Result<()> {
		lua.register_userdata_type::<Self>(|reg| {
			reg.add_field_method_get("url", |lua, me| Url::cast(lua, me.url.clone()));
			reg.add_field_method_get("cha", |lua, me| Cha::cast(lua, me.cha));
			reg.add_field_method_get("link_to", |lua, me| {
				me.link_to.as_ref().cloned().map(|u| Url::cast(lua, u)).transpose()
			});

			reg.add_field_method_get("name", |lua, me| {
				me.url.file_name().map(|n| lua.create_string(n.as_encoded_bytes())).transpose()
			});
			reg.add_method("size", |_, me, ()| {
				Ok(if me.is_dir() { me.folder().files.sizes.get(&me.url).copied() } else { Some(me.len) })
			});
			reg.add_method("mime", |lua, me, ()| {
				let cx = lua.named_registry_value::<CtxRef>("cx")?;
				Ok(cx.manager.mimetype.get(&me.url).cloned())
			});
			reg.add_method("prefix", |lua, me, ()| {
				if !me.folder().cwd.is_search() {
					return Ok(None);
				}

				let mut p = me.url.strip_prefix(&me.folder().cwd).unwrap_or(&me.url).components();
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
			reg.add_method("style", |lua, me, ()| {
				let cx = lua.named_registry_value::<CtxRef>("cx")?;
				let mime =
					if me.is_dir() { Some(MIME_DIR) } else { cx.manager.mimetype.get(&me.url).map(|x| &**x) };

				Ok(THEME.filetypes.iter().find(|&x| x.matches(me, mime)).map(|x| Style::from(x.style)))
			});
			reg.add_method("is_hovered", |_, me, ()| {
				Ok(matches!(me.folder().hovered(), Some(f) if f.url == me.url))
			});
			reg.add_method("is_yanked", |lua, me, ()| {
				let cx = lua.named_registry_value::<CtxRef>("cx")?;
				Ok(if !cx.manager.yanked.contains(&me.url) {
					0u8
				} else if cx.manager.yanked.cut {
					2u8
				} else {
					1u8
				})
			});
			reg.add_method("is_selected", |lua, me, ()| {
				let cx = lua.named_registry_value::<CtxRef>("cx")?;
				let selected = me.tab().selected.contains(&me.url);

				#[allow(clippy::if_same_then_else)]
				Ok(if !cx.manager.active().mode.is_visual() {
					selected
				} else if me.folder().cwd != me.tab().current.cwd {
					selected
				} else {
					cx.manager.active().mode.pending(me.idx, selected)
				})
			});
			reg.add_method("found", |lua, me, ()| {
				let cx = lua.named_registry_value::<CtxRef>("cx")?;
				let Some(finder) = &cx.manager.active().finder else {
					return Ok(None);
				};

				let Some(idx) = finder.matched_idx(&me.url) else {
					return Ok(None);
				};

				Some(lua.create_sequence_from([idx.into_lua(lua)?, finder.matched().len().into_lua(lua)?]))
					.transpose()
			});
			reg.add_method("highlights", |lua, me, ()| {
				let cx = lua.named_registry_value::<CtxRef>("cx")?;
				let Some(finder) = &cx.manager.active().finder else {
					return Ok(None);
				};

				let Some(h) = me.name().and_then(|n| finder.filter.highlighted(n)) else {
					return Ok(None);
				};

				Ok(Some(h.into_iter().map(Range::from).collect::<Vec<_>>()))
			});
		})?;

		Ok(())
	}

	#[inline]
	fn folder(&self) -> &yazi_core::folder::Folder { unsafe { &*self.folder } }

	#[inline]
	fn tab(&self) -> &yazi_core::tab::Tab { unsafe { &*self.tab } }
}
