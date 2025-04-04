use std::cell::RefCell;

use mlua::{AnyUserData, UserData};
use tracing::error;
use yazi_plugin::LUA;
use yazi_shared::RoCell;

use crate::Ctx;

static TO_DESTROY: RoCell<RefCell<Vec<AnyUserData>>> = RoCell::new_const(RefCell::new(Vec::new()));

pub(crate) struct Lives;

impl Lives {
	pub(crate) fn scope<T>(cx: &Ctx, f: impl FnOnce() -> mlua::Result<T>) -> mlua::Result<T> {
		let result = LUA.scope(|scope| {
			scope.add_destructor(|| {
				for ud in TO_DESTROY.borrow_mut().drain(..) {
					ud.destroy().expect("failed to destruct scoped userdata");
				}
			});

			LUA.set_named_registry_value("cx", scope.create_any_userdata_ref(cx)?)?;
			LUA.globals().raw_set("cx", super::Ctx::make(cx)?)?;
			f()
		});

		if let Err(ref e) = result {
			error!("{e}");
		}
		result
	}

	#[inline]
	pub(crate) fn scoped_userdata<T>(data: T) -> mlua::Result<AnyUserData>
	where
		T: UserData + 'static,
	{
		let ud = LUA.create_userdata(data)?;
		TO_DESTROY.borrow_mut().push(ud.clone());
		Ok(ud)
	}
}
