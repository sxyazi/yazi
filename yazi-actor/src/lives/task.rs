use std::ops::Deref;

use mlua::{AnyUserData, LuaSerdeExt, UserData, UserDataFields};
use yazi_scheduler::Progress;
use yazi_shim::mlua::{SER_OPT, UserDataFieldsExt};

use super::{Lives, PtrCell};

pub(super) struct TaskSnap {
	inner: PtrCell<yazi_scheduler::TaskSnap>,
}

impl Deref for TaskSnap {
	type Target = yazi_scheduler::TaskSnap;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl TaskSnap {
	pub(super) fn make(inner: &yazi_scheduler::TaskSnap) -> mlua::Result<AnyUserData> {
		Lives::scoped_userdata(Self { inner: inner.into() })
	}
}

impl UserData for TaskSnap {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_cached_field("title", |lua, me| lua.create_string(&me.title));
		fields.add_cached_field("prog", |lua, me| lua.to_value_with(&me.prog, SER_OPT));

		fields.add_field_method_get("running", |_, me| Ok(me.prog.running()));
		fields.add_field_method_get("cooked", |_, me| Ok(me.prog.cooked()));
		fields.add_field_method_get("success", |_, me| Ok(me.prog.success()));
		fields.add_field_method_get("failed", |_, me| Ok(me.prog.failed()));
		fields.add_field_method_get("percent", |_, me| Ok(me.prog.percent()));
	}
}
