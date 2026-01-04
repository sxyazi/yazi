use anyhow::Result;
use mlua::{ErrorContext, ExternalError, IntoLua, Value};
use yazi_binding::runtime_mut;
use yazi_dds::{LOCAL, spark::{Spark, SparkKind}};
use yazi_plugin::LUA;

use crate::{Ctx, lives::Lives};

pub struct Preflight;

impl Preflight {
	pub fn act<'a>(cx: &mut Ctx, opt: (SparkKind, Spark<'a>)) -> Result<Spark<'a>> {
		let kind = opt.0;
		let Some(handlers) = LOCAL.read().get(kind.as_ref()).filter(|&m| !m.is_empty()).cloned() else {
			return Ok(opt.1);
		};

		Ok(Lives::scope(cx.core, || {
			let mut body = opt.1.into_lua(&LUA)?;
			for (id, cb) in handlers {
				runtime_mut!(LUA)?.push(&id);
				let result = cb.call::<Value>(&body);
				runtime_mut!(LUA)?.pop();

				match result {
					Ok(Value::Nil) => {
						Err(format!("`{kind}` event cancelled by `{id}` plugin on preflight").into_lua_err())?
					}
					Ok(v) => body = v,
					Err(e) => Err(
						format!("Failed to run `{kind}` event handler in `{id}` plugin: {e}").into_lua_err(),
					)?,
				};
			}

			Spark::from_lua(&LUA, kind, body)
				.with_context(|e| format!("Unexpected return type from `{kind}` event handlers: {e}"))
		})?)
	}
}
