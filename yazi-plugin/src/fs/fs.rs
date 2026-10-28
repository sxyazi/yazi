use std::{borrow::Cow, str::FromStr};

use mlua::{BorrowedBytes, ExternalError, Function, IntoLua, IntoLuaMulti, Lua, LuaString, Table, Value};
use yazi_binding::{Composer, ComposerGet};
use yazi_config::Pattern;
use yazi_fs::{engine::{Attrs, DirReader, FileHolder}, file::{File, Files}, mounts::PARTITIONS};
use yazi_shared::url::{UrlBuf, UrlCow, UrlLike, UrlRef};
use yazi_shim::fs::Error;
use yazi_vfs::engine;

use crate::fs::SizeCalculator;

pub(crate) fn compose() -> Composer<ComposerGet> {
	fn get(lua: &Lua, key: &[u8]) -> mlua::Result<Value> {
		match key {
			b"access" => access(lua)?,
			b"calc_size" => calc_size(lua)?,
			b"cha" => stat(lua)?, // TODO: remove
			b"stat" => stat(lua)?,
			b"clean_url" => clean_url(lua)?,
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
			b"reroute" => reroute(lua)?,
			b"safename" => safename(lua)?,
			b"trash" => return yazi_fs::trash::Trash.into_lua(lua),
			b"unique" => unique(lua)?,
			b"write" => write(lua)?,
			_ => return Ok(Value::Nil),
		}
		.into_lua(lua)
	}

	Composer::new(get)
}

fn access(lua: &Lua) -> mlua::Result<Function> {
	lua.create_function(|_, ()| Ok(yazi_vfs::engine::Demand::default()))
}

fn calc_size(lua: &Lua) -> mlua::Result<Function> {
	lua.create_async_function(|_, url: UrlRef| async move {
		let it = if let Some(path) = url.as_local() {
			yazi_fs::engine::local::SizeCalculator::new(path).await.map(SizeCalculator::Local)
		} else {
			yazi_vfs::engine::SizeCalculator::new(&*url).await.map(SizeCalculator::Virtual)
		};

		Ok(it.map_err(Error::from))
	})
}

fn stat(lua: &Lua) -> mlua::Result<Function> {
	lua.create_async_function(|_, (url, follow): (UrlRef, bool)| async move {
		let stat =
			if follow { engine::metadata(&*url).await } else { engine::symlink_metadata(&*url).await };

		Ok(stat.map_err(Error::from))
	})
}

fn clean_url(lua: &Lua) -> mlua::Result<Function> {
	lua.create_function(|_, url: UrlRef| Ok(yazi_fs::path::clean_url(&*url)))
}

fn copy(lua: &Lua) -> mlua::Result<Function> {
	lua.create_async_function(|_, (from, to): (UrlRef, UrlRef)| async move {
		let result = async { engine::copy(&*from, &*to, Attrs::default()).await?.total().await }.await;
		Ok(result.map_err(Error::from))
	})
}

fn create(lua: &Lua) -> mlua::Result<Function> {
	lua.create_async_function(|lua, (r#type, url): (BorrowedBytes, UrlRef)| async move {
		let result = match &*r#type {
			b"dir" => engine::create_dir(&*url).await,
			b"dir_all" => engine::create_dir_all(&*url).await,
			b"file" => engine::create_file(&*url).await,
			b"file_new" => engine::create_file_new(&*url).await,
			_ => Err("Creation type must be 'dir', 'dir_all', 'file', or 'file_new'".into_lua_err())?,
		};

		match result {
			Ok(()) => true.into_lua_multi(&lua),
			Err(e) => (false, Error::from(e)).into_lua_multi(&lua),
		}
	})
}

fn cwd(lua: &Lua) -> mlua::Result<Function> {
	lua.create_function(|_, ()| Ok(std::env::current_dir().map(UrlBuf::from).map_err(Error::from)))
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
	lua.create_async_function(|_, url: UrlRef| async move {
		Ok(engine::file(&*url).await.map_err(Error::from))
	})
}

