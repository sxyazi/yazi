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
			reg.add_field_method_get("idx", |_, me| Ok(me.idx + 1));
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
				use yazi_shared::theme::IconCache;

				match me.icon.get() {
					IconCache::Missing => {
						let matched = THEME.icons.iter().find(|&i| i.matches(me));
						me.icon.set(matched.map_or(IconCache::Undefined, |i| IconCache::Icon(i)));
						matched.map(|i| Icon::cast(lua, i)).transpose()
					}
					IconCache::Undefined => Ok(None),
					IconCache::Icon(cached) => Some(Icon::cast(lua, cached)).transpose(),
				}
			});
			reg.add_method("style", |lua, me, ()| {
				let cx = lua.named_registry_value::<CtxRef>("cx")?;
				let mime =
					if me.is_dir() { Some(MIME_DIR) } else { cx.manager.mimetype.get(&me.url).map(|x| &**x) };

				Ok(THEME.filetypes.iter().find(|&x| x.matches(me, mime)).map(|x| Style::from(x.style)))
			});
			reg.add_method("is_hovered", |_, me, ()| Ok(me.idx == me.folder().cursor));
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
			reg.add_method("is_marked", |_, me, ()| {
				use yazi_core::tab::Mode::*;
				if !me.tab().mode.is_visual() || me.folder().cwd != me.tab().current.cwd {
					return Ok(0u8);
				}

				Ok(match &me.tab().mode {
					Select(_, indices) if indices.contains(&me.idx) => 1u8,
					Unset(_, indices) if indices.contains(&me.idx) => 2u8,
					_ => 0u8,
				})
			});
			reg.add_method("is_selected", |_, me, ()| Ok(me.tab().selected.contains_key(&me.url)));
			reg.add_method("in_parent", |_, me, ()| {
				Ok(me.tab().parent.as_ref().is_some_and(|f| me.folder().cwd == f.cwd))
			});
			reg.add_method("in_current", |_, me, ()| Ok(me.folder().cwd == me.tab().current.cwd));
			reg.add_method("in_preview", |_, me, ()| {
				Ok(me.tab().current.hovered().is_some_and(|f| me.folder().cwd == f.url))
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
				if me.folder().cwd != me.tab().current.cwd {
					return Ok(None);
				}
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
