use std::fmt::Display;

use mlua::TableExt;
use scopeguard::defer;
use tracing::warn;
use yazi_plugin::{loader::LOADER, OptData, RtRef, LUA};
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

		if LOADER.read().contains_key(&opt.name) {
			return self.plugin_do(opt);
		}

		tokio::spawn(async move {
			if LOADER.ensure(&opt.name).await.is_ok() {
				Self::_plugin_do(opt.name, opt.data);
			}
		});
	}

	#[inline]
	pub(crate) fn _plugin_do(name: String, data: OptData) {
		emit!(Call(Cmd::args("plugin_do", vec![name]).with_data(data), Layer::App));
	}

	pub(crate) fn plugin_do(&mut self, opt: impl TryInto<yazi_plugin::Opt, Error = impl Display>) {
		let opt = match opt.try_into() {
			Ok(opt) => opt as yazi_plugin::Opt,
			Err(e) => return warn!("{e}"),
		};

		match LUA.named_registry_value::<RtRef>("rt") {
			Ok(mut r) => r.swap(&opt.name),
			Err(e) => return warn!("{e}"),
		}

		defer! { LUA.named_registry_value::<RtRef>("rt").map(|mut r| r.reset()).ok(); };
		let plugin = match LOADER.load(&opt.name) {
			Ok(plugin) => plugin,
			Err(e) => return warn!("{e}"),
		};

		_ = Lives::scope(&self.cx, |_| {
			if let Some(cb) = opt.data.cb {
				cb(&LUA, plugin)
			} else {
				plugin.call_method("entry", opt.data.args)
			}
		});
	}
}
