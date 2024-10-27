use mlua::IntoLua;
use tracing::error;
use yazi_dds::{LOCAL, Payload, REMOTE};
use yazi_plugin::LUA;
use yazi_shared::event::Cmd;

use crate::{app::App, lives::Lives};

impl App {
	pub(crate) fn accept_payload(&mut self, mut cmd: Cmd) {
		let Some(payload) = cmd.take_any::<Payload>("payload") else {
			return;
		};

		let kind = payload.body.kind().to_owned();
		let map = if payload.receiver == 0 || payload.receiver != payload.sender {
			REMOTE.read()
		} else {
			LOCAL.read()
		};

		let Some(map) = map.get(&kind).filter(|&m| !m.is_empty()) else {
			return;
		};

		_ = Lives::scope(&self.cx, || {
			let body = payload.body.into_lua(&LUA)?;
			for f in map.values() {
				if let Err(e) = f.call::<()>(body.clone()) {
					error!("Failed to call `{kind}` handler: {e}");
				}
			}
			Ok(())
		});
	}
}
