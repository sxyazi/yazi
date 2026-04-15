use std::{ops::Deref, sync::Arc};

use mlua::{UserData, UserDataFields, Value};

use crate::{Id, cached_field};

#[derive(Clone)]
pub struct Fetcher {
	inner: Arc<yazi_config::plugin::Fetcher>,

	v_name: Option<Value>,
}

impl Deref for Fetcher {
	type Target = yazi_config::plugin::Fetcher;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl Fetcher {
	pub fn new(inner: impl Into<Arc<yazi_config::plugin::Fetcher>>) -> Self {
		Self { inner: inner.into(), v_name: None }
	}
}

impl UserData for Fetcher {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("id", |_, me| Ok(Id(me.id)));

		cached_field!(fields, name, |lua, me| lua.create_string(&*me.name));
	}
}
