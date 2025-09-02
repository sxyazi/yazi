use std::ops::Deref;

use mlua::{AnyUserData, LuaSerdeExt, UserData, UserDataFields, Value};
use yazi_binding::{cached_field, deprecate};
use yazi_plugin::runtime::SER_OPT;

use super::{Lives, PtrCell};

pub(super) struct Tasks {
	inner: PtrCell<yazi_core::tasks::Tasks>,

	v_summary:  Option<Value>,
	v_progress: Option<Value>,
}

impl Deref for Tasks {
	type Target = yazi_core::tasks::Tasks;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl Tasks {
	pub(super) fn make(inner: &yazi_core::tasks::Tasks) -> mlua::Result<AnyUserData> {
		Lives::scoped_userdata(Self { inner: inner.into(), v_summary: None, v_progress: None })
	}
}

impl UserData for Tasks {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		cached_field!(fields, summary, |lua, me| lua.to_value_with(&me.summary, SER_OPT));

		cached_field!(fields, progress, |lua, me| {
			deprecate!(
				lua,
				"`cx.tasks.progress` is deprecated, use `cx.tasks.summary` instead, in your {}"
			);
			lua.create_table_from([
				("total", me.summary.total),
				("succ", me.summary.success),
				("fail", me.summary.failed),
				("found", 0),
				("processed", 0),
			])
		});
	}
}
