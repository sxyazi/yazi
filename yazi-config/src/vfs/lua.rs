use std::{ops::Deref, sync::Arc};

use hashbrown::HashMap;
use serde::Deserialize;
use yazi_shared::{auth::Auth, data::{Data, DataKey}, event::Cmd};

#[derive(Deserialize)]
pub struct ServiceLua {
	#[serde(skip)]
	pub(crate) auth: Arc<Auth>,
	run:             Cmd,
	#[serde(flatten)]
	pub opts:        HashMap<DataKey, Data>,
}

impl Deref for ServiceLua {
	type Target = Cmd;

	fn deref(&self) -> &Self::Target { &self.run }
}
