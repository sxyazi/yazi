use std::ops::Deref;

use hashbrown::{HashMap, hash_map::Entry};
use mlua::{AnyUserData, ExternalError, IntoLuaMulti, MetaMethod, UserData, UserDataFields, UserDataMethods, UserDataRef};
use yazi_fs::{file::File, files::{FileSource, FilesInventory}};

use super::{Lives, PtrCell};

pub(super) struct Yanked {
	inner: PtrCell<yazi_core::mgr::Yanked>,
	cache: HashMap<usize, AnyUserData>,
}

impl Deref for Yanked {
	type Target = yazi_core::mgr::Yanked;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl Yanked {
	pub(super) fn make(inner: &yazi_core::mgr::Yanked) -> mlua::Result<AnyUserData> {
		Lives::scoped_userdata(Self { inner: inner.into(), cache: HashMap::new() })
	}
}

impl FileSource for Yanked {
	fn len(&self) -> usize { self.inner.len() }

	fn borrow(
		&self,
		idx: usize,
		visit: &mut dyn FnMut(&File) -> mlua::Result<()>,
	) -> mlua::Result<()> {
		visit(&self.get_index(idx).ok_or_else(|| "file index out of bounds".into_lua_err())?.0)
	}
}

impl UserData for Yanked {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("is_cut", |_, me| Ok(me.cut));
	}

	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_method(MetaMethod::Len, |_, me, ()| Ok(me.len()));

		methods.add_meta_method_mut(MetaMethod::Index, |lua, me, idx: usize| {
			let Some(file) = idx.checked_sub(1).and_then(|i| me.inner.get_index(i)) else {
				return Ok(None);
			};

			let cached = match me.cache.entry(idx) {
				Entry::Occupied(oe) => oe.into_mut(),
				Entry::Vacant(ve) => ve.insert(lua.create_userdata(file.0.clone())?),
			};
			Ok(Some(cached.clone()))
		});

		// TODO: remove
		methods.add_meta_function(MetaMethod::Pairs, |lua, ud: AnyUserData| {
			let next = lua.create_function(|lua, (me, idx): (UserDataRef<Self>, usize)| {
				match me.get_index(idx) {
					Some(file) => (idx + 1, file.0.clone()).into_lua_multi(lua),
					None => ().into_lua_multi(lua),
				}
			})?;
			Ok((next, ud, 0))
		});
	}
}

inventory::submit! {
	FilesInventory::new::<Yanked>()
}
