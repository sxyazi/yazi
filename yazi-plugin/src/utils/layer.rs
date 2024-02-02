use std::str::FromStr;

use mlua::{ExternalError, ExternalResult, Lua, Table, Value};
use tokio::sync::mpsc;
use yazi_config::keymap::{Control, Key};
use yazi_shared::{emit, event::Cmd, Layer};

use super::Utils;

impl Utils {
	fn parse_keys(value: Value) -> mlua::Result<Vec<Key>> {
		Ok(match value {
			Value::String(s) => {
				vec![Key::from_str(s.to_str()?).into_lua_err()?]
			}
			Value::Table(t) => {
				let mut v = Vec::with_capacity(10);
				for s in t.sequence_values::<mlua::String>() {
					v.push(Key::from_str(s?.to_str()?).into_lua_err()?);
				}
				v
			}
			_ => Err("invalid `on`".into_lua_err())?,
		})
	}

	pub(super) fn layer(lua: &Lua, ya: &Table) -> mlua::Result<()> {
		ya.set(
			"which",
			lua.create_async_function(|_, t: Table| async move {
				let (tx, mut rx) = mpsc::channel::<usize>(1);

				let mut cands = Vec::with_capacity(30);
				for (i, cand) in t.get::<_, Table>("cands")?.sequence_values::<Table>().enumerate() {
					let cand = cand?;
					cands.push(Control {
						on:   Self::parse_keys(cand.get("on")?)?,
						exec: vec![Cmd::args("callback", vec![i.to_string()]).with_data(tx.clone())],
						desc: cand.get("desc").ok(),
					});
				}

				drop(tx);
				emit!(Call(
					Cmd::new("show")
						.with("layer", Layer::Which)
						.with_bool("silent", t.get("silent").unwrap_or_default())
						.with_data(cands),
					Layer::Which
				));

				Ok(rx.recv().await.map(|idx| idx + 1))
			})?,
		)?;

		Ok(())
	}
}
