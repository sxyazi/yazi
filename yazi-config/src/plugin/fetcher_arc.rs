use std::{ops::{Deref, DerefMut}, sync::Arc};

use mlua::{UserData, UserDataFields};
use serde::Deserialize;
use yazi_shim::mlua::UserDataFieldsExt;

use crate::{Mixable, plugin::Fetcher};

#[derive(Clone, Debug, Deserialize)]
pub struct FetcherArc(Arc<Fetcher>);

impl Deref for FetcherArc {
	type Target = Arc<Fetcher>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl DerefMut for FetcherArc {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl From<Fetcher> for FetcherArc {
	fn from(value: Fetcher) -> Self { Self(value.into()) }
}

impl Mixable for FetcherArc {}

impl UserData for FetcherArc {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("id", |_, me| Ok(me.id));

		fields.add_cached_field("name", |lua, me| lua.create_string(&*me.name));
	}
}
