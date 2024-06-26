use std::time::{Duration, SystemTime, UNIX_EPOCH};

use mlua::{AnyUserData, ExternalError, Lua, Table, UserDataFields, UserDataMethods, UserDataRef};
use yazi_shared::fs::ChaKind;

use crate::bindings::Cast;

pub type UrlRef<'lua> = UserDataRef<'lua, yazi_shared::fs::Cha>;

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

	pub fn install(lua: &Lua) -> mlua::Result<()> {
		#[inline]
		fn parse_time(f: Option<f64>) -> mlua::Result<Option<SystemTime>> {
			Ok(match f {
				Some(n) if n >= 0.0 => Some(SystemTime::UNIX_EPOCH + Duration::from_secs_f64(n)),
				Some(n) => Err(format!("Invalid timestamp: {n}").into_lua_err())?,
				None => None,
			})
		}

		lua.globals().raw_set(
			"Cha",
			lua.create_function(|lua, t: Table| {
				let kind =
					ChaKind::from_bits(t.raw_get("kind")?).ok_or_else(|| "Invalid kind".into_lua_err())?;

				Self::cast(lua, yazi_shared::fs::Cha {
					kind,
					len: t.raw_get("len").unwrap_or_default(),
					accessed: parse_time(t.raw_get("atime").ok())?,
					created: parse_time(t.raw_get("ctime").ok())?,
					modified: parse_time(t.raw_get("mtime").ok())?,
					#[cfg(unix)]
					permissions: t.raw_get("permissions").unwrap_or_default(),
					#[cfg(unix)]
					uid: t.raw_get("uid").unwrap_or_default(),
					#[cfg(unix)]
					gid: t.raw_get("gid").unwrap_or_default(),
				})
			})?,
		)
	}
}

impl<T: Into<yazi_shared::fs::Cha>> Cast<T> for Cha {
	fn cast(lua: &Lua, data: T) -> mlua::Result<AnyUserData> { lua.create_any_userdata(data.into()) }
}
