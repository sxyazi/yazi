use std::{ops::Deref, ptr};

use mlua::{AnyUserData, IntoLua, UserData, UserDataFields, UserDataMethods};
use yazi_binding::{Range, style::Style};
use yazi_config::THEME;
use yazi_fs::file::FileInventory;
use yazi_shared::{path::AsPath, url::UrlLike};

use super::{FILE_CACHE, Lives};
use crate::lives::{CoreRef, PtrCell};

pub(super) struct File {
	idx:    usize,
	folder: PtrCell<yazi_core::tab::Folder>,
	tab:    PtrCell<yazi_core::tab::Tab>,
}

impl Deref for File {
	type Target = yazi_fs::file::File;

	fn deref(&self) -> &Self::Target { &self.folder.entries[self.idx] }
}

impl AsRef<yazi_fs::file::File> for File {
	fn as_ref(&self) -> &yazi_fs::file::File { self }
}

impl File {
	pub(super) fn make(
		idx: usize,
		folder: &yazi_core::tab::Folder,
		tab: &yazi_core::tab::Tab,
	) -> mlua::Result<AnyUserData> {
		use hashbrown::hash_map::Entry;

		Ok(
			match unsafe { (*FILE_CACHE.get()).assume_init_mut() }.entry(PtrCell(&folder.entries[idx])) {
				Entry::Occupied(oe) => oe.into_mut().clone(),
				Entry::Vacant(ve) => {
					let ud = Lives::scoped_userdata(Self { idx, folder: folder.into(), tab: tab.into() })?;
					ve.insert(ud.clone());
					ud
				}
			},
		)
	}

	#[inline]
	fn is_hovered(&self) -> bool { self.idx == self.folder.cursor }
}

impl UserData for File {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		yazi_binding::impl_file_fields!(fields);

		fields.add_field_method_get("idx", |_, me| Ok(me.idx + 1));
		fields.add_field_method_get("is_hovered", |_, me| Ok(me.is_hovered()));
		fields.add_field_method_get("in_current", |_, me| Ok(ptr::eq(&*me.folder, &me.tab.current)));
		fields.add_field_method_get("in_preview", |_, me| {
			Ok(me.idx == me.folder.cursor && me.tab.hovered().is_some_and(|f| f.url == me.folder.url))
		});
	}

	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		yazi_binding::impl_file_methods!(methods);

		methods.add_method("icon", |lua, me, ()| {
			yazi_binding::deprecate!(
				lua,
				"{}: `File:icon()` is deprecated, use `th.icon:match(file)` instead"
			);
			// TODO: use a cache
			Ok(yazi_config::THEME.icon.matches(me, me.is_hovered()))
		});
		methods.add_method("size", |_, me, ()| {
			Ok(if me.is_dir() { me.folder.entries.sizes.get(&me.urn()).copied() } else { Some(me.len) })
		});
		methods.add_method("mime", |lua, me, ()| {
			let core: CoreRef = lua.named_registry_value("cx")?;
			core.mgr.mimetype.get(&me.url).map(|s| lua.create_string(s)).transpose()
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
			let core: CoreRef = lua.named_registry_value("cx")?;
			let mime = core.mgr.mimetype.get(&me.url).unwrap_or_default();
			Ok(THEME.filetype.match_style(me, mime).map(Style::from))
		});
		methods.add_method("is_yanked", |lua, me, ()| {
			let core: CoreRef = lua.named_registry_value("cx")?;
			Ok(if !core.mgr.yanked.contains(&me.url) {
				0u8
			} else if core.mgr.yanked.cut {
				2u8
			} else {
				1u8
			})
		});
		methods.add_method("is_marked", |_, me, ()| {
			let Some(visual) = me.tab.mode.visual() else {
				return Ok(0u8);
			};
			if !visual.contains(me.idx, me.tab.current.cursor, me.folder.entries.len()) {
				return Ok(0u8);
			}
			if me.folder.url != me.tab.current.url {
				return Ok(0u8);
			}
			Ok(if me.tab.mode.is_select() { 1u8 } else { 2u8 })
		});
		methods.add_method("is_selected", |_, me, ()| Ok(me.tab.selected.contains(&me.url)));
		methods.add_method("found", |lua, me, ()| {
			let core: CoreRef = lua.named_registry_value("cx")?;
			let Some(finder) = &core.active().finder else {
				return Ok(None);
			};

			let Some(idx) = finder.matched_idx(&me.folder, me.urn()) else {
				return Ok(None);
			};

			lua.create_sequence_from([idx.into_lua(lua)?, finder.matched.len().into_lua(lua)?]).map(Some)
		});
		methods.add_method("highlights", |lua, me, ()| {
			let core: CoreRef = lua.named_registry_value("cx")?;
			let Some(finder) = &core.active().finder else {
				return Ok(None);
			};
			if me.folder.url != me.tab.current.url {
				return Ok(None);
			}
			let Some(Some(h)) = me.url.name().map(|s| finder.filter.highlighted(s)) else {
				return Ok(None);
			};

			lua.create_sequence_from(h.into_iter().map(Range::from)).map(Some)
		});
	}
}

inventory::submit! {
	FileInventory {
		register: |_| {},
		borrow: |ud, f| f(&*ud.borrow::<File>()?),
	}
}
