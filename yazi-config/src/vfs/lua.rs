use std::{ops::Deref, sync::Arc};

use serde::Deserialize;
use yazi_shared::{auth::Auth, event::Cmd};

#[derive(Deserialize)]
pub struct ServiceLua {
	#[serde(skip, default)]
	pub auth: Arc<Auth>,
	pub run:  Cmd,
}

impl Deref for ServiceLua {
	type Target = Auth;

	fn deref(&self) -> &Self::Target { &self.auth }
}
