use std::ops::Deref;

use mlua::{UserData, UserDataFields};
use yazi_fs::FsScheme;
use yazi_shared::scheme::SchemeLike;
use yazi_shim::{mlua::UserDataFieldsExt, strum::IntoStr};

use crate::Path;

pub struct Scheme {
	inner: yazi_shared::scheme::Scheme,
}

impl Deref for Scheme {
	type Target = yazi_shared::scheme::Scheme;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl Scheme {
	pub fn new(scheme: impl Into<yazi_shared::scheme::Scheme>) -> Self {
		Self { inner: scheme.into() }
	}
}

impl UserData for Scheme {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_cached_field("kind", |_, me| Ok(me.kind().into_str()));
		fields.add_cached_field("cache", |_, me| Ok(me.cache().map(Path::new)));

		fields.add_field_method_get("is_virtual", |_, me| Ok(me.is_virtual()));
	}
}
