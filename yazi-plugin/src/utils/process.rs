use mlua::{Function, Lua};

use super::Utils;

impl Utils {
	#[cfg(target_os = "macos")]
	pub(super) fn proc_info(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|lua, pid: usize| {
			let info = unsafe {
				let mut info: libc::proc_taskinfo = std::mem::zeroed();
				libc::proc_pidinfo(
					pid as _,
					libc::PROC_PIDTASKINFO,
					0,
					&mut info as *mut _ as *mut _,
					std::mem::size_of_val(&info) as _,
				);
				info
			};

			lua.create_table_from([("mem_resident", info.pti_resident_size)])
		})
	}

	#[cfg(not(target_os = "macos"))]
	pub(super) fn proc_info(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|lua, ()| lua.create_table())
	}
}
