use std::ops::Deref;

use mlua::{AnyUserData, IntoLua, UserData, UserDataFields, UserDataMethods};
use yazi_config::THEME;
use yazi_plugin::{bindings::Range, elements::Style};
use yazi_shared::MIME_DIR;

use super::SCOPE;
use crate::Ctx;

pub(super) struct File {
	idx:    usize,
	folder: *const yazi_fs::Folder,
	tab:    *const yazi_core::tab::Tab,
}

impl Deref for File {
	type Target = yazi_shared::fs::File;

	fn deref(&self) -> &Self::Target { &self.folder().files[self.idx] }
}

impl AsRef<yazi_shared::fs::File> for File {
	fn as_ref(&self) -> &yazi_shared::fs::File { self }
}

impl File {
	#[inline]
	pub(super) fn make(
		idx: usize,
		folder: &yazi_fs::Folder,
		tab: &yazi_core::tab::Tab,
	) -> mlua::Result<AnyUserData> {
		SCOPE.create_userdata(Self { idx, folder, tab })
	}

	#[inline]
	fn folder(&self) -> &yazi_fs::Folder { unsafe { &*self.folder } }

	#[inline]
	fn tab(&self) -> &yazi_core::tab::Tab { unsafe { &*self.tab } }
}

impl UserData for File {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		yazi_plugin::impl_file_fields!(fields);

		fields.add_field_method_get("idx", |_, me| Ok(me.idx + 1));
	}

	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		yazi_plugin::impl_file_methods!(methods);

		methods.add_method("size", |_, me, ()| {
			Ok(if me.is_dir() { me.folder().files.sizes.get(me.urn()).copied() } else { Some(me.len) })
		});
		methods.add_method("mime", |lua, me, ()| {
			lua
				.named_registry_value::<AnyUserData>("cx")?
				.borrow_scoped(|cx: &Ctx| cx.manager.mimetype.get_owned(&me.url))
		});
		methods.add_method("prefix", |lua, me, ()| {
			if !me.folder().url.is_search() {
				return Ok(None);
			}

			let mut p = me.url.strip_prefix(&me.folder().url).unwrap_or(&me.url).components();
			p.next_back();
			Some(lua.create_string(p.as_path().as_os_str().as_encoded_bytes())).transpose()
		});
		methods.add_method("style", |lua, me, ()| {
			lua.named_registry_value::<AnyUserData>("cx")?.borrow_scoped(|cx: &Ctx| {
				let mime =
					if me.is_dir() { MIME_DIR } else { cx.manager.mimetype.get(&me.url).unwrap_or_default() };

				THEME.filetypes.iter().find(|&x| x.matches(me, mime)).map(|x| Style::from(x.style))
			})
		});
		methods.add_method("is_hovered", |_, me, ()| Ok(me.idx == me.folder().cursor));
		methods.add_method("is_yanked", |lua, me, ()| {
			lua.named_registry_value::<AnyUserData>("cx")?.borrow_scoped(|cx: &Ctx| {
				if !cx.manager.yanked.contains(&me.url) {
					0u8
				} else if cx.manager.yanked.cut {
					2u8
				} else {
					1u8
				}
			})
		});
		methods.add_method("is_marked", |_, me, ()| {
			use yazi_core::tab::Mode::*;
			if !me.tab().mode.is_visual() || me.folder().url != me.tab().current.url {
				return Ok(0u8);
			}

			Ok(match &me.tab().mode {
				Select(_, indices) if indices.contains(&me.idx) => 1u8,
				Unset(_, indices) if indices.contains(&me.idx) => 2u8,
				_ => 0u8,
			})
		});
		methods.add_method("is_selected", |_, me, ()| Ok(me.tab().selected.contains_key(&me.url)));
		methods.add_method("in_parent", |_, me, ()| {
			Ok(me.tab().parent.as_ref().is_some_and(|f| me.folder().url == f.url))
		});
		methods.add_method("in_current", |_, me, ()| Ok(me.folder().url == me.tab().current.url));
		methods.add_method("in_preview", |_, me, ()| {
			Ok(me.tab().hovered().is_some_and(|f| f.url == me.folder().url))
		});
		methods.add_method("found", |lua, me, ()| {
			lua.named_registry_value::<AnyUserData>("cx")?.borrow_scoped(|cx: &Ctx| {
				let Some(finder) = &cx.manager.active().finder else {
					return Ok(None);
				};

				let Some(idx) = finder.matched_idx(&me.url) else {
					return Ok(None);
				};

				Some(lua.create_sequence_from([idx.into_lua(lua)?, finder.matched().len().into_lua(lua)?]))
					.transpose()
			})
		});
		methods.add_method("highlights", |lua, me, ()| {
			lua.named_registry_value::<AnyUserData>("cx")?.borrow_scoped(|cx: &Ctx| {
				let Some(finder) = &cx.manager.active().finder else {
					return None;
				};
				if me.folder().url != me.tab().current.url {
					return None;
				}

				let h = finder.filter.highlighted(me.name())?;
				Some(h.into_iter().map(Range::from).collect::<Vec<_>>())
			})
		});
	}
}
