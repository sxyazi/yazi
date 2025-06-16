use std::ops::Deref;

use mlua::{UserData, UserDataFields, Value};

use crate::{Style, cached_field};

pub struct Icon {
	inner: &'static yazi_config::Icon,

	v_text:  Option<Value>,
	v_style: Option<Value>,
}

impl Deref for Icon {
	type Target = yazi_config::Icon;

	fn deref(&self) -> &Self::Target { self.inner }
}

impl From<&'static yazi_config::Icon> for Icon {
	fn from(icon: &'static yazi_config::Icon) -> Self {
		Self { inner: icon, v_text: None, v_style: None }
	}
}

impl UserData for Icon {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		cached_field!(fields, text, |lua, me| lua.create_string(&me.text));
		cached_field!(fields, style, |_, me| Ok(Style::from(me.style)));
	}
}
