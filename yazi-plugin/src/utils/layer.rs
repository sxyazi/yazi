use std::{str::FromStr, time::Duration};

use mlua::{ExternalError, ExternalResult, Function, IntoLuaMulti, Lua, Table, Value};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use yazi_config::{keymap::{Chord, Key}, popup::{ConfirmCfg, InputCfg}};
use yazi_macro::emit;
use yazi_proxy::{AppProxy, ConfirmProxy, InputProxy};
use yazi_shared::{Debounce, event::Cmd};

use super::Utils;
use crate::{bindings::InputRx, deprecate, elements::{Line, Pos, Text}};

impl Utils {
	pub(super) fn which(lua: &Lua) -> mlua::Result<Function> {
		lua.create_async_function(|_, t: Table| async move {
			let (tx, mut rx) = mpsc::channel::<usize>(1);

			let mut cands = Vec::with_capacity(30);
			for (i, cand) in t.raw_get::<Table>("cands")?.sequence_values::<Table>().enumerate() {
				let cand = cand?;
				cands.push(Chord {
					on:    Self::parse_keys(cand.raw_get("on")?)?,
					run:   vec![Cmd::args("which:callback", [i]).with_any("tx", tx.clone())],
					desc:  cand.raw_get("desc").ok(),
					r#for: None,
				});
			}

			drop(tx);
			emit!(Call(
				Cmd::new("which:show")
					.with_any("candidates", cands)
					.with("silent", t.raw_get::<bool>("silent").unwrap_or_default())
			));

			Ok(rx.recv().await.map(|idx| idx + 1))
		})
	}

	pub(super) fn input(lua: &Lua) -> mlua::Result<Function> {
		lua.create_async_function(|lua, t: Table| async move {
			let mut pos = t.raw_get::<Value>("pos")?;
			if pos.is_nil() {
				pos = t.raw_get("position")?;
				if !pos.is_nil() {
					deprecate!(
						lua,
						"The `position` property of `ya.input()` is deprecated, use `pos` instead in your {}\nSee #2921 for more details: https://github.com/sxyazi/yazi/pull/2921"
					);
				}
			}

			let realtime = t.raw_get("realtime").unwrap_or_default();
			let rx = UnboundedReceiverStream::new(InputProxy::show(InputCfg {
				title: t.raw_get("title")?,
				value: t.raw_get("value").unwrap_or_default(),
				cursor: None, // TODO
				obscure: t.raw_get("obscure").unwrap_or_default(),
				position: Pos::new_input(pos)?.into(),
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

		lua.create_async_function(|_, t: Table| async move {
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
		lua.create_function(|_, t: Table| {
			AppProxy::notify(t.try_into()?);
			Ok(())
		})
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
