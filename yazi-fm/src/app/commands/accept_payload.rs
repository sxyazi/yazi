use anyhow::Result;
use mlua::IntoLua;
use tracing::error;
use yazi_binding::runtime_mut;
use yazi_dds::{LOCAL, REMOTE};
use yazi_macro::succ;
use yazi_parser::app::AcceptPayload;
use yazi_plugin::LUA;
use yazi_shared::event::Data;

use crate::{app::App, lives::Lives};

impl App {
	pub(crate) fn accept_payload(&mut self, opt: AcceptPayload) -> Result<Data> {
		let kind = opt.payload.body.kind().to_owned();
		let lock = if opt.payload.receiver == 0 || opt.payload.receiver != opt.payload.sender {
			REMOTE.read()
		} else {
			LOCAL.read()
		};

		let Some(handlers) = lock.get(&kind).filter(|&m| !m.is_empty()).cloned() else { succ!() };
		drop(lock);

		succ!(Lives::scope(&self.core, || {
			let body = opt.payload.body.into_lua(&LUA)?;
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
