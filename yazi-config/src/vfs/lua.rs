use std::ops::Deref;

use hashbrown::HashMap;
use mlua::{UserData, UserDataFields};
use serde::Deserialize;
use tokio::sync::OnceCell;
use yazi_fs::engine::Capabilities;
use yazi_shared::{auth::AuthArc, data::{Data, DataKey}, event::Cmd, sendable::Sendable};
use yazi_shim::mlua::UserDataFieldsExt;

#[derive(Clone, Debug, Deserialize)]
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

impl UserData for ServiceLua {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_cached_field("opts", |lua, me| Sendable::args_to_table_ref(lua, &me.opts));
	}
}
