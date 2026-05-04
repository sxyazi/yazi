use std::sync::Arc;

use hashbrown::HashMap;
use mlua::{IntoLua, MetaMethod, UserData, UserDataMethods, Value};
use yazi_shared::SnakeCasedString;

use crate::theme::CustomField;

pub struct CustomSection {
	inner: Arc<HashMap<SnakeCasedString, yazi_config::theme::CustomField>>,
}

impl CustomSection {
	pub fn new(inner: Arc<HashMap<SnakeCasedString, yazi_config::theme::CustomField>>) -> Self {
		Self { inner }
	}
}

impl UserData for CustomSection {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_method(MetaMethod::Index, |lua, me, key: mlua::String| {
			match me.inner.get(&*key.to_str()?) {
				Some(value) => CustomField::new(value).into_lua(lua),
				None => Ok(Value::Nil),
			}
		});
	}
}
