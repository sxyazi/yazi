use std::fmt::Display;

use mlua::{ExternalError, Table, TableExt};
use tracing::warn;
use yazi_plugin::{LOADED, LUA};
use yazi_shared::{emit, event::Cmd, Layer};

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
				emit!(Call(Cmd::args("plugin_do", vec![opt.name]).with_data(opt.data), Layer::App));
			}
		});
	}

	pub(crate) fn plugin_do(&mut self, opt: impl TryInto<yazi_plugin::Opt, Error = impl Display>) {
		let opt = match opt.try_into() {
			Ok(opt) => opt as yazi_plugin::Opt,
			Err(e) => return warn!("{e}"),
		};

		_ = Lives::scope(&self.cx, |_| {
			if let Some(init) = opt.data.init {
				init(&LUA)?;
			}
			LUA.globals().set("YAZI_PLUGIN_NAME", LUA.create_string(&opt.name)?)?;

			let mut plugin: Option<Table> = None;
			if let Some(b) = LOADED.read().get(&opt.name) {
				plugin = LUA.load(b).call(())?;
			}

			let Some(plugin) = plugin else {
				return Err("plugin not found".into_lua_err());
			};

			if let Some(cb) = opt.data.cb {
				cb(&LUA, plugin)
			} else {
				plugin.call_method("entry", opt.data.args)
			}
		});
	}
}
