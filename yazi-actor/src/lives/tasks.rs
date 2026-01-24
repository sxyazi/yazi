use std::ops::Deref;

use mlua::{AnyUserData, LuaSerdeExt, UserData, UserDataFields, Value};
use yazi_binding::{SER_OPT, cached_field};

use super::{Lives, PtrCell};
use crate::lives::TaskSnap;

pub(super) struct Tasks {
	inner: PtrCell<yazi_core::tasks::Tasks>,

	v_snaps:   Option<Value>,
	v_summary: Option<Value>,
}

impl Deref for Tasks {
	type Target = yazi_core::tasks::Tasks;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl Tasks {
	pub(super) fn make(inner: &yazi_core::tasks::Tasks) -> mlua::Result<AnyUserData> {
		Lives::scoped_userdata(Self {
			inner: inner.into(),

			v_snaps:   None,
			v_summary: None,
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
	}
}
