use std::str::FromStr;

use mlua::{ExternalError, Function, IntoLua, IntoLuaMulti, Lua, LuaString, Table, Value};
use yazi_binding::{Composer, ComposerGet, ComposerSet, Error};
use yazi_config::Pattern;
use yazi_fs::{engine::{Attrs, DirReader, FileHolder}, file::File, mounts::PARTITIONS};
use yazi_shared::url::{UrlBuf, UrlCow, UrlLike, UrlRef};
use yazi_vfs::{VfsFile, engine};

use crate::fs::SizeCalculator;

pub fn compose() -> Composer<ComposerGet, ComposerSet> {
	fn get(lua: &Lua, key: &[u8]) -> mlua::Result<Value> {
		match key {
			b"access" => access(lua)?,
			b"calc_size" => calc_size(lua)?,
			b"cha" => cha(lua)?,
			b"copy" => copy(lua)?,
			b"create" => create(lua)?,
			b"cwd" => cwd(lua)?,
			b"expand_url" => expand_url(lua)?,
			b"file" => file(lua)?,
			b"op" => op(lua)?,
			b"partitions" => partitions(lua)?,
			b"read_dir" => read_dir(lua)?,
			b"remove" => remove(lua)?,
			b"rename" => rename(lua)?,
			b"unique" => unique(lua)?,
			b"write" => write(lua)?,
			_ => return Ok(Value::Nil),
		}
		.into_lua(lua)
	}

	fn set(_: &Lua, _: &[u8], value: Value) -> mlua::Result<Value> { Ok(value) }

	Composer::new(get, set)
}

fn access(lua: &Lua) -> mlua::Result<Function> {
	lua.create_function(|_, ()| Ok(yazi_vfs::engine::Demand::default()))
}

fn calc_size(lua: &Lua) -> mlua::Result<Function> {
	lua.create_async_function(|lua, url: UrlRef| async move {
		let it = if let Some(path) = url.as_local() {
			yazi_fs::engine::local::SizeCalculator::new(path).await.map(SizeCalculator::Local)
		} else {
			yazi_vfs::engine::SizeCalculator::new(&*url).await.map(SizeCalculator::Remote)
		};

		match it {
			Ok(it) => it.into_lua_multi(&lua),
			Err(e) => (Value::Nil, Error::Io(e)).into_lua_multi(&lua),
		}
	})
}

fn cha(lua: &Lua) -> mlua::Result<Function> {
	lua.create_async_function(|lua, (url, follow): (UrlRef, Option<bool>)| async move {
		let cha = if follow.unwrap_or(false) {
			engine::metadata(&*url).await
		} else {
			engine::symlink_metadata(&*url).await
		};

		match cha {
			Ok(c) => c.into_lua_multi(&lua),
			Err(e) => (Value::Nil, Error::Io(e)).into_lua_multi(&lua),
		}
	})
}

fn copy(lua: &Lua) -> mlua::Result<Function> {
	lua.create_async_function(|lua, (from, to): (UrlRef, UrlRef)| async move {
		match engine::copy(&*from, &*to, Attrs::default()).await {
			Ok(len) => len.into_lua_multi(&lua),
			Err(e) => (Value::Nil, Error::Io(e)).into_lua_multi(&lua),
		}
	})
}

fn create(lua: &Lua) -> mlua::Result<Function> {
	lua.create_async_function(|lua, (r#type, url): (LuaString, UrlRef)| async move {
		let result = match &*r#type.as_bytes() {
			b"dir" => engine::create_dir(&*url).await,
			b"dir_all" => engine::create_dir_all(&*url).await,
			_ => Err("Creation type must be 'dir' or 'dir_all'".into_lua_err())?,
		};

		match result {
			Ok(()) => true.into_lua_multi(&lua),
			Err(e) => (false, Error::Io(e)).into_lua_multi(&lua),
		}
	})
}

fn cwd(lua: &Lua) -> mlua::Result<Function> {
	lua.create_function(|lua, ()| match std::env::current_dir() {
		Ok(p) => UrlBuf::from(p).into_lua_multi(lua),
		Err(e) => (Value::Nil, Error::Io(e)).into_lua_multi(lua),
	})
}

