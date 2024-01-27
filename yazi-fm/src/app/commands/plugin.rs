use std::fmt::Display;

use mlua::{ExternalError, ExternalResult, IntoLua, Table, TableExt, Variadic};
use tracing::warn;
use yazi_plugin::{LOADED, LUA};
use yazi_shared::{emit, event::Exec, Layer};

use crate::{app::App, lives::Lives};

impl App {
	pub(crate) fn plugin(&mut self, opt: impl TryInto<yazi_plugin::Opt, Error = impl Display>) {
		let opt = match opt.try_into() {
			Ok(opt) => opt as yazi_plugin::Opt,
			Err(e) => return warn!("{e}"),
		};

		if !opt.sync {
			return self.cx.tasks.plugin_micro(opt.name, opt.data.args);
		}

		if LOADED.read().contains_key(&opt.name) {
			return self.plugin_do(opt);
		}

		tokio::spawn(async move {
			if LOADED.ensure(&opt.name).await.is_ok() {
				emit!(Call(Exec::call("plugin_do", vec![opt.name]).with_data(opt.data), Layer::App));
			}
		});
	}

	pub(crate) fn plugin_do(&mut self, opt: impl TryInto<yazi_plugin::Opt, Error = impl Display>) {
		let opt = match opt.try_into() {
			Ok(opt) => opt as yazi_plugin::Opt,
			Err(e) => return warn!("{e}"),
		};

		let args = Variadic::from_iter(opt.data.args.into_iter().filter_map(|v| v.into_lua(&LUA).ok()));
		let result = Lives::scope(&self.cx, |_| {
			LUA.globals().set("YAZI_PLUGIN_NAME", LUA.create_string(&opt.name)?)?;

			let mut plugin: Option<Table> = None;
			if let Some(b) = LOADED.read().get(&opt.name) {
				plugin = LUA.load(b).call(args)?;
			}

			let Some(plugin) = plugin else {
				return Err("plugin not found".into_lua_err());
			};

			if let Some(cb) = opt.data.cb { cb(plugin) } else { plugin.call_method("entry", ()) }
		});

		let Some(tx) = opt.data.tx else {
			return;
		};

		if let Ok(v) = result.and_then(|v| v.try_into().into_lua_err()) {
			tx.send(v).ok();
		}
	}
}
