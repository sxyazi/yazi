use std::marker::PhantomData;

use anyhow::Result;
use mlua::{IntoLua, Value};
use tracing::error;
use yazi_binding::runtime_mut;
use yazi_dds::{LOCAL, body::Body};
use yazi_macro::succ;
use yazi_plugin::LUA;
use yazi_shared::event::Data;

use crate::{Actor, Ctx, lives::Lives};

pub struct Preflight<'a> {
	_lifetime: PhantomData<&'a ()>,
}

impl<'a> Actor for Preflight<'a> {
	type Options = (&'static str, Body<'a>);

	const NAME: &'static str = "preflight";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		let kind = opt.0;
		let Some(handlers) = LOCAL.read().get(kind).filter(|&m| !m.is_empty()).cloned() else {
			succ!(false)
		};

		succ!(Lives::scope(cx.core, || {
			let body = opt.1.into_lua(&LUA)?;
			for (id, cb) in handlers {
				runtime_mut!(LUA)?.push(&id);
				let result = cb.call::<Value>(body.clone());
				runtime_mut!(LUA)?.pop();

				match result {
					Ok(Value::Boolean(true)) => return Ok(true),
					Ok(Value::Nil | Value::Boolean(false)) => {}
					Ok(v) => {
						error!("Unexpected return type from `{kind}` event handler in `{id}` plugin: {v:?}")
					}
					Err(e) => error!("Failed to run `{kind}` event handler in `{id}` plugin: {e}"),
				}
			}
			Ok(false)
		})?)
	}
}
