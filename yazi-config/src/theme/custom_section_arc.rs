use std::{ops::Deref, sync::Arc};

use hashbrown::HashMap;
use mlua::{MetaMethod, UserData, UserDataMethods};
use yazi_shared::SnakeCasedString;

use crate::theme::{CustomField, CustomSection};

pub struct CustomSectionArc(Arc<HashMap<SnakeCasedString, CustomField>>);

impl Deref for CustomSectionArc {
	type Target = Arc<HashMap<SnakeCasedString, CustomField>>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl From<&CustomSection> for CustomSectionArc {
	fn from(value: &CustomSection) -> Self { Self(value.load_full()) }
}

impl UserData for CustomSectionArc {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_method(MetaMethod::Index, |_, me, key: mlua::String| {
			Ok(me.get(&*key.to_str()?).cloned())
		});
	}
}
