use std::ops::Deref;

use mlua::{AnyUserData, UserData, UserDataFields, Value};
use yazi_binding::cached_field;

use super::{Lives, PtrCell};

pub(super) struct Which {
	inner: PtrCell<yazi_core::which::Which>,

	v_tx:    Option<Value>,
	v_cands: Option<Value>,
}

impl Deref for Which {
	type Target = yazi_core::which::Which;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl Which {
	pub(super) fn make(inner: &yazi_core::which::Which) -> mlua::Result<AnyUserData> {
		Lives::scoped_userdata(Self { inner: inner.into(), v_tx: None, v_cands: None })
	}
}

impl UserData for Which {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		cached_field!(fields, tx, |_, me| Ok(me.tx.clone().map(yazi_binding::MpscUnboundedTx)));
		fields.add_field_method_get("times", |_, me| Ok(me.inner.times));
		cached_field!(fields, cands, |lua, me| {
			lua.create_sequence_from(me.inner.cands.iter().cloned().map(yazi_binding::ChordCow))
		});

		fields.add_field_method_get("active", |_, me| Ok(me.inner.active));
		fields.add_field_method_get("silent", |_, me| Ok(me.inner.silent));
	}
}
