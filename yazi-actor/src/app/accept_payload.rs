use anyhow::Result;
use mlua::IntoLua;
use tracing::error;
use yazi_actor::lives::Lives;
use yazi_binding::runtime_scope;
use yazi_dds::{LOCAL, Payload, REMOTE};
use yazi_macro::succ;
use yazi_plugin::LUA;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct AcceptPayload;

impl Actor for AcceptPayload {
	type Form = Payload<'static>;

	const NAME: &str = "accept_payload";

	fn act(cx: &mut Ctx, payload: Payload) -> Result<Data> {
		let kind = payload.body.kind();
		let lock = if payload.receiver == 0 || payload.receiver != payload.sender {
			REMOTE.read()
		} else {
			LOCAL.read()
		};

		let Some(handlers) = lock.get(kind).filter(|&m| !m.is_empty()).cloned() else { succ!() };
		drop(lock);

		let kind = kind.to_owned();
		succ!(Lives::scope(cx.core, |_| {
			let body = payload.body.into_lua(&LUA)?;
			for (name, cb) in handlers {
				if let Err(e) = runtime_scope!(LUA, &name, cb.call::<()>(body.clone())) {
					error!("Failed to run `{kind}` event handler in your `{name}` plugin: {e}");
				}
			}
			Ok(())
		})?);
	}
}
