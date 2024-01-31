use mlua::TableExt;
use yazi_config::LAYOUT;
use yazi_shared::{emit, event::Cmd, Layer};

use crate::{bindings::{Cast, File}, elements::Rect, OptData, LUA};

pub fn seek_sync(cmd: &Cmd, file: yazi_shared::fs::File, units: i16) {
	let data = OptData {
		args: vec![],
		cb:   Some(Box::new(move |plugin| {
			plugin.set("file", File::cast(&LUA, file)?)?;
			plugin.set("area", Rect::cast(&LUA, LAYOUT.load().preview)?)?;
			plugin.call_method("seek", units)
		})),
		tx:   None,
	};
	emit!(Call(
		Cmd::args("plugin", vec![cmd.name.to_owned()]).with_bool("sync", true).with_data(data),
		Layer::App
	));
}
