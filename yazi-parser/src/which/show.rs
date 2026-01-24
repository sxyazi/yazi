use anyhow::bail;
use mlua::{ExternalError, FromLua, IntoLua, Lua, Table, Value};
use yazi_config::{KEYMAP, keymap::{ChordCow, Key}};
use yazi_shared::{Layer, event::CmdCow};

#[derive(Debug)]
pub struct ShowOpt {
	pub cands:  Vec<ChordCow>,
	pub silent: bool,
	pub times:  usize,
}

impl TryFrom<CmdCow> for ShowOpt {
	type Error = anyhow::Error;

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		match c.take_any2("opt") {
			Some(opt) => opt,
			None => bail!("missing 'opt' argument"),
		}
	}
}

impl From<(Layer, Key)> for ShowOpt {
	fn from((layer, key): (Layer, Key)) -> Self {
		Self {
			cands:  KEYMAP
				.get(layer)
				.iter()
				.filter(|c| c.on.len() > 1 && c.on[0] == key)
				.map(Into::into)
				.collect(),
			times:  1,
			silent: false,
		}
	}
}

impl FromLua for ShowOpt {
	fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
		let Value::Table(t) = value else {
			return Err("expected a table".into_lua_err());
		};

		Ok(ShowOpt {
			cands:  t
				.raw_get::<Table>("cands")?
				.sequence_values::<yazi_binding::ChordCow>()
				.map(|c| c.map(Into::into))
				.collect::<mlua::Result<Vec<_>>>()?,
			times:  t.raw_get("times").unwrap_or_default(),
			silent: t.raw_get("silent")?,
		})
	}
}

impl IntoLua for ShowOpt {
	#[rustfmt::skip]
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		lua
			.create_table_from([
				("cands", lua.create_sequence_from(self.cands.into_iter().map(yazi_binding::ChordCow))?.into_lua(lua)?),
				("times", self.times.into_lua(lua)?),
				("silent", self.silent.into_lua(lua)?),
			])?
			.into_lua(lua)
	}
}
