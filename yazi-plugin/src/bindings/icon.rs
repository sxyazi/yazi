use std::ops::Deref;

use mlua::{UserData, UserDataFields};

use crate::elements::Style;

pub struct Icon(&'static yazi_shared::theme::Icon);

impl Deref for Icon {
	type Target = yazi_shared::theme::Icon;

	fn deref(&self) -> &Self::Target { self.0 }
}

impl From<&'static yazi_shared::theme::Icon> for Icon {
	fn from(icon: &'static yazi_shared::theme::Icon) -> Self { Self(icon) }
}

impl UserData for Icon {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("text", |lua, me| lua.create_string(&me.text));
		fields.add_field_method_get("style", |_, me| Ok(Style::from(me.style)));
	}
}
