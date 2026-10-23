use std::time::Duration;

use mlua::{ExternalError, ExternalResult, Function, IntoLuaMulti, Lua, LuaString, Table, Value};
use tokio_stream::wrappers::UnboundedReceiverStream;
use yazi_binding::{elements::{Line, Text}, runtime};
use yazi_config::{Platform, keymap::{Chord, ChordArc, Key}, popup::ConfirmCfg};
use yazi_core::notify::MessageOpt;
use yazi_macro::relay;
use yazi_proxy::{ConfirmProxy, InputProxy, NotifyProxy, WhichProxy};
use yazi_shared::{Debounce, Layer};
use yazi_widgets::input::{InputOpt, InputStream};

use super::Utils;

impl Utils {
	pub(super) fn which(lua: &Lua) -> mlua::Result<Function> {
		lua.create_async_function(|lua, t: Table| async move {
			if runtime!(lua)?.is_blocking() {
				return Err("Cannot call `ya.which()` while main thread is blocked".into_lua_err());
			}

			let cands: Vec<_> = t
				.raw_get::<Table>("cands")?
				.sequence_values::<Table>()
				.enumerate()
				.map(|(i, cand)| {
					let cand = cand?;
					Ok(ChordArc::from(Chord {
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
			if runtime!(lua)?.is_blocking() {
				return Err("Cannot call `ya.input()` while main thread is blocked".into_lua_err());
			}

			let opt = InputOpt::try_from(&t)?;
			let realtime = opt.realtime;

			let secs = t.raw_get::<f64>("debounce").unwrap_or_default();
			let dur = Duration::try_from_secs_f64(secs).into_lua_err()?;
			let rx = UnboundedReceiverStream::new(InputProxy::show(opt));

			if !realtime {
				InputStream::consume(rx).await.into_lua_multi(&lua)
			} else if dur.is_zero() {
				InputStream::new(rx).into_lua_multi(&lua)
			} else {
				InputStream::new(Debounce::new(rx, dur)).into_lua_multi(&lua)
			}
		})
	}

	pub(super) fn confirm(lua: &Lua) -> mlua::Result<Function> {
		lua.create_async_function(|lua, t: Table| async move {
			if runtime!(lua)?.is_blocking() {
				return Err("Cannot call `ya.confirm()` while main thread is blocked".into_lua_err());
			}

			let result = ConfirmProxy::show(ConfirmCfg {
				position: t.raw_get("pos")?,
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
		match value {
			Value::String(s) => Ok(vec![s.to_str()?.parse()?]),
			Value::Table(t) => {
				t.sequence_values::<LuaString>().map(|s| Ok(s?.to_str()?.parse()?)).collect()
			}
			_ => Err("invalid `on`".into_lua_err()),
		}
	}
}
