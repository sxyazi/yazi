use anyhow::Result;
use mlua::IntoLua;
use yazi_actor::lives::Lives;
use yazi_binding::runtime_scope;
use yazi_core::Ctx;
use yazi_dds::{LOCAL, Payload, REMOTE};
use yazi_macro::{log_if_err, succ};
use yazi_plugin::LUA;
use yazi_shared::data::Data;

use crate::Actor;

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
		succ!(Lives::scope(cx, |cx| {
			let body = payload.body.into_lua(&LUA)?;
			for (name, cb) in handlers {
				log_if_err!(
					runtime_scope!(cx, &name, cb.call::<()>(body.clone())),
					"`{kind}` event handler in `{name}` plugin",
				);
			}
			Ok(())
		})?);
	}
}
