use mlua::{UserData, UserDataFields, UserDataRegistry};
use yazi_shim::{mlua::UserDataFieldsExt, strum::IntoStr};

use crate::scheme::{Scheme, SchemeInventory, SchemeLike};

impl UserData for Scheme {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_cached_field("kind", |_, me| Ok(me.kind().into_str()));
		fields.add_field_method_get("is_virtual", |_, me| Ok(me.is_virtual()));
	}

	fn register(registry: &mut UserDataRegistry<Self>) {
		Self::add_fields(registry);
		Self::add_methods(registry);

		for inv in inventory::iter::<SchemeInventory>() {
			(inv.register)(registry);
		}
	}
}
