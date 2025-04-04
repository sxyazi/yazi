use mlua::{IntoLua, ObjectLike};
use yazi_config::LAYOUT;
use yazi_proxy::{AppProxy, options::{PluginCallback, PluginOpt}};
use yazi_shared::event::Cmd;

use crate::{elements::Rect, file::File};

pub fn seek_sync(cmd: &'static Cmd, file: yazi_fs::File, units: i16) {
	let cb: PluginCallback = Box::new(move |lua, plugin| {
		let job = lua.create_table_from([
			("file", File::new(file).into_lua(lua)?),
			("area", Rect::from(LAYOUT.get().preview).into_lua(lua)?),
			("units", units.into_lua(lua)?),
		])?;

		plugin.call_method("seek", job)
	});

	AppProxy::plugin(PluginOpt::new_callback(&cmd.name, cb));
}
