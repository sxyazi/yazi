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

impl TryFrom<&Exec> for Opt {
	type Error = anyhow::Error;

	fn try_from(e: &Exec) -> Result<Self, Self::Error> {
		let Some(name) = e.args.first().filter(|s| !s.is_empty()) else {
			bail!("invalid plugin name");
		};

		Ok(Self {
			name: name.to_owned(),
			sync: e.named.contains_key("sync"),
			data: e.take_data().unwrap_or_default(),
		})
	}
}
