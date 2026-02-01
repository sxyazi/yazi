use mlua::{ExternalError, FromLua, IntoLua, Lua, Table, Value};
use tokio::sync::mpsc;
use yazi_config::{KEYMAP, keymap::{ChordCow, Key}};
use yazi_shared::{Layer, event::CmdCow};

#[derive(Clone, Debug)]
pub struct ActivateOpt {
	pub tx:     Option<mpsc::UnboundedSender<Option<yazi_binding::ChordCow>>>,
	pub cands:  Vec<ChordCow>,
	pub silent: bool,
	pub times:  usize,
}

impl TryFrom<CmdCow> for ActivateOpt {
	type Error = anyhow::Error;

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		if let Some(opt) = c.take_any2("opt") {
			return opt;
		}

		Ok(Self {
			tx:     c.take_any2("tx").transpose()?,
			cands:  c.take_any_iter::<yazi_binding::ChordCow>().map(Into::into).collect(),
			silent: c.bool("silent"),
			times:  c.get("times").unwrap_or(0),
		})
	}
}

impl From<(Layer, Key)> for ActivateOpt {
	fn from((layer, key): (Layer, Key)) -> Self {
		Self {
			tx:     None,
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

impl FromLua for ActivateOpt {
	fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
		let Value::Table(t) = value else {
			return Err("expected a table".into_lua_err());
		};

		Ok(Self {
			tx:     t.raw_get::<yazi_binding::MpscUnboundedTx<_>>("tx").ok().map(|t| t.0),
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

impl IntoLua for ActivateOpt {
	#[rustfmt::skip]
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		lua
			.create_table_from([
				("tx", self.tx.map(yazi_binding::MpscUnboundedTx).into_lua(lua)?),
				("cands", lua.create_sequence_from(self.cands.into_iter().map(yazi_binding::ChordCow))?.into_lua(lua)?),
				("times", self.times.into_lua(lua)?),
				("silent", self.silent.into_lua(lua)?),
			])?
			.into_lua(lua)
	}
}
