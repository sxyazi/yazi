use std::str::FromStr;

use mlua::{ExternalError, ExternalResult, IntoLuaMulti, Lua, Table, Value};
use tokio::sync::mpsc;
use yazi_config::{keymap::{Control, Key}, popup::InputCfg};
use yazi_proxy::{AppProxy, InputProxy};
use yazi_shared::{emit, event::Cmd, Layer};

use super::Utils;
use crate::bindings::{InputRx, Position};

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
		ya.raw_set(
			"which",
			lua.create_async_function(|_, t: Table| async move {
				let (tx, mut rx) = mpsc::channel::<usize>(1);

				let mut cands = Vec::with_capacity(30);
				for (i, cand) in t.raw_get::<_, Table>("cands")?.sequence_values::<Table>().enumerate() {
					let cand = cand?;
					cands.push(Control {
						on:   Self::parse_keys(cand.raw_get("on")?)?,
						run:  vec![Cmd::args("callback", vec![i.to_string()]).with_data(tx.clone())],
						desc: cand.raw_get("desc").ok(),
					});
				}

				drop(tx);
				emit!(Call(
					Cmd::new("show")
						.with("layer", Layer::Which)
						.with_bool("silent", t.raw_get("silent").unwrap_or_default())
						.with_data(cands),
					Layer::Which
				));

				Ok(rx.recv().await.map(|idx| idx + 1))
			})?,
		)?;

		ya.raw_set(
			"input",
			lua.create_async_function(|lua, t: Table| async move {
				let realtime = t.raw_get("realtime").unwrap_or_default();
				let mut rx = InputProxy::show(InputCfg {
					title: t.raw_get("title")?,
					value: t.raw_get("value").unwrap_or_default(),
					cursor: None, // TODO
					position: Position::try_from(t.raw_get::<_, Table>("position")?)?.into(),
					realtime,
					completion: false,
					highlight: false,
				});

				if realtime {
					(InputRx::new(rx), Value::Nil).into_lua_multi(lua)
				} else if let Some(res) = rx.recv().await {
					InputRx::parse(res).into_lua_multi(lua)
				} else {
					(Value::Nil, 0).into_lua_multi(lua)
				}
			})?,
		)?;

		ya.raw_set(
			"notify",
			lua.create_function(|_, t: Table| {
				AppProxy::notify(t.try_into()?);
				Ok(())
			})?,
		)?;

		Ok(())
	}
}
