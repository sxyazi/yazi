use std::{str::FromStr, time::Duration};

use mlua::{ExternalError, ExternalResult, Function, IntoLuaMulti, Lua, Table, Value};
use tokio_stream::wrappers::UnboundedReceiverStream;
use yazi_binding::{elements::{Line, Pos, Text}, runtime};
use yazi_config::{Platform, keymap::{Chord, ChordArc}, popup::ConfirmCfg};
use yazi_core::notify::MessageOpt;
use yazi_macro::relay;
use yazi_proxy::{ConfirmProxy, InputProxy, NotifyProxy, WhichProxy};
use yazi_shared::{Debounce, Layer};
use yazi_term::event::KeyEvent;
use yazi_widgets::input::{InputOpt, InputStream};

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
					Ok(ChordArc::from(Chord::<{ Layer::Null as u8 }> {
						id:    yazi_config::keymap::chord_id(),
						on:    Self::parse_keys(cand.raw_get("on")?)?,
						run:   relay!(which:callback, [i + 1]).into(),
						desc:  cand.raw_get("desc").unwrap_or_default(),
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

			let opt = InputOpt::try_from(&t)?;
			let realtime = opt.realtime;

			let rx = UnboundedReceiverStream::new(InputProxy::show(opt));
			if !realtime {
				return InputStream::consume(rx).await.into_lua_multi(&lua);
			}

			let debounce = t.raw_get::<f64>("debounce").unwrap_or_default();
			if debounce < 0.0 {
				Err("negative debounce duration".into_lua_err())
			} else if debounce == 0.0 {
				InputStream::new(rx).into_lua_multi(&lua)
			} else {
				InputStream::new(Debounce::new(rx, Duration::from_secs_f64(debounce))).into_lua_multi(&lua)
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

	fn parse_keys(value: Value) -> mlua::Result<Vec<KeyEvent>> {
		Ok(match value {
			Value::String(s) => {
				vec![KeyEvent::from_str(&s.to_str()?).into_lua_err()?]
			}
			Value::Table(t) => {
				let mut v = Vec::with_capacity(10);
				for s in t.sequence_values::<mlua::String>() {
					v.push(KeyEvent::from_str(&s?.to_str()?).into_lua_err()?);
				}
				v
			}
			_ => Err("invalid `on`".into_lua_err())?,
		})
	}
}
