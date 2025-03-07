use globset::GlobBuilder;
use mlua::{ExternalError, ExternalResult, Function, IntoLua, IntoLuaMulti, Lua, Table, Value};
use tokio::fs;
use yazi_fs::{mounts::PARTITIONS, remove_dir_clean};

use crate::{Composer, Error, bindings::Cha, file::File, url::{Url, UrlRef}};

pub fn compose(lua: &Lua) -> mlua::Result<Value> {
	Composer::make(lua, 10, |lua, key| {
		match key {
			b"op" => op(lua)?,
			b"cwd" => cwd(lua)?,
			b"cha" => cha(lua)?,
			b"write" => write(lua)?,
			b"create" => create(lua)?,
			b"remove" => remove(lua)?,
			b"read_dir" => read_dir(lua)?,
			b"unique_name" => unique_name(lua)?,
			b"partitions" => partitions(lua)?,
			_ => return Ok(Value::Nil),
		}
		.into_lua(lua)
	})
}

fn op(lua: &Lua) -> mlua::Result<Function> {
	lua.create_function(|lua, (name, t): (mlua::String, Table)| match name.as_bytes().as_ref() {
		b"part" => super::FilesOp::part(lua, t),
		b"done" => super::FilesOp::done(lua, t),
		_ => Err("Unknown operation".into_lua_err())?,
	})
}

fn cwd(lua: &Lua) -> mlua::Result<Function> {
	lua.create_function(|lua, ()| match std::env::current_dir() {
		Ok(p) => (Url::from(p), Value::Nil).into_lua_multi(lua),
		Err(e) => (Value::Nil, Error::Io(e)).into_lua_multi(lua),
	})
}

fn cha(lua: &Lua) -> mlua::Result<Function> {
	lua.create_async_function(|lua, (url, follow): (UrlRef, Option<bool>)| async move {
		let meta = if follow.unwrap_or(false) {
			fs::metadata(&*url).await
		} else {
			fs::symlink_metadata(&*url).await
		};

		match meta {
			Ok(m) => (Cha::from(m), Value::Nil).into_lua_multi(&lua),
			Err(e) => (Value::Nil, Error::Io(e)).into_lua_multi(&lua),
		}
	})
}

fn write(lua: &Lua) -> mlua::Result<Function> {
	lua.create_async_function(|lua, (url, data): (UrlRef, mlua::String)| async move {
		match fs::write(&*url, data.as_bytes()).await {
			Ok(()) => (true, Value::Nil).into_lua_multi(&lua),
			Err(e) => (false, Error::Io(e)).into_lua_multi(&lua),
		}
	})
}

fn create(lua: &Lua) -> mlua::Result<Function> {
	lua.create_async_function(|lua, (type_, url): (mlua::String, UrlRef)| async move {
		let result = match type_.as_bytes().as_ref() {
			b"dir" => fs::create_dir(&*url).await,
			b"dir_all" => fs::create_dir_all(&*url).await,
			_ => Err("Creation type must be 'dir' or 'dir_all'".into_lua_err())?,
		};

		match result {
			Ok(()) => (true, Value::Nil).into_lua_multi(&lua),
			Err(e) => (false, Error::Io(e)).into_lua_multi(&lua),
		}
	})
}

fn remove(lua: &Lua) -> mlua::Result<Function> {
	lua.create_async_function(|lua, (type_, url): (mlua::String, UrlRef)| async move {
		let result = match type_.as_bytes().as_ref() {
			b"file" => fs::remove_file(&*url).await,
			b"dir" => fs::remove_dir(&*url).await,
			b"dir_all" => fs::remove_dir_all(&*url).await,
			b"dir_clean" => Ok(remove_dir_clean(&url).await),
			_ => Err("Removal type must be 'file', 'dir', 'dir_all', or 'dir_clean'".into_lua_err())?,
		};

		match result {
			Ok(()) => (true, Value::Nil).into_lua_multi(&lua),
			Err(e) => (false, Error::Io(e)).into_lua_multi(&lua),
		}
	})
}

fn read_dir(lua: &Lua) -> mlua::Result<Function> {
	lua.create_async_function(|lua, (dir, options): (UrlRef, Table)| async move {
		let glob = if let Ok(s) = options.raw_get::<mlua::String>("glob") {
			Some(
				GlobBuilder::new(&s.to_str()?)
					.case_insensitive(true)
					.literal_separator(true)
					.backslash_escape(false)
					.empty_alternates(true)
					.build()
					.into_lua_err()?
					.compile_matcher(),
			)
		} else {
			None
		};

		let limit = options.raw_get("limit").unwrap_or(usize::MAX);
		let resolve = options.raw_get("resolve").unwrap_or(false);

		let mut it = match fs::read_dir(&*dir).await {
			Ok(it) => it,
			Err(e) => return (Value::Nil, Error::Io(e)).into_lua_multi(&lua),
		};

		let mut files = vec![];
		while let Ok(Some(next)) = it.next_entry().await {
			if files.len() >= limit {
				break;
			}

			let path = next.path();
			if glob.as_ref().is_some_and(|g| !g.is_match(&path)) {
				continue;
			}

			let url = yazi_shared::url::Url::from(path);
			let file = if !resolve {
				yazi_fs::File::from_dummy(url, next.file_type().await.ok())
			} else if let Ok(meta) = next.metadata().await {
				yazi_fs::File::from_meta(url, meta).await
			} else {
				yazi_fs::File::from_dummy(url, next.file_type().await.ok())
			};
			files.push(File(file));
		}

		let tbl = lua.create_table_with_capacity(files.len(), 0)?;
		for f in files {
			tbl.raw_push(f)?;
		}

		(tbl, Value::Nil).into_lua_multi(&lua)
	})
}

fn unique_name(lua: &Lua) -> mlua::Result<Function> {
	lua.create_async_function(|lua, url: UrlRef| async move {
		match yazi_fs::unique_name(url.clone(), async { false }).await {
			Ok(u) => (Url(u), Value::Nil).into_lua_multi(&lua),
			Err(e) => (Value::Nil, Error::Io(e)).into_lua_multi(&lua),
		}
	})
}

fn partitions(lua: &Lua) -> mlua::Result<Function> {
	lua.create_async_function(|lua, ()| async move {
		PARTITIONS
			.read()
			.iter()
			.filter(|&p| !p.systemic())
			.map(|p| {
				lua.create_table_from([
					("src", p.src.clone().into_lua(&lua)?),
					("dist", p.dist.clone().into_lua(&lua)?),
					("label", p.label.clone().into_lua(&lua)?),
					("fstype", p.fstype.clone().into_lua(&lua)?),
					("external", p.external.into_lua(&lua)?),
					("removable", p.removable.into_lua(&lua)?),
				])
			})
			.collect::<mlua::Result<Vec<Table>>>()
	})
}
