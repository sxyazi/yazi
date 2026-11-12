use std::ops::Deref;

use hashbrown::{HashMap, hash_map::Entry};
use mlua::{AnyUserData, ExternalError, IntoLuaMulti, MetaMethod, UserData, UserDataMethods, UserDataRef};
use yazi_fs::{file::File, files::{FileSource, FilesInventory}};

use super::{Lives, PtrCell};

pub(super) struct Selected {
	inner: PtrCell<yazi_core::tab::Selected>,
	cache: HashMap<usize, AnyUserData>,
}

impl Deref for Selected {
	type Target = yazi_core::tab::Selected;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl Selected {
	pub(super) fn make(inner: &yazi_core::tab::Selected) -> mlua::Result<AnyUserData> {
		Lives::scoped_userdata(Self { inner: inner.into(), cache: HashMap::new() })
	}
}

impl FileSource for Selected {
	fn len(&self) -> usize { self.inner.len() }

	fn borrow(
		&self,
		idx: usize,
		visit: &mut dyn FnMut(&File) -> mlua::Result<()>,
	) -> mlua::Result<()> {
		visit(self.file_at(idx).ok_or_else(|| "file index out of bounds".into_lua_err())?)
	}
}

impl UserData for Selected {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_method(MetaMethod::Len, |_, me, ()| Ok(me.len()));

		methods.add_meta_method_mut(MetaMethod::Index, |lua, me, idx: usize| {
			let Some(file) = idx.checked_sub(1).and_then(|i| me.inner.file_at(i)) else {
				return Ok(None);
			};

			let cached = match me.cache.entry(idx) {
				Entry::Occupied(oe) => oe.into_mut(),
				Entry::Vacant(ve) => ve.insert(lua.create_userdata(file.clone())?),
			};
			Ok(Some(cached.clone()))
		});

		// TODO: remove
		methods.add_meta_function(MetaMethod::Pairs, |lua, ud: AnyUserData| {
			let next = lua.create_function(|lua, (me, idx): (UserDataRef<Self>, usize)| {
				match me.file_at(idx) {
					Some(file) => (idx + 1, file.clone()).into_lua_multi(lua),
					None => ().into_lua_multi(lua),
				}
			})?;
			Ok((next, ud, 0))
		});
	}
}

inventory::submit! {
	FilesInventory::new::<Selected>()
}
