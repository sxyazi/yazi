use std::ops::Deref;

use mlua::{AnyUserData, LuaSerdeExt, UserData, UserDataFields, Value};
use yazi_binding::{SER_OPT, cached_field, deprecate};

use super::{Lives, PtrCell};
use crate::lives::TaskSnap;

pub(super) struct Tasks {
	inner: PtrCell<yazi_core::tasks::Tasks>,

	v_snaps:    Option<Value>,
	v_summary:  Option<Value>,
	v_progress: Option<Value>,
}

impl Deref for Tasks {
	type Target = yazi_core::tasks::Tasks;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl Tasks {
	pub(super) fn make(inner: &yazi_core::tasks::Tasks) -> mlua::Result<AnyUserData> {
		Lives::scoped_userdata(Self {
			inner: inner.into(),

			v_snaps:    None,
			v_summary:  None,
			v_progress: None,
		})
	}
}

impl UserData for Tasks {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("cursor", |_, me| Ok(me.cursor));

		cached_field!(fields, snaps, |lua, me| {
			let tbl = lua.create_table_with_capacity(me.snaps.len(), 0)?;
			for snap in &me.snaps {
				tbl.raw_push(TaskSnap::make(snap)?)?;
			}
			Ok(tbl)
		});

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
