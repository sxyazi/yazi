use mlua::{UserData, UserDataFields, UserDataRegistry};
use yazi_shim::{mlua::UserDataFieldsExt, strum::IntoStr};

use crate::{sendable::Sendable, spec::{Spec, SpecInventory}};

impl UserData for Spec {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_cached_field("kind", |_, me| Ok(me.kind.into_str()));
		fields.add_cached_field("scheme", |lua, me| lua.create_string(&me.scheme));
		fields.add_cached_field("domain", |lua, me| lua.create_string(&*me.domain));
		fields.add_cached_field("data", |lua, me| {
			me.view.data().map(|d| Sendable::wire_to_value_ref(lua, d)).transpose()
		});
		fields.add_field_method_get("is_regular", |_, me| Ok(me.kind.is_regular()));
		fields.add_field_method_get("is_view", |_, me| Ok(me.kind.is_view()));
	}

	fn register(registry: &mut UserDataRegistry<Self>) {
		Self::add_fields(registry);
		Self::add_methods(registry);

		for inv in inventory::iter::<SpecInventory>() {
			(inv.register)(registry);
		}
	}
}
