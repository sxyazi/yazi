use std::str::FromStr;

use mlua::{ExternalError, Function, IntoLua, IntoLuaMulti, Lua, Table, Value};
use yazi_binding::{Cha, Composer, ComposerGet, ComposerSet, Error, File, Url, UrlRef};
use yazi_config::Pattern;
use yazi_fs::{mounts::PARTITIONS, provider::{Attrs, DirReader, FileHolder}};
use yazi_shared::url::{UrlCow, UrlLike};
use yazi_vfs::{VfsFile, provider};

use crate::bindings::SizeCalculator;

pub fn compose() -> Composer<ComposerGet, ComposerSet> {
	fn get(lua: &Lua, key: &[u8]) -> mlua::Result<Value> {
		match key {
			b"op" => op(lua)?,
			b"cwd" => cwd(lua)?,
			b"cha" => cha(lua)?,
			b"copy" => copy(lua)?,
			b"write" => write(lua)?,
			b"create" => create(lua)?,
			b"remove" => remove(lua)?,
			b"rename" => rename(lua)?,
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
	lua.create_function(|lua, (name, t): (mlua::String, Table)| match &*name.as_bytes() {
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
		let cha = if follow.unwrap_or(false) {
			provider::metadata(&*url).await
		} else {
			provider::symlink_metadata(&*url).await
		};

		match cha {
			Ok(c) => Cha(c).into_lua_multi(&lua),
			Err(e) => (Value::Nil, Error::Io(e)).into_lua_multi(&lua),
		}
	})
}

fn copy(lua: &Lua) -> mlua::Result<Function> {
	lua.create_async_function(|lua, (from, to): (UrlRef, UrlRef)| async move {
		match provider::copy(&*from, &*to, Attrs::default()).await {
			Ok(len) => len.into_lua_multi(&lua),
			Err(e) => (Value::Nil, Error::Io(e)).into_lua_multi(&lua),
		}
	})
}

fn write(lua: &Lua) -> mlua::Result<Function> {
	lua.create_async_function(|lua, (url, data): (UrlRef, mlua::String)| async move {
		match provider::write(&*url, data.as_bytes()).await {
			Ok(()) => true.into_lua_multi(&lua),
			Err(e) => (false, Error::Io(e)).into_lua_multi(&lua),
		}
	})
}

fn create(lua: &Lua) -> mlua::Result<Function> {
	lua.create_async_function(|lua, (r#type, url): (mlua::String, UrlRef)| async move {
		let result = match &*r#type.as_bytes() {
			b"dir" => provider::create_dir(&*url).await,
			b"dir_all" => provider::create_dir_all(&*url).await,
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
		let result = match &*r#type.as_bytes() {
			b"file" => provider::remove_file(&*url).await,
			b"dir" => provider::remove_dir(&*url).await,
			b"dir_all" => provider::remove_dir_all(&*url).await,
			b"dir_clean" => provider::remove_dir_clean(&*url).await,
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
		match provider::rename(&*from, &*to).await {
			Ok(()) => true.into_lua_multi(&lua),
			Err(e) => (false, Error::Io(e)).into_lua_multi(&lua),
		}
	})
}

fn read_dir(lua: &Lua) -> mlua::Result<Function> {
	lua.create_async_function(|lua, (dir, options): (UrlRef, Table)| async move {
		let pat = if let Ok(s) = options.raw_get::<mlua::String>("glob") {
			Some(Pattern::from_str(&s.to_str()?)?)
		} else {
			None
		};

		let limit = options.raw_get("limit").unwrap_or(usize::MAX);
		let resolve = options.raw_get("resolve").unwrap_or(false);

		let mut it = match provider::read_dir(&*dir).await {
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
				yazi_fs::File::from_dummy(url, next.file_type().await.ok())
			} else if let Ok(cha) = next.metadata().await {
				yazi_fs::File::from_follow(url, cha).await
			} else {
				yazi_fs::File::from_dummy(url, next.file_type().await.ok())
			};

			files.push(File::new(file));
			if files.len() == limit {
				break;
			}
		}

		lua.create_sequence_from(files)?.into_lua_multi(&lua)
	})
}

fn calc_size(lua: &Lua) -> mlua::Result<Function> {
	lua.create_async_function(|lua, url: UrlRef| async move {
		let it = if let Some(path) = url.as_local() {
			yazi_fs::provider::local::SizeCalculator::new(path).await.map(SizeCalculator::Local)
		} else {
			yazi_vfs::provider::SizeCalculator::new(&*url).await.map(SizeCalculator::Remote)
		};

		match it {
			Ok(it) => it.into_lua_multi(&lua),
			Err(e) => (Value::Nil, Error::Io(e)).into_lua_multi(&lua),
		}
	})
}

#[allow(irrefutable_let_patterns)]
fn expand_url(lua: &Lua) -> mlua::Result<Function> {
	lua.create_function(|lua, value: Value| {
		use yazi_fs::path::expand_url;
		match &value {
			Value::String(s) => Url::new(expand_url(UrlCow::try_from(&*s.as_bytes())?)).into_lua(lua),
			Value::UserData(ud) => {
				if let u = expand_url(&*ud.borrow::<yazi_binding::Url>()?)
					&& u.is_owned()
				{
					Url::new(u.into_owned()).into_lua(lua)
				} else {
					Ok(value)
				}
			}
			_ => Err("must be a string or a Url".into_lua_err())?,
		}
	})
}

fn unique_name(lua: &Lua) -> mlua::Result<Function> {
	lua.create_async_function(|lua, url: UrlRef| async move {
		match yazi_vfs::unique_name(url.clone(), async { false }).await {
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
