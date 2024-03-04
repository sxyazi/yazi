use anyhow::bail;
use mlua::{Lua, Table};
use yazi_shared::event::Cmd;

use crate::ValueSendable;

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
		let Some(name) = c.take_first().filter(|s| !s.is_empty()) else {
			bail!("plugin name cannot be empty");
		};

		let mut data: OptData = c.take_data().unwrap_or_default();

		if let Some(args) = c.named.get("args") {
			data.args = shell_words::split(args)?
				.into_iter()
				.map(|s| ValueSendable::String(s.into_bytes()))
				.collect();
		}

		Ok(Self { name, sync: c.named.contains_key("sync"), data })
	}
}
