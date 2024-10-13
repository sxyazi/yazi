use std::collections::{HashMap, HashSet};

use globset::GlobBuilder;
use mlua::{ExternalError, ExternalResult, IntoLuaMulti, Lua, Table, Value};
use tokio::fs;
use yazi_shared::fs::{maybe_exists, ok_or_not_found, realname, remove_dir_clean, FilesOp, UrnBuf};

use crate::{
	bindings::Cast,
	cha::Cha,
	file::File,
	url::{Url, UrlRef},
};

pub fn install(lua: &Lua) -> mlua::Result<()> {
	lua.globals().raw_set(
		"fs",
		lua.create_table_from([
			(
				"cha",
				lua.create_async_function(|lua, (url, follow): (UrlRef, Option<bool>)| async move {
					let meta = if follow.unwrap_or(false) {
						fs::metadata(&*url).await
					} else {
						fs::symlink_metadata(&*url).await
					};

					match meta {
						Ok(m) => (Cha::cast(lua, m)?, Value::Nil).into_lua_multi(lua),
						Err(e) => (Value::Nil, e.raw_os_error()).into_lua_multi(lua),
					}
				})?,
			),
			(
				"write",
				lua.create_async_function(|lua, (url, data): (UrlRef, mlua::String)| async move {
					match fs::write(&*url, data).await {
						Ok(_) => (true, Value::Nil).into_lua_multi(lua),
						Err(e) => (false, e.raw_os_error()).into_lua_multi(lua),
					}
				})?,
			),
			(
				"create",
				lua.create_async_function(
					|lua, (url, is_dir, overwrite): (UrlRef, Option<bool>, Option<bool>)| async move {
						let Some(parent_dir) = url.parent_url() else {
							let error_msg = format!("{} has no parent directory", url.to_string());
							return (false, error_msg.into_lua_err()).into_lua_multi(lua);
						};

						if !overwrite.unwrap_or(false) && maybe_exists(&*url).await {
							let error_msg = format!("{} already exists", url.to_string());
							return (false, error_msg.into_lua_err()).into_lua_multi(lua);
						}

						let outcome = if is_dir.unwrap_or(false) {
							fs::create_dir_all(&*url).await
						} else if let Some(real) = realname(&url).await {
							ok_or_not_found(fs::remove_file(&*url).await)?;
							FilesOp::Deleting(parent_dir.clone(), HashSet::from_iter([UrnBuf::from(real)]))
								.emit();
							match fs::File::create(&*url).await {
								Ok(_) => Ok(()),
								Err(e) => Err(e),
							}
						} else {
							fs::create_dir_all(&parent_dir).await.ok();
							ok_or_not_found(fs::remove_file(&*url).await)?;
							match fs::File::create(&*url).await {
								Ok(_) => Ok(()),
								Err(e) => Err(e),
							}
						};

						if let Ok(f) = yazi_shared::fs::File::from(url.clone()).await {
							FilesOp::Upserting(parent_dir, HashMap::from_iter([(f.urn_owned(), f)])).emit();
						};

						match outcome {
							Ok(_) => (true, Value::Nil).into_lua_multi(lua),
							Err(e) => (false, e.raw_os_error()).into_lua_multi(lua),
						}
					},
				)?,
			),
			(
				"remove",
				lua.create_async_function(|lua, (type_, url): (mlua::String, UrlRef)| async move {
					let result = match type_.to_str()? {
						"file" => fs::remove_file(&*url).await,
						"dir" => fs::remove_dir(&*url).await,
						"dir_all" => fs::remove_dir_all(&*url).await,
						"dir_clean" => Ok(remove_dir_clean(&url).await),
						_ => {
							Err("Removal type must be 'file', 'dir', 'dir_all', or 'dir_clean'".into_lua_err())?
						}
					};

					match result {
						Ok(_) => (true, Value::Nil).into_lua_multi(lua),
						Err(e) => (false, e.raw_os_error()).into_lua_multi(lua),
					}
				})?,
			),
			(
				"read_dir",
				lua.create_async_function(|lua, (dir, options): (UrlRef, Table)| async move {
					let glob = if let Ok(s) = options.raw_get::<_, mlua::String>("glob") {
						Some(
							GlobBuilder::new(s.to_str()?)
								.case_insensitive(true)
								.literal_separator(false)
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
						Err(e) => return (Value::Nil, e.raw_os_error()).into_lua_multi(lua),
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

						let url = yazi_shared::fs::Url::from(path);
						let file = if !resolve {
							yazi_shared::fs::File::from_dummy(url, next.file_type().await.ok())
						} else if let Ok(meta) = next.metadata().await {
							yazi_shared::fs::File::from_meta(url, meta).await
						} else {
							yazi_shared::fs::File::from_dummy(url, next.file_type().await.ok())
						};
						files.push(File::cast(lua, file)?);
					}

					let tbl = lua.create_table_with_capacity(files.len(), 0)?;
					for f in files {
						tbl.raw_push(f)?;
					}

					(tbl, Value::Nil).into_lua_multi(lua)
				})?,
			),
			(
				"unique_name",
				lua.create_async_function(|lua, url: UrlRef| async move {
					match yazi_shared::fs::unique_name(url.clone()).await {
						Ok(u) => (Url::cast(lua, u)?, Value::Nil).into_lua_multi(lua),
						Err(e) => (Value::Nil, e.raw_os_error()).into_lua_multi(lua),
					}
				})?,
			),
		])?,
	)
}
