use anyhow::{Result, bail};
use mlua::IntoLua;
use tracing::error;
use yazi_actor::lives::Lives;
use yazi_binding::runtime_mut;
use yazi_dds::{LOCAL, Payload, REMOTE};
use yazi_macro::succ;
use yazi_plugin::LUA;
use yazi_shared::{data::Data, event::CmdCow};

use crate::app::App;

impl App {
	pub(crate) fn accept_payload(&self, mut c: CmdCow) -> Result<Data> {
		let Some(payload) = c.take_any2::<Payload>("payload").transpose()? else {
			bail!("'payload' is required for accept_payload");
		};

		let kind = payload.body.kind().to_owned();
		let lock = if payload.receiver == 0 || payload.receiver != payload.sender {
			REMOTE.read()
		} else {
			LOCAL.read()
		};

		let Some(handlers) = lock.get(&kind).filter(|&m| !m.is_empty()).cloned() else { succ!() };
		drop(lock);

		succ!(Lives::scope(&self.core, || {
			let body = payload.body.into_lua(&LUA)?;
			for (id, cb) in handlers {
				runtime_mut!(LUA)?.push(&id);
				if let Err(e) = cb.call::<()>(body.clone()) {
					error!("Failed to run `{kind}` event handler in your `{id}` plugin: {e}");
				}
				runtime_mut!(LUA)?.pop();
			}
			Ok(())
		})?);
	}
}
