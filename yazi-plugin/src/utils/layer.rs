use std::{str::FromStr, time::Duration};

use mlua::{ExternalError, ExternalResult, Function, IntoLuaMulti, Lua, Table, Value};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use yazi_binding::{elements::{Line, Pos, Text}, runtime};
use yazi_config::{keymap::{Chord, ChordCow, Key}, popup::{ConfirmCfg, InputCfg}};
use yazi_macro::relay;
use yazi_parser::{app::NotifyOpt, which::ActivateOpt};
use yazi_proxy::{AppProxy, ConfirmProxy, InputProxy, WhichProxy};
use yazi_shared::Debounce;

use super::Utils;
use crate::bindings::InputRx;

impl Utils {
	pub(super) fn which(lua: &Lua) -> mlua::Result<Function> {
		lua.create_async_function(|lua, t: Table| async move {
			if runtime!(lua)?.blocking {
				return Err("Cannot call `ya.which()` while main thread is blocked".into_lua_err());
			}

			let (tx, mut rx) = mpsc::unbounded_channel::<usize>();
			let cands: Vec<_> = t
				.raw_get::<Table>("cands")?
				.sequence_values::<Table>()
				.enumerate()
				.map(|(i, cand)| {
					let cand = cand?;
					Ok(ChordCow::Owned(Chord {
						on:    Self::parse_keys(cand.raw_get("on")?)?,
						run:   vec![relay!(which:callback, [i]).with_any("tx", tx.clone())],
						desc:  cand.raw_get("desc").ok(),
						r#for: None,
					}))
				})
				.collect::<mlua::Result<_>>()?;

			drop(tx);
			WhichProxy::activate(ActivateOpt { cands, times: 0, silent: t.raw_get("silent")? });

			Ok(rx.recv().await.map(|idx| idx + 1))
		})
	}

	pub(super) fn input(lua: &Lua) -> mlua::Result<Function> {
		lua.create_async_function(|lua, t: Table| async move {
			if runtime!(lua)?.blocking {
				return Err("Cannot call `ya.input()` while main thread is blocked".into_lua_err());
			}

			let realtime = t.raw_get("realtime")?;
			let rx = UnboundedReceiverStream::new(InputProxy::show(InputCfg {
				title: t.raw_get("title")?,
				value: t.raw_get("value").unwrap_or_default(),
				cursor: None, // TODO
				obscure: t.raw_get("obscure")?,
				position: Pos::new_input(t.raw_get("pos")?)?.into(),
				realtime,
				completion: false,
			}));

			if !realtime {
				return InputRx::consume(rx).await.into_lua_multi(&lua);
			}

			let debounce = t.raw_get::<f64>("debounce").unwrap_or_default();
			if debounce < 0.0 {
				Err("negative debounce duration".into_lua_err())
			} else if debounce == 0.0 {
				InputRx::new(rx).into_lua_multi(&lua)
			} else {
				InputRx::new(Debounce::new(rx, Duration::from_secs_f64(debounce))).into_lua_multi(&lua)
			}
		})
	}

	pub(super) fn confirm(lua: &Lua) -> mlua::Result<Function> {
		fn body(t: &Table) -> mlua::Result<ratatui::widgets::Paragraph<'static>> {
			Ok(match t.raw_get::<Value>("body")? {
				Value::Nil => Default::default(),
				v => Text::try_from(v)?.into(),
			})
		}

		lua.create_async_function(|lua, t: Table| async move {
			if runtime!(lua)?.blocking {
				return Err("Cannot call `ya.confirm()` while main thread is blocked".into_lua_err());
			}

			let result = ConfirmProxy::show(ConfirmCfg {
				position: Pos::try_from(t.raw_get::<Value>("pos")?)?.into(),
				title:    Line::try_from(t.raw_get::<Value>("title")?)?.into(),
				body:     body(&t)?,
				list:     Default::default(), // TODO
			});

			Ok(result.await)
		})
	}

	pub(super) fn notify(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|_, opt: NotifyOpt| Ok(AppProxy::notify(opt)))
	}

	fn parse_keys(value: Value) -> mlua::Result<Vec<Key>> {
		Ok(match value {
			Value::String(s) => {
				vec![Key::from_str(&s.to_str()?).into_lua_err()?]
			}
			Value::Table(t) => {
				let mut v = Vec::with_capacity(10);
				for s in t.sequence_values::<mlua::String>() {
					v.push(Key::from_str(&s?.to_str()?).into_lua_err()?);
				}
				v
			}
			_ => Err("invalid `on`".into_lua_err())?,
		})
	}
}
