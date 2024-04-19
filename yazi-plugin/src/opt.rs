use anyhow::bail;
use mlua::{Lua, Table};
use yazi_shared::event::{Cmd, Data};

pub(super) type OptCallback = Box<dyn FnOnce(&Lua, Table) -> mlua::Result<()> + Send>;

#[derive(Default)]
pub struct Opt {
	pub name: String,
	pub sync: bool,
	pub args: Vec<Data>,
	pub cb:   Option<OptCallback>,
}

impl TryFrom<Cmd> for Opt {
	type Error = anyhow::Error;

	fn try_from(mut c: Cmd) -> Result<Self, Self::Error> {
		let Some(name) = c.take_first_str().filter(|s| !s.is_empty()) else {
			bail!("plugin name cannot be empty");
		};

		let args = if let Some(s) = c.get_str("args") {
			shell_words::split(s)?.into_iter().map(Data::String).collect()
		} else {
			c.take_any::<Vec<Data>>("args").unwrap_or_default()
		};

		Ok(Self { name, sync: c.get_bool("sync"), args, cb: c.take_any("callback") })
	}
}

impl From<Opt> for Cmd {
	fn from(value: Opt) -> Self {
		let mut cmd =
			Cmd::args("", vec![value.name]).with_bool("sync", value.sync).with_any("args", value.args);

		if let Some(cb) = value.cb {
			cmd = cmd.with_any("callback", cb);
		}
		cmd
	}
}