#[allow(irrefutable_let_patterns)]
fn expand_url(lua: &Lua) -> mlua::Result<Function> {
	lua.create_function(|lua, value: Value| {
		use yazi_fs::path::expand_url;
		match &value {
			Value::String(s) => expand_url(UrlCow::try_from(&*s.as_bytes())?).into_owned().into_lua(lua),
			Value::UserData(ud) => {
				if let u = expand_url(&*ud.borrow::<UrlBuf>()?)
					&& u.is_owned()
				{
					u.into_owned().into_lua(lua)
				} else {
					Ok(value)
				}
			}
			_ => Err("must be a string or a Url".into_lua_err())?,
		}
	})
}

fn file(lua: &Lua) -> mlua::Result<Function> {
	lua.create_async_function(|lua, url: UrlRef| async move {
		match File::new(&*url).await {
			Ok(file) => file.into_lua_multi(&lua),
			Err(e) => (Value::Nil, Error::Io(e)).into_lua_multi(&lua),
		}
	})
}

fn op(lua: &Lua) -> mlua::Result<Function> {
	lua.create_function(|lua, (name, t): (LuaString, Table)| match &*name.as_bytes() {
		b"part" => super::FilesOp::part(lua, t),
		b"done" => super::FilesOp::done(lua, t),
		b"size" => super::FilesOp::size(lua, t),
		_ => Err("Unknown operation".into_lua_err())?,
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

fn read_dir(lua: &Lua) -> mlua::Result<Function> {
	lua.create_async_function(|lua, (dir, options): (UrlRef, Table)| async move {
		let pat = if let Ok(s) = options.raw_get::<LuaString>("glob") {
			Some(Pattern::from_str(&s.to_str()?)?)
		} else {
			None
		};

		let limit = options.raw_get("limit").unwrap_or(usize::MAX);
		let resolve = options.raw_get::<bool>("resolve")?;

		let mut it = match engine::read_dir(&*dir).await {
			Ok(it) => it,
			Err(e) => return (Value::Nil, Error::Io(e)).into_lua_multi(&lua),
		};

		let mut files = vec![];
		while let Ok(Some(next)) = it.next().await {
			let url = next.url();
			if pat.as_ref().is_some_and(|p| !p.match_url(&url, p.is_dir)) {
				continue;
			}

			let file = if !resolve {
				File::from_dummy(url, next.file_type().await.ok())
			} else if let Ok(cha) = next.metadata().await {
				File::from_follow(url, cha).await
			} else {
				File::from_dummy(url, next.file_type().await.ok())
			};

			files.push(file);
			if files.len() == limit {
				break;
			}
		}

		lua.create_sequence_from(files)?.into_lua_multi(&lua)
	})
}

fn remove(lua: &Lua) -> mlua::Result<Function> {
	lua.create_async_function(|lua, (r#type, url): (LuaString, UrlRef)| async move {
		let result = match &*r#type.as_bytes() {
			b"file" => engine::remove_file(&*url).await,
			b"dir" => engine::remove_dir(&*url).await,
			b"dir_all" => engine::remove_dir_all(&*url).await,
			b"dir_clean" => engine::remove_dir_clean(&*url).await,
			_ => Err("Removal type must be 'file', 'dir', 'dir_all', or 'dir_clean'".into_lua_err())?,
		};

		match result {
			Ok(()) => true.into_lua_multi(&lua),
			Err(e) => (false, Error::Io(e)).into_lua_multi(&lua),
		}
	})
}

fn rename(lua: &Lua) -> mlua::Result<Function> {
	lua.create_async_function(|lua, (from, to): (UrlRef, UrlRef)| async move {
		match engine::rename(&*from, &*to).await {
			Ok(()) => true.into_lua_multi(&lua),
			Err(e) => (false, Error::Io(e)).into_lua_multi(&lua),
		}
	})
}

fn unique(lua: &Lua) -> mlua::Result<Function> {
	lua.create_async_function(|lua, (r#type, url): (LuaString, UrlRef)| async move {
		let result = match &*r#type.as_bytes() {
			b"dir" => yazi_vfs::unique_file(url.clone(), true).await,
			b"file" => yazi_vfs::unique_file(url.clone(), false).await,
			_ => Err("Type must be 'dir' or 'file'".into_lua_err())?,
		};

		match result {
			Ok(u) => u.into_lua_multi(&lua),
			Err(e) => (Value::Nil, Error::Io(e)).into_lua_multi(&lua),
		}
	})
}

fn write(lua: &Lua) -> mlua::Result<Function> {
	lua.create_async_function(|lua, (url, data): (UrlRef, LuaString)| async move {
		match engine::write(&*url, data.as_bytes()).await {
			Ok(()) => true.into_lua_multi(&lua),
			Err(e) => (false, Error::Io(e)).into_lua_multi(&lua),
		}
	})
}
