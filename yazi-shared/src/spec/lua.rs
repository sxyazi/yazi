use mlua::{UserData, UserDataFields, UserDataRegistry};
use yazi_shim::{mlua::UserDataFieldsExt, strum::IntoStr};

use crate::{auth::AuthKind, spec::{Spec, SpecInventory}};

impl UserData for Spec {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_cached_field("kind", |_, me| Ok(me.kind.into_str()));
		fields.add_cached_field("domain", |lua, me| lua.create_string(&*me.domain));
		fields.add_field_method_get("is_regular", |_, me| Ok(me.kind == AuthKind::Regular));
		fields.add_field_method_get("is_search", |_, me| Ok(me.kind == AuthKind::Search));
		fields.add_field_method_get("is_virtual", |_, me| Ok(me.kind.is_virtual()));
	}

	fn register(registry: &mut UserDataRegistry<Self>) {
		Self::add_fields(registry);
		Self::add_methods(registry);

		for inv in inventory::iter::<SpecInventory>() {
			(inv.register)(registry);
		}
	}
}
