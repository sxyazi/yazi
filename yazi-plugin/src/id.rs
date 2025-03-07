use std::ops::Deref;

use mlua::{FromLua, UserData};

#[derive(Clone, Copy, FromLua)]
pub struct Id(pub yazi_shared::Id);

impl Deref for Id {
	type Target = yazi_shared::Id;

	#[inline]
	fn deref(&self) -> &Self::Target { &self.0 }
}

impl UserData for Id {}
