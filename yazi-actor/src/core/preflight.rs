use anyhow::Result;
use mlua::{ErrorContext, ExternalError, IntoLua, Value};
use yazi_binding::runtime_scope;
use yazi_core::Ctx;
use yazi_dds::{Pubsub, Subscription};
use yazi_parser::spark::{Spark, SparkKind};
use yazi_plugin::LUA;

use crate::lives::Lives;

pub struct Preflight;

impl Preflight {
	pub fn act<'a>(cx: &mut Ctx, opt: (SparkKind, Spark<'a>)) -> Result<Spark<'a>> {
		let kind = opt.0;
		let subscriptions = Pubsub::subscriptions(kind.into(), false);
		if subscriptions.is_empty() {
			return Ok(opt.1);
		}

		Ok(Lives::scope(cx, |cx| {
			let mut body = opt.1.into_lua(&LUA)?;
			for Subscription { plugin, callback, .. } in subscriptions {
				match runtime_scope!(cx, &plugin, callback.call::<Value>(&body)) {
					Ok(Value::Nil) => Err(
						format!("`{kind}` event cancelled by `{plugin}` plugin on preflight").into_lua_err(),
					)?,
					Ok(v) => body = v,
					Err(e) => Err(
						format!("Failed to run `{kind}` event handler in `{plugin}` plugin: {e}")
							.into_lua_err(),
					)?,
				};
			}

			Spark::from_lua(&LUA, kind, body)
				.with_context(|e| format!("Unexpected return type from `{kind}` event handlers: {e}"))
		})?)
	}
}
