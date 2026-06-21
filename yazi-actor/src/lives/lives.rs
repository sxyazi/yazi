use std::mem::MaybeUninit;

use hashbrown::HashMap;
use mlua::{AnyUserData, UserData};
use scopeguard::defer;
use tracing::error;
use yazi_plugin::LUA;

use super::{Core, PtrCell};
use crate::lives::MutCell;

pub(super) static TO_DESTROY: MutCell<Vec<AnyUserData>> = MutCell::new(Vec::new());
pub(super) static FILE_CACHE: MutCell<
	MaybeUninit<HashMap<PtrCell<yazi_fs::file::File>, AnyUserData>>,
> = MutCell::new(MaybeUninit::uninit());

pub struct Lives;

impl Lives {
	pub fn scope<T, F>(core: &mut yazi_core::Core, f: F) -> mlua::Result<T>
	where
		F: FnOnce(&mut yazi_core::Core) -> mlua::Result<T>,
	{
		defer! {
			unsafe {
				(*FILE_CACHE.get()).assume_init_mut().clear();
				for ud in (*TO_DESTROY.get()).drain(..) {
					ud.destroy().expect("failed to destruct scoped userdata");
				}
			}
		}

		LUA.set_named_registry_value("cx", Core::make(core)?)?;
		LUA.globals().raw_set("cx", Core::make(core)?)?;
		let result = f(core);

		if let Err(ref e) = result {
			error!("{e}");
		}
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
