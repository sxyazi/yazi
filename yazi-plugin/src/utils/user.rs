#[cfg(unix)]
use mlua::{Function, Lua};
#[cfg(unix)]
use yazi_shared::strand::IntoStrand;

use super::Utils;

impl Utils {
	#[cfg(unix)]
	pub(super) fn uid(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|_, ()| Ok(yazi_shim::Uzers::uid()))
	}

	#[cfg(unix)]
	pub(super) fn gid(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|_, ()| Ok(yazi_shim::Uzers::gid()))
	}

	#[cfg(unix)]
	pub(super) fn user_name(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|_, uid: Option<u32>| {
			Ok(yazi_shim::Uzers::user_name(uid).map(|s| s.into_strand()))
		})
	}

	#[cfg(unix)]
	pub(super) fn group_name(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|_, gid: Option<u32>| {
			Ok(yazi_shim::Uzers::group_name(gid).map(|s| s.into_strand()))
		})
	}

	#[cfg(unix)]
	pub(super) fn host_name(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|lua, ()| yazi_shared::hostname().map(|s| lua.create_string(s)).transpose())
	}
}
