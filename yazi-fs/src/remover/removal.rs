use std::io;

use mlua::{AnyUserData, IntoLuaMulti, LuaSerdeExt, MetaMethod, UserData, UserDataMethods, Value};
use tokio::{sync::mpsc, task::JoinHandle};
use yazi_shim::{fs::Error, mlua::SER_OPT};

use super::Delta;

pub struct Removal {
	rx:     Option<mpsc::Receiver<io::Result<Delta>>>,
	handle: JoinHandle<()>,
}

impl Removal {
	pub fn new(rx: mpsc::Receiver<io::Result<Delta>>, handle: JoinHandle<()>) -> Self {
		Self { rx: Some(rx), handle }
	}

	pub async fn recv(&mut self) -> Option<io::Result<Delta>> {
		if let Some(result) = self.rx.as_mut()?.recv().await {
			if result.is_err() {
				self.rx = None;
			}
			return Some(result);
		}

		let result = (&mut self.handle).await;
		self.rx = None;
		result.err().map(|e| Err(e.into()))
	}

	pub async fn total(mut self) -> io::Result<Delta> {
		let mut total = Delta::default();
		while let Some(delta) = self.recv().await {
			total = total.try_add(delta?).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
		}
		Ok(total)
	}

	pub async fn finish(self) -> io::Result<()> {
		let total = self.total().await?;
		if total.failed == 0 {
			Ok(())
		} else {
			Err(io::Error::other(format!("Failed to remove {} entries", total.failed)))
		}
	}
}

impl UserData for Removal {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_async_meta_method_mut(MetaMethod::Call, |lua, mut me, ()| async move {
			match me.recv().await.transpose() {
				Ok(delta) => (lua.to_value_with(&delta, SER_OPT)?, Value::Nil).into_lua_multi(&lua),
				Err(e) => (false, Error::from(e)).into_lua_multi(&lua),
			}
		});
		methods.add_meta_function(MetaMethod::Close, |_, ud: AnyUserData| ud.destroy());
	}
}
