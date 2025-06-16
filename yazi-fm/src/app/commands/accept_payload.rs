use mlua::IntoLua;
use tracing::error;
use yazi_dds::{LOCAL, Payload, REMOTE};
use yazi_plugin::{LUA, runtime_mut};
use yazi_shared::event::CmdCow;

use crate::{app::App, lives::Lives};

impl App {
	pub(crate) fn accept_payload(&mut self, mut cmd: CmdCow) {
		let Some(payload) = cmd.take_any::<Payload>("payload") else {
			return;
		};

		let kind = payload.body.kind().to_owned();
		let lock = if payload.receiver == 0 || payload.receiver != payload.sender {
			REMOTE.read()
		} else {
			LOCAL.read()
		};

		let Some(handlers) = lock.get(&kind).filter(|&m| !m.is_empty()).cloned() else {
			return;
		};
		drop(lock);

		_ = Lives::scope(&self.cx, || {
			let body = payload.body.into_lua(&LUA)?;
			for (id, cb) in handlers {
				runtime_mut!(LUA)?.push(&id);
				if let Err(e) = cb.call::<()>(body.clone()) {
					error!("Failed to run `{kind}` event handler in your `{id}` plugin: {e}");
				}
				runtime_mut!(LUA)?.pop();
			}
			Ok(())
		});
	}
}
