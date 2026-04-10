use std::{str::FromStr, time::Duration};

use mlua::{ExternalError, ExternalResult, Function, IntoLuaMulti, Lua, Table, Value};
use tokio_stream::wrappers::UnboundedReceiverStream;
use yazi_binding::{InputRx, elements::{Line, Pos, Text}, runtime};
use yazi_config::{Platform, keymap::{Chord, ChordCow, Key}, popup::{ConfirmCfg, InputCfg}};
use yazi_core::notify::MessageOpt;
use yazi_macro::relay;
use yazi_proxy::{ConfirmProxy, InputProxy, NotifyProxy, WhichProxy};
use yazi_shared::{Debounce, Layer};

use super::Utils;

impl Utils {
	pub(super) fn which(lua: &Lua) -> mlua::Result<Function> {
		lua.create_async_function(|lua, t: Table| async move {
			if runtime!(lua)?.blocking {
				return Err("Cannot call `ya.which()` while main thread is blocked".into_lua_err());
			}

			let cands: Vec<_> = t
				.raw_get::<Table>("cands")?
				.sequence_values::<Table>()
				.enumerate()
				.map(|(i, cand)| {
					let cand = cand?;
					Ok(ChordCow::Owned(Chord {
						on:    Self::parse_keys(cand.raw_get("on")?)?,
						run:   vec![relay!(which:callback, [i + 1])],
						desc:  cand.raw_get("desc").ok(),
						r#for: Platform::All,
					}))
				})
				.collect::<mlua::Result<_>>()?;

			let idx: Option<usize> = WhichProxy::activate(cands, t.raw_get("silent")?)
				.await
				.iter()
				.flat_map(|chord| &chord.run)
				.find(|action| action.layer == Layer::Which && action.name == "callback")
				.and_then(|action| action.first().ok());

			Ok(idx)
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
				position: t.raw_get::<Pos>("pos")?.with_height(3).into(),
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
		lua.create_async_function(|lua, t: Table| async move {
			if runtime!(lua)?.blocking {
				return Err("Cannot call `ya.confirm()` while main thread is blocked".into_lua_err());
			}

			let result = ConfirmProxy::show(ConfirmCfg {
				position: t.raw_get::<Pos>("pos")?.into(),
				title:    t.raw_get::<Line>("title")?.into(),
				body:     t.raw_get::<Option<Text>>("body")?.unwrap_or_default().into(),
				list:     Default::default(), // TODO
			});

			Ok(result.await)
		})
	}

	pub(super) fn notify(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|_, opt: MessageOpt| Ok(NotifyProxy::push(opt)))
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
