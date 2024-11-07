use mlua::TableExt;
use yazi_config::LAYOUT;
use yazi_proxy::{AppProxy, options::{PluginCallback, PluginOpt}};
use yazi_shared::event::Cmd;

use crate::{LUA, bindings::Cast, elements::Rect, file::File};

pub fn seek_sync(cmd: &Cmd, file: yazi_shared::fs::File, units: i16) {
	let cb: PluginCallback = Box::new(move |_, plugin| {
		plugin.raw_set("file", File::cast(&LUA, file)?)?;
		plugin.raw_set("area", Rect::from(LAYOUT.get().preview))?;
		plugin.call_method("seek", units)
	});

	AppProxy::plugin(PluginOpt::new_callback(&cmd.name, cb));
}
