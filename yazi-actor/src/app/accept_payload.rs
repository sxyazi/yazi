use anyhow::Result;
use mlua::IntoLua;
use yazi_actor::lives::Lives;
use yazi_binding::runtime_scope;
use yazi_core::Ctx;
use yazi_dds::{Payload, Pubsub, Subscription};
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
		let scope = cx.scope.for_tab(payload.tab());

		let subscriptions =
			Pubsub::subscriptions(kind, payload.receiver == 0 || payload.receiver != payload.sender);
		if subscriptions.is_empty() {
			succ!();
		}

		let kind = kind.to_owned();
		succ!(Lives::scope(cx, |_| {
			let body = payload.body.into_lua(&LUA)?;
			for Subscription { plugin, callback, .. } in subscriptions {
				log_if_err!(
					runtime_scope!(scope, &plugin, callback.call::<()>(body.clone())),
					"`{kind}` event handler in `{plugin}` plugin",
				);
			}
			Ok(())
		})?);
	}
}
