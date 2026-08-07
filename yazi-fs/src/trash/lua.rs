use std::io;

use mlua::{ExternalError, ExternalResult, FromLua, IntoLuaMulti, LuaString, UserData, UserDataMethods, Value};
use tokio::task::spawn_blocking;
use yazi_shared::path::{PathBufDyn, PathLike};
use yazi_shim::fs::Error;

use super::{Trash, TrashEntries, TrashEntry, TrashId};
use crate::file::File;

impl UserData for Trash {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_async_function("empty", |lua, ()| async move {
			match spawn_blocking(|| Trash::new()?.empty()).await.into_lua_err()? {
				Ok(()) => true.into_lua_multi(&lua),
				Err(e) => (false, Error::from(e)).into_lua_multi(&lua),
			}
		});

		methods.add_async_function("entry", |lua, value: Value| async move {
			if let Value::UserData(ud) = value {
				return ud.borrow::<TrashEntry>()?.clone().into_lua_multi(&lua);
			}

			let id = TrashId::from_lua(value, &lua)?;
			match spawn_blocking(move || Trash::new()?.entry(&id)).await.into_lua_err()? {
				Ok(entry) => entry.into_lua_multi(&lua),
				Err(e) => (Value::Nil, Error::from(e)).into_lua_multi(&lua),
			}
		});

		methods.add_async_function("list", |lua, entry: Option<TrashEntry>| async move {
			match spawn_blocking(move || Trash::new()?.list(entry.as_ref())).await.into_lua_err()? {
				Ok(items) => lua.create_sequence_from(items)?.into_lua_multi(&lua),
				Err(e) => (Value::Nil, Error::from(e)).into_lua_multi(&lua),
			}
		});

		methods.add_async_function("metadata", |lua, (entry, follow): (TrashEntry, bool)| async move {
			match spawn_blocking(move || Trash::new()?.metadata(&entry, follow)).await.into_lua_err()? {
				Ok(cha) => cha.into_lua_multi(&lua),
				Err(e) => (Value::Nil, Error::from(e)).into_lua_multi(&lua),
			}
		});

		methods.add_async_function(
			"remove",
			|lua, (kind, entry): (LuaString, TrashEntry)| async move {
				let f: fn(&Trash, &TrashEntry) -> io::Result<()> = match &*kind.as_bytes() {
					b"file" => Trash::remove_file,
					b"dir" => Trash::remove_dir,
					_ => Err("Removal type must be 'file' or 'dir'".into_lua_err())?,
				};

				match spawn_blocking(move || f(&Trash::new()?, &entry)).await.into_lua_err()? {
					Ok(()) => true.into_lua_multi(&lua),
					Err(e) => (false, Error::from(e)).into_lua_multi(&lua),
				}
			},
		);

		methods.add_async_function(
			"rename",
			|lua, (entry, path): (TrashEntry, PathBufDyn)| async move {
				match spawn_blocking(move || Trash::new()?.rename(&entry, path.as_os()?))
					.await
					.into_lua_err()?
				{
					Ok(()) => true.into_lua_multi(&lua),
					Err(e) => (false, Error::from(e)).into_lua_multi(&lua),
				}
			},
		);

		methods.add_async_function("restore", |lua, entries: TrashEntries| async move {
			match spawn_blocking(move || Trash::new()?.restore(entries)).await.into_lua_err()? {
				Ok(()) => true.into_lua_multi(&lua),
				Err(e) => (false, Error::from(e)).into_lua_multi(&lua),
			}
		});

		methods.add_async_function(
			"revalidate",
			|lua, (entry, file): (Option<TrashEntry>, File)| async move {
				match spawn_blocking(move || Trash::new()?.revalidate(entry.as_ref(), &file))
					.await
					.into_lua_err()?
				{
					Ok(file) => file.into_lua_multi(&lua),
					Err(e) => (Value::Nil, Error::from(e)).into_lua_multi(&lua),
				}
			},
		);
	}
}
