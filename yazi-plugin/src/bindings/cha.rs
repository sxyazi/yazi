use std::time::UNIX_EPOCH;

use mlua::{AnyUserData, Lua, UserDataFields, UserDataMethods};

use super::Cast;

pub struct Cha;

impl Cha {
	pub fn register(lua: &Lua) -> mlua::Result<()> {
		lua.register_userdata_type::<yazi_shared::fs::Cha>(|reg| {
			reg.add_field_method_get("is_dir", |_, me| Ok(me.is_dir()));
			reg.add_field_method_get("is_hidden", |_, me| Ok(me.is_hidden()));
			reg.add_field_method_get("is_link", |_, me| Ok(me.is_link()));
			reg.add_field_method_get("is_orphan", |_, me| Ok(me.is_orphan()));
			reg.add_field_method_get("is_block", |_, me| Ok(me.is_block()));
			reg.add_field_method_get("is_char", |_, me| Ok(me.is_char()));
			reg.add_field_method_get("is_fifo", |_, me| Ok(me.is_fifo()));
			reg.add_field_method_get("is_sock", |_, me| Ok(me.is_sock()));
			reg.add_field_method_get("is_exec", |_, me| Ok(me.is_exec()));
			reg.add_field_method_get("is_sticky", |_, me| Ok(me.is_sticky()));

			#[cfg(unix)]
			{
				reg.add_field_method_get("uid", |_, me| Ok(me.uid));
				reg.add_field_method_get("gid", |_, me| Ok(me.gid));
			}

			reg.add_field_method_get("length", |_, me| Ok(me.len));
			reg.add_field_method_get("created", |_, me| {
				Ok(me.created.and_then(|t| t.duration_since(UNIX_EPOCH).map(|d| d.as_secs_f64()).ok()))
			});
			reg.add_field_method_get("modified", |_, me| {
				Ok(me.modified.and_then(|t| t.duration_since(UNIX_EPOCH).map(|d| d.as_secs_f64()).ok()))
			});
			reg.add_field_method_get("accessed", |_, me| {
				Ok(me.accessed.and_then(|t| t.duration_since(UNIX_EPOCH).map(|d| d.as_secs_f64()).ok()))
			});
			reg.add_method("permissions", |_, me, ()| {
				Ok(
					#[cfg(unix)]
					Some(yazi_shared::fs::permissions(me.permissions)),
					#[cfg(windows)]
					None::<String>,
				)
			});
		})?;

		Ok(())
	}
}

impl<T: Into<yazi_shared::fs::Cha>> Cast<T> for Cha {
	fn cast(lua: &Lua, data: T) -> mlua::Result<AnyUserData> { lua.create_any_userdata(data.into()) }
}
