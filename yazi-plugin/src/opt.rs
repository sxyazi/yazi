use anyhow::bail;
use mlua::{Lua, Table};
use yazi_dds::ValueSendable;
use yazi_shared::event::Cmd;

pub struct Opt {
	pub name: String,
	pub sync: bool,
	pub data: OptData,
}

#[derive(Default)]
pub struct OptData {
	pub args: Vec<ValueSendable>,
	pub cb:   Option<Box<dyn FnOnce(&Lua, Table) -> mlua::Result<()> + Send>>,
}

impl TryFrom<Cmd> for Opt {
	type Error = anyhow::Error;

	fn try_from(mut c: Cmd) -> Result<Self, Self::Error> {
		let Some(name) = c.take_first_str().filter(|s| !s.is_empty()) else {
			bail!("plugin name cannot be empty");
		};

		let mut data: OptData = c.take_data().unwrap_or_default();

		if let Some(args) = c.get_str("args") {
			data.args = shell_words::split(args)?.into_iter().map(ValueSendable::String).collect();
		}

		Ok(Self { name, sync: c.get_bool("sync"), data })
	}
}
