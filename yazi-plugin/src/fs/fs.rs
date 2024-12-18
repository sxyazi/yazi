use globset::GlobBuilder;
use mlua::{
	ExternalError, ExternalResult, Function, IntoLua, IntoLuaMulti, Lua, MetaMethod, Table, Value,
};
use tokio::fs;
use yazi_fs::remove_dir_clean;

use crate::{
	Error,
	bindings::{Cast, Cha},
	file::File,
	url::{Url, UrlRef},
};

const READ: u8 = 1;
const WRITE: u8 = 2;
const APPEND: u8 = 4;
const TRUNCATE: u8 = 8;
const CREATE: u8 = 16;
const CREATE_NEW: u8 = 32;

pub fn compose(lua: &Lua) -> mlua::Result<Table> {
	let index = lua.create_function(|lua, (ts, key): (Table, mlua::String)| {
		let value = match key.as_bytes().as_ref() {
			b"cwd" => cwd(lua)?,
			b"cha" => cha(lua)?,
			b"open" => open(lua)?,
			b"write" => write(lua)?,
			b"remove" => remove(lua)?,
			b"create_dir" => create_dir(lua)?,
			b"read_dir" => read_dir(lua)?,
			b"unique_name" => unique_name(lua)?,
			_ => return Ok(Value::Nil),
		}
		.into_lua(lua)?;

		ts.raw_set(key, value.clone())?;
		Ok(value)
	})?;

	let open_options = lua.create_table_from([
		("READ", READ),
		("WRITE", WRITE),
		("APPEND", APPEND),
		("TRUNCATE", TRUNCATE),
		("CREATE", CREATE),
		("CREATE_NEW", CREATE_NEW),
	])?;

	lua.globals().raw_set("OpenOpts", open_options)?;

	let fs = lua.create_table_with_capacity(0, 10)?;
	fs.set_metatable(Some(lua.create_table_from([(MetaMethod::Index.name(), index)])?));

	Ok(fs)
}

fn cwd(lua: &Lua) -> mlua::Result<Function> {
	lua.create_function(|lua, ()| match std::env::current_dir() {
		Ok(p) => (Url::cast(lua, p)?, Value::Nil).into_lua_multi(lua),
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

fn open(lua: &Lua) -> mlua::Result<Function> {
	lua.create_async_function(|lua, (url, opts): (UrlRef, Option<mlua::Integer>)| async move {
		let given_options = opts.unwrap_or(READ as i64);
		let given_options = given_options as u8;
		let mut write_flag = false;
		let mut requires_write = false;
		let mut open_opts = fs::OpenOptions::new();

		if given_options & READ != 0 {
			open_opts.read(true);
		}
		if given_options & WRITE != 0 {
			write_flag = true;
			open_opts.write(true);
		}
		if given_options & APPEND != 0 {
			open_opts.append(true);
		}
		if given_options & TRUNCATE != 0 {
			requires_write = true;
			open_opts.truncate(true);
		}
		if given_options & CREATE != 0 {
			requires_write = true;
			open_opts.create(true);
		}
		if given_options & CREATE_NEW != 0 {
			requires_write = true;
			open_opts.create_new(true);
		}

		if requires_write && !write_flag {
			open_opts.write(true);
		}

		let result = open_opts.open(&*url).await;
		let yazi_file = yazi_fs::File::from(url.clone()).await;

		match (result, yazi_file) {
			(Ok(_), Ok(f)) => match File::cast(&lua, f) {
				Ok(file) => (file, Value::Nil).into_lua_multi(&lua),
				Err(e) => (Value::Nil, e).into_lua_multi(&lua)
			}
			(Err(e), _) => (Value::Nil, Error::Io(e)).into_lua_multi(&lua),
			(_, Err(e)) => (Value::Nil, e).into_lua_multi(&lua)
		}
	})
}

fn write(lua: &Lua) -> mlua::Result<Function> {
	lua.create_async_function(|lua, (url, data): (UrlRef, mlua::String)| async move {
		match fs::write(&*url, data.as_bytes()).await {
			Ok(_) => (true, Value::Nil).into_lua_multi(&lua),
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
			Ok(_) => (true, Value::Nil).into_lua_multi(&lua),
			Err(e) => (false, Error::Io(e)).into_lua_multi(&lua),
		}
	})
}

fn create_dir(lua: &Lua) -> mlua::Result<Function> {
	lua.create_async_function(|lua, (dir, recursive): (UrlRef, Option<bool>)| async move {
		let is_recursive = recursive.unwrap_or(false);
		let result =
			if is_recursive { fs::create_dir_all(&*dir).await } else { fs::create_dir(&*dir).await };
		match result {
			Ok(_) => (true, Value::Nil).into_lua_multi(&lua),
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
			files.push(File::cast(&lua, file)?);
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
			Ok(u) => (Url::cast(&lua, u)?, Value::Nil).into_lua_multi(&lua),
			Err(e) => (Value::Nil, Error::Io(e)).into_lua_multi(&lua),
		}
	})
}
