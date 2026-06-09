use std::ops::Deref;

use mlua::{AnyUserData, UserData, UserDataFields};
use yazi_shim::mlua::UserDataFieldsExt;

use super::{Lives, PtrCell};

pub(super) struct Which {
	inner: PtrCell<yazi_core::which::Which>,
}

impl Deref for Which {
	type Target = yazi_core::which::Which;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl Which {
	pub(super) fn make(inner: &yazi_core::which::Which) -> mlua::Result<AnyUserData> {
		Lives::scoped_userdata(Self { inner: inner.into() })
	}
}

impl UserData for Which {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_cached_field("tx", |_, me| Ok(me.tx.clone().map(yazi_binding::MpscUnboundedTx)));
		fields.add_field_method_get("times", |_, me| Ok(me.inner.times));
		fields.add_cached_field("cands", |lua, me| {
			lua.create_sequence_from(me.inner.cands.iter().cloned().map(yazi_binding::keymap::ChordCow))
		});

		fields.add_field_method_get("active", |_, me| Ok(me.inner.active));
		fields.add_field_method_get("silent", |_, me| Ok(me.inner.silent));
	}
}
