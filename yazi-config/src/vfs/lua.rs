use std::ops::Deref;

use hashbrown::HashMap;
use serde::Deserialize;
use tokio::sync::OnceCell;
use yazi_fs::engine::Capabilities;
use yazi_shared::{auth::AuthArc, data::{Data, DataKey}, event::Cmd};

#[derive(Deserialize)]
pub struct ServiceLua {
	#[serde(skip)]
	pub(crate) auth: AuthArc,
	#[serde(skip)]
	pub caps:        OnceCell<Capabilities>,
	run:             Cmd,
	#[serde(flatten)]
	pub opts:        HashMap<DataKey, Data>,
}

impl Deref for ServiceLua {
	type Target = Cmd;

	fn deref(&self) -> &Self::Target { &self.run }
}
