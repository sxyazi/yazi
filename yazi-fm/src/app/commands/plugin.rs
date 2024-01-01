use mlua::{ExternalError, ExternalResult, IntoLua, Table, TableExt, Value, Variadic};
use tracing::error;
use yazi_plugin::{LOADED, LUA};
use yazi_shared::{emit, event::Exec, Layer};

use crate::{app::App, lives::Lives};

impl App {
	pub(crate) fn plugin(&mut self, opt: impl TryInto<yazi_plugin::Opt>) {
		let Ok(opt) = opt.try_into() else {
			return;
		};

		if !opt.sync {
			return self.cx.tasks.plugin_micro(&opt.name);
		}

		if LOADED.read().contains_key(&opt.name) {
			return self.plugin_do(opt);
		}

		tokio::spawn(async move {
			if LOADED.ensure(&opt.name).await.is_ok() {
				emit!(Call(Exec::call("plugin_do", vec![opt.name]).with_data(opt.data).vec(), Layer::App));
			}
		});
	}

	pub(crate) fn plugin_do(&mut self, opt: impl TryInto<yazi_plugin::Opt>) {
		let Ok(opt) = opt.try_into() else {
			return;
		};

		let args = Variadic::from_iter(opt.data.args.into_iter().filter_map(|v| v.into_lua(&LUA).ok()));
		let mut ret: mlua::Result<Value> = Err("uninitialized plugin".into_lua_err());

		Lives::scope(&self.cx, |_| {
			let mut plugin: Option<Table> = None;
			if let Some(b) = LOADED.read().get(&opt.name) {
				match LUA.load(b).call(()) {
					Ok(t) => plugin = Some(t),
					Err(e) => ret = Err(e),
				}
			}
			if let Some(plugin) = plugin {
				ret =
					if let Some(cb) = opt.data.cb { cb(plugin) } else { plugin.call_method("entry", args) };
			}
		});

		if let Err(e) = ret {
			error!("{e}");
			return;
		}

		let Some(tx) = opt.data.tx else {
			return;
		};

		if let Ok(v) = ret.and_then(|v| v.try_into().into_lua_err()) {
			tx.send(v).ok();
		}
	}
}
