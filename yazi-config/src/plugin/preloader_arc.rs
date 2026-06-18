use std::{ops::{Deref, DerefMut}, sync::Arc};

use mlua::{UserData, UserDataFields};
use serde::Deserialize;
use yazi_shim::mlua::UserDataFieldsExt;

use crate::{Mixable, plugin::Preloader};

#[derive(Clone, Debug, Deserialize)]
pub struct PreloaderArc(Arc<Preloader>);

impl Deref for PreloaderArc {
	type Target = Arc<Preloader>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl DerefMut for PreloaderArc {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl From<Preloader> for PreloaderArc {
	fn from(value: Preloader) -> Self { Self(value.into()) }
}

impl Mixable for PreloaderArc {}

impl UserData for PreloaderArc {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("id", |_, me| Ok(me.id));

		fields.add_cached_field("name", |lua, me| lua.create_string(&*me.name));
	}
}
