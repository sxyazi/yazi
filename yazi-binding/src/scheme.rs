use std::ops::Deref;

use mlua::{UserData, UserDataFields, Value};
use yazi_fs::FsScheme;
use yazi_shared::scheme::SchemeLike;

use crate::{Url, cached_field};

pub struct Scheme {
	inner: yazi_shared::scheme::Scheme,

	v_kind:  Option<Value>,
	v_cache: Option<Value>,
}

impl Deref for Scheme {
	type Target = yazi_shared::scheme::Scheme;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl Scheme {
	pub fn new(scheme: &yazi_shared::scheme::Scheme) -> Self {
		Self { inner: scheme.clone(), v_kind: None, v_cache: None }
	}
}

impl UserData for Scheme {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		cached_field!(fields, kind, |_, me| Ok(me.kind()));
		cached_field!(fields, cache, |_, me| Ok(me.cache().map(Url::new)));

		fields.add_field_method_get("is_virtual", |_, me| Ok(me.is_virtual()));
	}
}
