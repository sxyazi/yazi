use mlua::{AnyUserData, UserData};
use scopeguard::defer;
use yazi_macro::log_if_err;
use yazi_plugin::LUA;

use super::{Ctx, MutCell, TABS};

static TO_DESTROY: MutCell<Vec<AnyUserData>> = MutCell::new(Vec::new());

pub struct Lives;

impl Lives {
	pub fn scope<T, F>(cx: &mut yazi_core::Ctx, f: F) -> mlua::Result<T>
	where
		F: FnOnce(&mut yazi_core::Ctx) -> mlua::Result<T>,
	{
		defer! {
			unsafe {
				for ud in (*TO_DESTROY.get()).drain(..) {
					ud.destroy().expect("failed to destruct scoped userdata");
				}
				for tab in (*TABS.get()).assume_init_mut() {
					tab.clear();
				}
			}
		}

		let ud = Ctx::make(cx)?;
		LUA.globals().raw_set("cx", ud.clone())?;
		LUA.set_named_registry_value("cx", ud)?;
		let result = f(cx);

		log_if_err!("Scoped Lua execution", &result);
		result
	}

	pub(crate) fn scoped_userdata<T>(data: T) -> mlua::Result<AnyUserData>
	where
		T: UserData + 'static,
	{
		let ud = LUA.create_userdata(data)?;
		unsafe { &mut *TO_DESTROY.get() }.push(ud.clone());
		Ok(ud)
	}
}