fn op(lua: &Lua) -> mlua::Result<Function> {
	lua.create_function(|_, (name, t): (BorrowedBytes, Table)| match &*name {
		b"part" => super::FilesOp::part(t),
		b"done" => super::FilesOp::done(t),
		b"size" => super::FilesOp::size(t),
		b"rank" => super::FilesOp::rank(t),
		b"upsert" => super::FilesOp::upsert(t),
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
					("src", p.src.as_os_str().into_lua(&lua)?),
					("dist", p.dist.as_deref().into_lua(&lua)?),
					("label", p.label.as_deref().into_lua(&lua)?),
					("fstype", p.fstype.as_deref().into_lua(&lua)?),
					("capacity", p.capacity.into_lua(&lua)?),
					("external", p.external.into_lua(&lua)?),
					("removable", p.removable.into_lua(&lua)?),
				])
			})
			.collect::<mlua::Result<Vec<Table>>>()
	})
}

fn read_dir(lua: &Lua) -> mlua::Result<Function> {
	lua.create_async_function(|_, (dir, options): (UrlRef, Table)| async move {
		let pat = if let Ok(s) = options.raw_get::<LuaString>("glob") {
			Some(Pattern::from_str(&s.to_str()?)?)
		} else {
			None
		};

		let limit = options.raw_get("limit").unwrap_or(usize::MAX);
		let resolve = options.raw_get::<bool>("resolve")?;

		let mut it = match engine::read_dir(&*dir).await {
			Ok(it) => it,
			Err(e) => return Ok(Err(Error::from(e))),
		};

		let mut files = Files::default();
		while let Ok(Some(dent)) = it.next().await {
			let url = dent.url();
			if pat.as_ref().is_some_and(|p| !p.match_url(&url, p.is_dir)) {
				continue;
			}

			let file = if !resolve {
				File::from_dummy(url, dent.file_type().await.ok())
			} else if let Ok(file) = dent.file().await {
				file
			} else {
				File::from_dummy(url, dent.file_type().await.ok())
			};

			files.push(file);
			if files.len() == limit {
				break;
			}
		}

		Ok(Ok(files))
	})
}

fn remove(lua: &Lua) -> mlua::Result<Function> {
	lua.create_async_function(|lua, (r#type, url): (BorrowedBytes, UrlRef)| async move {
		let result = match &*r#type {
			b"file" => engine::remove_file(&*url).await,
			b"dir" => engine::remove_dir(&*url).await,
			b"dir_all" => async { engine::remove_dir_all(&*url).await?.finish().await }.await,
			b"dir_clean" => engine::remove_dir_clean(&*url).await,
			_ => Err("Removal type must be 'file', 'dir', 'dir_all', or 'dir_clean'".into_lua_err())?,
		};

		match result {
			Ok(()) => true.into_lua_multi(&lua),
			Err(e) => (false, Error::from(e)).into_lua_multi(&lua),
		}
	})
}

fn rename(lua: &Lua) -> mlua::Result<Function> {
	lua.create_async_function(|lua, (from, to): (UrlRef, UrlRef)| async move {
		match engine::rename(&*from, &*to).await {
			Ok(()) => true.into_lua_multi(&lua),
			Err(e) => (false, Error::from(e)).into_lua_multi(&lua),
		}
	})
}

fn reroute(lua: &Lua) -> mlua::Result<Function> {
	lua.create_async_function(|_, url: UrlRef| async move {
		Ok(engine::reroute(&*url).await.map_err(Error::from))
	})
}

fn safename(lua: &Lua) -> mlua::Result<Function> {
	lua.create_function(|lua, s: LuaString| {
		let b = s.as_bytes();
		match yazi_fs::path::safename(&b) {
			Some(Cow::Borrowed(_)) => Ok(Some(s)),
			Some(Cow::Owned(s)) => lua.create_external_string(s).map(Some),
			None => Ok(None),
		}
	})
}

fn unique(lua: &Lua) -> mlua::Result<Function> {
	lua.create_async_function(|_, (r#type, url): (BorrowedBytes, UrlRef)| async move {
		let result = match &*r#type {
			b"dir" => yazi_vfs::unique_file(url.clone(), true).await,
			b"file" => yazi_vfs::unique_file(url.clone(), false).await,
			_ => Err("Type must be 'dir' or 'file'".into_lua_err())?,
		};

		Ok(result.map_err(Error::from))
	})
}

fn write(lua: &Lua) -> mlua::Result<Function> {
	lua.create_async_function(|lua, (url, data): (UrlRef, BorrowedBytes)| async move {
		match engine::write(&*url, data).await {
			Ok(()) => true.into_lua_multi(&lua),
			Err(e) => (false, Error::from(e)).into_lua_multi(&lua),
		}
	})
}
