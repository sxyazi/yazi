use mlua::TableExt;
use yazi_config::LAYOUT;
use yazi_shared::{Layer, emit, event::Cmd};

use crate::{LUA, Opt, OptCallback, bindings::Cast, elements::Rect, file::File};

pub fn seek_sync(cmd: &Cmd, file: yazi_shared::fs::File, units: i16) {
	let cb: OptCallback = Box::new(move |_, plugin| {
		plugin.raw_set("file", File::cast(&LUA, file)?)?;
		plugin.raw_set("area", Rect::from(LAYOUT.load().preview))?;
		plugin.call_method("seek", units)
	});

	let cmd: Cmd =
		Opt { id: cmd.name.to_owned(), sync: true, cb: Some(cb), ..Default::default() }.into();

	emit!(Call(cmd.with_name("plugin"), Layer::App));
}
