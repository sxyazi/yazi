use anyhow::bail;
use mlua::{Table, Value};
use tokio::sync::oneshot;
use yazi_shared::event::Exec;

use crate::ValueSendable;

pub struct Opt {
	pub name: String,
	pub sync: bool,
	pub data: OptData,
}

#[derive(Default)]
pub struct OptData {
	pub args: Vec<ValueSendable>,
	pub cb:   Option<Box<dyn FnOnce(Table) -> mlua::Result<Value> + Send>>,
	pub tx:   Option<oneshot::Sender<ValueSendable>>,
}

impl TryFrom<Exec> for Opt {
	type Error = anyhow::Error;

	fn try_from(mut e: Exec) -> Result<Self, Self::Error> {
		let Some(name) = e.take_first().filter(|s| !s.is_empty()) else {
			bail!("invalid plugin name");
		};

		let mut data: OptData = e.take_data().unwrap_or_default();

		if let Some(args) = e.named.get("args") {
			data.args = shell_words::split(args)?
				.into_iter()
				.map(|s| ValueSendable::String(s.into_bytes()))
				.collect();
		}

		Ok(Self { name, sync: e.named.contains_key("sync"), data })
	}
}
