use std::io;

use mlua::{ExternalError, ExternalResult, IntoLuaMulti, LuaString, UserData, UserDataMethods, Value};
use tokio::task::spawn_blocking;
use yazi_binding::Error;

use super::{Trash, TrashNode, TrashNodes};
use crate::file::File;

impl UserData for Trash {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_async_function("empty", |lua, ()| async move {
			match spawn_blocking(|| Trash::new()?.empty()).await.into_lua_err()? {
				Ok(()) => true.into_lua_multi(&lua),
				Err(e) => (false, Error::Io(e)).into_lua_multi(&lua),
			}
		});

		methods.add_async_function("list", |lua, node: Option<TrashNode>| async move {
			match spawn_blocking(move || Trash::new()?.list(node.as_ref())).await.into_lua_err()? {
				Ok(items) => lua.create_sequence_from(items)?.into_lua_multi(&lua),
				Err(e) => (Value::Nil, Error::Io(e)).into_lua_multi(&lua),
			}
		});

		methods.add_async_function("entry", |lua, node: TrashNode| async move {
			match spawn_blocking(move || Trash::new()?.entry(&node)).await.into_lua_err()? {
				Ok(entry) => entry.into_lua_multi(&lua),
				Err(e) => (Value::Nil, Error::Io(e)).into_lua_multi(&lua),
			}
		});

		methods.add_async_function("metadata", |lua, (node, follow): (TrashNode, bool)| async move {
			match spawn_blocking(move || Trash::new()?.metadata(&node, follow)).await.into_lua_err()? {
				Ok(cha) => cha.into_lua_multi(&lua),
				Err(e) => (Value::Nil, Error::Io(e)).into_lua_multi(&lua),
			}
		});

		methods.add_async_function(
			"revalidate",
			|lua, (node, file): (Option<TrashNode>, File)| async move {
				match spawn_blocking(move || Trash::new()?.revalidate(node.as_ref(), &file))
					.await
					.into_lua_err()?
				{
					Ok(file) => file.into_lua_multi(&lua),
					Err(e) => (Value::Nil, Error::Io(e)).into_lua_multi(&lua),
				}
			},
		);

		methods.add_async_function("remove", |lua, (kind, node): (LuaString, TrashNode)| async move {
			let f: fn(&Trash, &TrashNode) -> io::Result<()> = match &*kind.as_bytes() {
				b"file" => Trash::remove_file,
				b"dir" => Trash::remove_dir,
				_ => Err("Removal type must be 'file' or 'dir'".into_lua_err())?,
			};

			match spawn_blocking(move || f(&Trash::new()?, &node)).await.into_lua_err()? {
				Ok(()) => true.into_lua_multi(&lua),
				Err(e) => (false, Error::Io(e)).into_lua_multi(&lua),
			}
		});

		methods.add_async_function("restore", |lua, nodes: TrashNodes| async move {
			match spawn_blocking(move || Trash::new()?.restore(nodes)).await.into_lua_err()? {
				Ok(()) => true.into_lua_multi(&lua),
				Err(e) => (false, Error::Io(e)).into_lua_multi(&lua),
			}
		});
	}
}
