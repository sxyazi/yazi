use std::{ops::Deref, ptr};

use mlua::{AnyUserData, IntoLua, UserData, UserDataFields, UserDataMethods, Value};
use yazi_binding::{Style, cached_field};
use yazi_config::THEME;
use yazi_plugin::bindings::Range;
use yazi_shared::{path::AsPath, url::UrlLike};

use super::Lives;
use crate::lives::PtrCell;

pub(super) struct File {
	idx:    usize,
	folder: PtrCell<yazi_core::tab::Folder>,
	tab:    PtrCell<yazi_core::tab::Tab>,

	v_cha:     Option<Value>,
	v_url:     Option<Value>,
	v_link_to: Option<Value>,

	v_name:  Option<Value>,
	v_path:  Option<Value>,
	v_cache: Option<Value>,

	v_bare: Option<Value>,
}

impl Deref for File {
	type Target = yazi_fs::File;

	fn deref(&self) -> &Self::Target { &self.folder.files[self.idx] }
}

impl AsRef<yazi_fs::File> for File {
	fn as_ref(&self) -> &yazi_fs::File { self }
}

impl File {
	pub(super) fn make(
		idx: usize,
		folder: &yazi_core::tab::Folder,
		tab: &yazi_core::tab::Tab,
	) -> mlua::Result<AnyUserData> {
		use hashbrown::hash_map::Entry;

		Ok(match super::FILE_CACHE.borrow_mut().entry(PtrCell(&folder.files[idx])) {
			Entry::Occupied(oe) => oe.into_mut().clone(),
			Entry::Vacant(ve) => {
				let ud = Lives::scoped_userdata(Self {
					idx,
					folder: folder.into(),
					tab: tab.into(),

					v_cha: None,
					v_url: None,
					v_link_to: None,

					v_name: None,
					v_path: None,
					v_cache: None,

					v_bare: None,
				})?;
				ve.insert(ud.clone());
				ud
			}
		})
	}
}

impl UserData for File {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		yazi_binding::impl_file_fields!(fields);
		cached_field!(fields, bare, |_, me| Ok(yazi_binding::File::new(&**me)));

		fields.add_field_method_get("idx", |_, me| Ok(me.idx + 1));
		fields.add_field_method_get("is_hovered", |_, me| Ok(me.idx == me.folder.cursor));
		fields.add_field_method_get("in_current", |_, me| Ok(ptr::eq(&*me.folder, &me.tab.current)));
		fields.add_field_method_get("in_preview", |_, me| {
			Ok(me.idx == me.folder.cursor && me.tab.hovered().is_some_and(|f| f.url == me.folder.url))
		});
	}

	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		yazi_binding::impl_file_methods!(methods);

		methods.add_method("size", |_, me, ()| {
			Ok(if me.is_dir() { me.folder.files.sizes.get(&me.urn()).copied() } else { Some(me.len) })
		});
		methods.add_method("mime", |lua, me, ()| {
			lua.named_registry_value::<AnyUserData>("cx")?.borrow_scoped(|core: &yazi_core::Core| {
				core.mgr.mimetype.get(&me.url).map(|s| lua.create_string(s)).transpose()
			})?
		});
		methods.add_method("prefix", |lua, me, ()| {
			if !me.url.has_trail() {
				return Ok(None);
			}

			let mut comp = me.url.try_strip_prefix(me.url.trail()).unwrap_or(me.url.loc()).components();
			comp.next_back();
			Some(lua.create_string(comp.as_path().encoded_bytes())).transpose()
		});
		methods.add_method("style", |lua, me, ()| {
			lua.named_registry_value::<AnyUserData>("cx")?.borrow_scoped(|core: &yazi_core::Core| {
				let mime = core.mgr.mimetype.get(&me.url).unwrap_or_default();
				THEME.filetype.iter().find(|&x| x.matches(me, mime)).map(|x| Style::from(x.style))
			})
		});
		methods.add_method("is_yanked", |lua, me, ()| {
			lua.named_registry_value::<AnyUserData>("cx")?.borrow_scoped(|core: &yazi_core::Core| {
				if !core.mgr.yanked.contains(&me.url) {
					0u8
				} else if core.mgr.yanked.cut {
					2u8
				} else {
					1u8
				}
			})
		});
		methods.add_method("is_marked", |_, me, ()| {
			use yazi_core::tab::Mode::*;
			if !me.tab.mode.is_visual() || me.folder.url != me.tab.current.url {
				return Ok(0u8);
			}

			Ok(match &me.tab.mode {
				Select(_, indices) if indices.contains(&me.idx) => 1u8,
				Unset(_, indices) if indices.contains(&me.idx) => 2u8,
				_ => 0u8,
			})
		});
		methods.add_method("is_selected", |_, me, ()| Ok(me.tab.selected.contains(&me.url)));
		methods.add_method("found", |lua, me, ()| {
			lua.named_registry_value::<AnyUserData>("cx")?.borrow_scoped(|core: &yazi_core::Core| {
				let Some(finder) = &core.active().finder else {
					return Ok(None);
				};

				let Some(idx) = finder.matched_idx(&me.folder, me.urn()) else {
					return Ok(None);
				};

				Some(lua.create_sequence_from([idx.into_lua(lua)?, finder.matched.len().into_lua(lua)?]))
					.transpose()
			})
		});
		methods.add_method("highlights", |lua, me, ()| {
			lua.named_registry_value::<AnyUserData>("cx")?.borrow_scoped(|core: &yazi_core::Core| {
				let Some(finder) = &core.active().finder else {
					return None;
				};
				if me.folder.url != me.tab.current.url {
					return None;
				}

				let h = finder.filter.highlighted(me.url.name()?)?;
				Some(h.into_iter().map(Range::from).collect::<Vec<_>>())
			})
		});
	}
}
