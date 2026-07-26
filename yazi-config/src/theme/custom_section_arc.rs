use std::{ops::Deref, sync::Arc};

use hashbrown::HashMap;
use mlua::{LuaString, MetaMethod, UserData, UserDataMethods};
use yazi_shared::SnakeCasedKey;

use crate::theme::{CustomField, CustomSection};

pub struct CustomSectionArc(Arc<HashMap<SnakeCasedKey, CustomField>>);

impl Deref for CustomSectionArc {
	type Target = Arc<HashMap<SnakeCasedKey, CustomField>>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl From<&CustomSection> for CustomSectionArc {
	fn from(value: &CustomSection) -> Self { Self(value.load_full()) }
}

impl UserData for CustomSectionArc {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_method(MetaMethod::Index, |_, me, key: LuaString| {
			Ok(me.get(&*key.to_str()?).cloned())
		});
	}
}
