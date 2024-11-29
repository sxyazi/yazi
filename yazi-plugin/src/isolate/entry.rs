use std::time::Duration;

use mlua::{ExternalError, ExternalResult, Lua, MetaMethod, ObjectLike, Table, Value};
use tokio::runtime::Handle;
use yazi_dds::Sendable;
use yazi_proxy::options::PluginOpt;
use yazi_shared::event::Data;

use super::slim_lua;
use crate::{RtRef, loader::LOADER};

pub async fn entry(opt: PluginOpt) -> mlua::Result<()> {
	LOADER.ensure(&opt.id).await.into_lua_err()?;

	tokio::task::spawn_blocking(move || {
		let lua = slim_lua(&opt.id)?;
		let plugin: Table = if let Some(b) = LOADER.read().get(opt.id.as_ref()) {
			lua.load(b.as_bytes()).set_name(opt.id.as_ref()).call(())?
		} else {
			return Err("unloaded plugin".into_lua_err());
		};

		let job = lua.create_table_from([("args", Sendable::args_to_table(&lua, opt.args)?)])?;

		// TODO: remove this
		install_entry_warn(&lua, &job, opt._old_args).ok();

		Handle::current().block_on(plugin.call_async_method("entry", job))
	})
	.await
	.into_lua_err()?
}

#[inline]
fn warn_deprecated(id: Option<&str>) {
	static WARNED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
	if !WARNED.swap(true, std::sync::atomic::Ordering::Relaxed) {
		let id = match id {
			Some(id) => format!("`{id}.yazi` plugin"),
			None => "`init.lua` config".to_owned(),
		};
		let s = "The first parameter of `entry()` has been replaced by the new `job` instead of the previous `args`.

Please use `job.args` instead `args` to access the arguments in your {id}.

See #1966 for details: https://github.com/sxyazi/yazi/pull/1966";
		yazi_proxy::AppProxy::notify(yazi_proxy::options::NotifyOpt {
			title:   "Deprecated API".to_owned(),
			content: s.replace("{id}", &id),
			level:   yazi_proxy::options::NotifyLevel::Warn,
			timeout: Duration::from_secs(20),
		});
	}
}

pub fn install_entry_warn(lua: &Lua, job: &Table, old_args: Vec<Data>) -> mlua::Result<()> {
	let mt = lua.create_table_from([(
		MetaMethod::Index.name(),
		lua.create_function(|lua, (ts, key): (Table, mlua::String)| {
			if key.as_bytes().first().is_some_and(|&b| b.is_ascii_digit()) {
				warn_deprecated(lua.named_registry_value::<RtRef>("rt")?.current());
				lua
					.load(mlua::chunk! {
						return rawget($ts, "__unsafe_args")[tonumber($key)]
					})
					.call::<Value>(())
			} else {
				ts.raw_get::<Value>(key)
			}
		})?,
	)])?;
	job.set_metatable(Some(mt));
	job.raw_set("__unsafe_args", Sendable::data_to_value(lua, Data::List(old_args))?)?;
	Ok(())
}
