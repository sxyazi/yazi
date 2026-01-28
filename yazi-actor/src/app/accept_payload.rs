use anyhow::Result;
use mlua::IntoLua;
use tracing::error;
use yazi_actor::lives::Lives;
use yazi_binding::runtime_mut;
use yazi_dds::{LOCAL, Payload, REMOTE};
use yazi_macro::succ;
use yazi_plugin::LUA;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct AcceptPayload;

impl Actor for AcceptPayload {
	type Options = Payload<'static>;

	const NAME: &str = "accept_payload";

	fn act(cx: &mut Ctx, payload: Payload) -> Result<Data> {
		let kind = payload.body.kind().to_owned();
		let lock = if payload.receiver == 0 || payload.receiver != payload.sender {
			REMOTE.read()
		} else {
			LOCAL.read()
		};

		let Some(handlers) = lock.get(&kind).filter(|&m| !m.is_empty()).cloned() else { succ!() };
		drop(lock);

		succ!(Lives::scope(&cx.core, || {
			let body = payload.body.into_lua(&LUA)?;
			for (id, cb) in handlers {
				let blocking = runtime_mut!(LUA)?.critical_push(&id, true);
				if let Err(e) = cb.call::<()>(body.clone()) {
					error!("Failed to run `{kind}` event handler in your `{id}` plugin: {e}");
				}
				runtime_mut!(LUA)?.critical_pop(blocking);
			}
			Ok(())
		})?);
	}
}
