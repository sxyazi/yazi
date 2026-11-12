use std::borrow::Cow;

use hashbrown::{HashMap, hash_map::Entry};
use mlua::{AnyUserData, ExternalError, FromLua, IntoLua, IntoLuaMulti, Lua, MetaMethod, UserData, UserDataFields, UserDataMethods, UserDataRef, Value};
use serde::{Deserialize, Serialize};
use yazi_fs::{file::{File, FileCov}, files::{FileSource, FilesInventory}};
use yazi_macro::impl_data_any;
use yazi_shim::indexmap::IndexSet;

use super::Ember;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EmberYank<'a> {
	pub cut:   bool,
	pub files: Cow<'a, IndexSet<FileCov>>,
}

impl_data_any!(EmberYank<'static>, from_into_lua = inherit);

impl<'a> EmberYank<'a> {
	pub(crate) fn borrowed(cut: bool, files: &'a IndexSet<FileCov>) -> Ember<'a> {
		Self { cut, files: Cow::Borrowed(files) }.into()
	}
}

impl EmberYank<'static> {
	pub(crate) fn owned(cut: bool, _: &IndexSet<FileCov>) -> Ember<'static> {
		Self { cut, files: Default::default() }.into()
	}
}

impl<'a> From<EmberYank<'a>> for Ember<'a> {
	fn from(value: EmberYank<'a>) -> Self { Self::Yank(value) }
}

impl FromLua for EmberYank<'static> {
	fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
		match value {
			Value::UserData(ud) => Ok(ud.take::<EmberYankFiles>()?.into()),
			_ => Err(mlua::Error::FromLuaConversionError {
				from:    value.type_name(),
				to:      "EmberYank".to_owned(),
				message: Some("expected EmberYankFiles userdata".to_owned()),
			}),
		}
	}
}

impl IntoLua for EmberYank<'_> {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		EmberYankFiles { cut: self.cut, files: self.files.into_owned(), cache: HashMap::new() }
			.into_lua(lua)
	}
}

// --- Files
pub struct EmberYankFiles {
	cut:   bool,
	files: IndexSet<FileCov>,
	cache: HashMap<usize, AnyUserData>,
}

impl From<EmberYankFiles> for EmberYank<'static> {
	fn from(value: EmberYankFiles) -> Self { Self { cut: value.cut, files: Cow::Owned(value.files) } }
}

impl FileSource for EmberYankFiles {
	fn len(&self) -> usize { self.files.len() }

	fn borrow(
		&self,
		idx: usize,
		visit: &mut dyn FnMut(&File) -> mlua::Result<()>,
	) -> mlua::Result<()> {
		visit(&self.files.get_index(idx).ok_or_else(|| "file index out of bounds".into_lua_err())?.0)
	}
}

impl UserData for EmberYankFiles {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("cut", |_, me| Ok(me.cut));
	}

	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_method(MetaMethod::Len, |_, me, ()| Ok(me.files.len()));

		methods.add_meta_method_mut(MetaMethod::Index, |lua, me, idx: usize| {
			let Some(file) = idx.checked_sub(1).and_then(|i| me.files.get_index(i)) else {
				return Ok(None);
			};

			let cached = match me.cache.entry(idx) {
				Entry::Occupied(oe) => oe.into_mut(),
				// TODO: remove .clone()
				Entry::Vacant(ve) => ve.insert(lua.create_userdata(file.0.clone())?),
			};
			Ok(Some(cached.clone()))
		});

		// TODO: remove
		methods.add_meta_function(MetaMethod::Pairs, |lua, ud: AnyUserData| {
			let next = lua.create_function(|lua, (me, idx): (UserDataRef<Self>, usize)| {
				match me.files.get_index(idx) {
					Some(file) => (idx + 1, file.0.clone()).into_lua_multi(lua),
					None => ().into_lua_multi(lua),
				}
			})?;
			Ok((next, ud, 0))
		});
	}
}

inventory::submit! {
	FilesInventory::new::<EmberYankFiles>()
}
