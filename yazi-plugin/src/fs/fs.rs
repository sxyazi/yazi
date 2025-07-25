use std::borrow::Cow;

use globset::GlobBuilder;
use mlua::{ExternalError, ExternalResult, Function, IntoLua, IntoLuaMulti, Lua, Table, Value};
use yazi_binding::{Cha, Composer, ComposerGet, ComposerSet, Error, File, Url, UrlRef};
use yazi_fs::{mounts::PARTITIONS, remove_dir_clean, services};

use crate::bindings::SizeCalculator;

pub fn compose() -> Composer<ComposerGet, ComposerSet> {
	fn get(lua: &Lua, key: &[u8]) -> mlua::Result<Value> {
		match key {
			b"op" => op(lua)?,
			b"cwd" => cwd(lua)?,
			b"cha" => cha(lua)?,
			b"write" => write(lua)?,
			b"create" => create(lua)?,
			b"remove" => remove(lua)?,
			b"read_dir" => read_dir(lua)?,
			b"calc_size" => calc_size(lua)?,
			b"expand_url" => expand_url(lua)?,
			b"unique_name" => unique_name(lua)?,
			b"partitions" => partitions(lua)?,
			_ => return Ok(Value::Nil),
		}
		.into_lua(lua)
	}

	fn set(_: &Lua, _: &[u8], value: Value) -> mlua::Result<Value> { Ok(value) }

	Composer::new(get, set)
}

fn op(lua: &Lua) -> mlua::Result<Function> {
	lua.create_function(|lua, (name, t): (mlua::String, Table)| match name.as_bytes().as_ref() {
		b"part" => super::FilesOp::part(lua, t),
		b"done" => super::FilesOp::done(lua, t),
		b"size" => super::FilesOp::size(lua, t),
		_ => Err("Unknown operation".into_lua_err())?,
	})
}

fn cwd(lua: &Lua) -> mlua::Result<Function> {
	lua.create_function(|lua, ()| match std::env::current_dir() {
		Ok(p) => Url::new(p).into_lua_multi(lua),
		Err(e) => (Value::Nil, Error::Io(e)).into_lua_multi(lua),
	})
}

fn cha(lua: &Lua) -> mlua::Result<Function> {
	lua.create_async_function(|lua, (url, follow): (UrlRef, Option<bool>)| async move {
		let meta = if follow.unwrap_or(false) {
			services::metadata(&*url).await
		} else {
			services::symlink_metadata(&*url).await
		};

		match meta {
			Ok(m) => Cha(yazi_fs::cha::Cha::new(&url, m)).into_lua_multi(&lua),
			Err(e) => (Value::Nil, Error::Io(e)).into_lua_multi(&lua),
		}
	})
}

fn write(lua: &Lua) -> mlua::Result<Function> {
	lua.create_async_function(|lua, (url, data): (UrlRef, mlua::String)| async move {
		match services::write(&*url, data.as_bytes()).await {
			Ok(()) => true.into_lua_multi(&lua),
			Err(e) => (false, Error::Io(e)).into_lua_multi(&lua),
		}
	})
}

fn create(lua: &Lua) -> mlua::Result<Function> {
	lua.create_async_function(|lua, (r#type, url): (mlua::String, UrlRef)| async move {
		let result = match r#type.as_bytes().as_ref() {
			b"dir" => services::create_dir(&*url).await,
			b"dir_all" => services::create_dir_all(&*url).await,
			_ => Err("Creation type must be 'dir' or 'dir_all'".into_lua_err())?,
		};

		match result {
			Ok(()) => true.into_lua_multi(&lua),
			Err(e) => (false, Error::Io(e)).into_lua_multi(&lua),
		}
	})
}

fn remove(lua: &Lua) -> mlua::Result<Function> {
	lua.create_async_function(|lua, (r#type, url): (mlua::String, UrlRef)| async move {
		let result = match r#type.as_bytes().as_ref() {
			b"file" => services::remove_file(&*url).await,
			b"dir" => services::remove_dir(&*url).await,
			b"dir_all" => services::remove_dir_all(&*url).await,
			b"dir_clean" => Ok(remove_dir_clean(&url).await),
			_ => Err("Removal type must be 'file', 'dir', 'dir_all', or 'dir_clean'".into_lua_err())?,
		};

		match result {
			Ok(()) => true.into_lua_multi(&lua),
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

		let mut it = match services::read_dir(&*dir).await {
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
				yazi_fs::File::from_follow(url, meta).await
			} else {
				yazi_fs::File::from_dummy(url, next.file_type().await.ok())
			};
			files.push(File::new(file));
		}

		let tbl = lua.create_table_with_capacity(files.len(), 0)?;
		for f in files {
			tbl.raw_push(f)?;
		}

		tbl.into_lua_multi(&lua)
	})
}

fn calc_size(lua: &Lua) -> mlua::Result<Function> {
	lua.create_async_function(|lua, url: UrlRef| async move {
		match yazi_fs::SizeCalculator::new(&url).await {
			Ok(it) => SizeCalculator(it).into_lua_multi(&lua),
			Err(e) => (Value::Nil, Error::Io(e)).into_lua_multi(&lua),
		}
	})
}

fn expand_url(lua: &Lua) -> mlua::Result<Function> {
	lua.create_function(|lua, value: Value| {
		use yazi_fs::expand_url;
		match &value {
			Value::String(s) => Url::new(expand_url(Url::try_from(s.as_bytes().as_ref())?)).into_lua(lua),
			Value::UserData(ud) => match expand_url(&*ud.borrow::<yazi_binding::Url>()?) {
				Cow::Borrowed(_) => Ok(value),
				Cow::Owned(u) => Url::new(u).into_lua(lua),
			},
			_ => Err("must be a string or a Url".into_lua_err()),
		}
	})
}

fn unique_name(lua: &Lua) -> mlua::Result<Function> {
	lua.create_async_function(|lua, url: UrlRef| async move {
		match yazi_fs::unique_name(url.clone(), async { false }).await {
			Ok(u) => Url::new(u).into_lua_multi(&lua),
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
