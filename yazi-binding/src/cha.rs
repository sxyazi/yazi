use std::{ops::Deref, time::{Duration, SystemTime}};

use mlua::{ExternalError, FromLua, IntoLua, Lua, Table, UserData, UserDataFields, UserDataMethods};
use yazi_fs::{FsHash128, cha::{ChaKind, ChaMode}};

#[derive(Clone, Copy, FromLua)]
pub struct Cha(pub yazi_fs::cha::Cha);

impl Deref for Cha {
	type Target = yazi_fs::cha::Cha;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl Cha {
	pub fn install(lua: &Lua) -> mlua::Result<()> {
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
				let kind = ChaKind::from_bits(t.raw_get("kind").unwrap_or_default())
					.ok_or_else(|| "Invalid kind".into_lua_err())?;

				let mode = ChaMode::try_from(t.raw_get::<u16>("mode")?)?;

				Self(yazi_fs::cha::Cha {
					kind,
					mode,
					len: t.raw_get("len").unwrap_or_default(),
					atime: parse_time(t.raw_get("atime").ok())?,
					btime: parse_time(t.raw_get("btime").ok())?,
					ctime: parse_time(t.raw_get("ctime").ok())?,
					mtime: parse_time(t.raw_get("mtime").ok())?,
					dev: t.raw_get("dev").unwrap_or_default(),
					uid: t.raw_get("uid").unwrap_or_default(),
					gid: t.raw_get("gid").unwrap_or_default(),
					nlink: t.raw_get("nlink").unwrap_or_default(),
				})
				.into_lua(lua)
			})?,
		)
	}
}

impl UserData for Cha {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("mode", |_, me| Ok(me.mode.bits()));
		fields.add_field_method_get("is_dir", |_, me| Ok(me.is_dir()));
		fields.add_field_method_get("is_hidden", |_, me| Ok(me.is_hidden()));
		fields.add_field_method_get("is_link", |_, me| Ok(me.is_link()));
		fields.add_field_method_get("is_orphan", |_, me| Ok(me.is_orphan()));
		fields.add_field_method_get("is_dummy", |_, me| Ok(me.is_dummy()));
		fields.add_field_method_get("is_block", |_, me| Ok(me.is_block()));
		fields.add_field_method_get("is_char", |_, me| Ok(me.is_char()));
		fields.add_field_method_get("is_fifo", |_, me| Ok(me.is_fifo()));
		fields.add_field_method_get("is_sock", |_, me| Ok(me.is_sock()));
		fields.add_field_method_get("is_exec", |_, me| Ok(me.is_exec()));
		fields.add_field_method_get("is_sticky", |_, me| Ok(me.is_sticky()));

		fields.add_field_method_get("len", |_, me| Ok(me.len));
		fields.add_field_method_get("atime", |_, me| Ok(me.atime_dur().ok().map(|d| d.as_secs_f64())));
		fields.add_field_method_get("btime", |_, me| Ok(me.btime_dur().ok().map(|d| d.as_secs_f64())));
		fields.add_field_method_get("ctime", |_, me| Ok(me.ctime_dur().ok().map(|d| d.as_secs_f64())));
		fields.add_field_method_get("mtime", |_, me| Ok(me.mtime_dur().ok().map(|d| d.as_secs_f64())));
		fields.add_field_method_get("dev", |_, me| Ok(me.dev));
		fields.add_field_method_get("uid", |_, me| Ok(me.uid));
		fields.add_field_method_get("gid", |_, me| Ok(me.gid));
		fields.add_field_method_get("nlink", |_, me| Ok(me.nlink));
	}

	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_method("hash", |_, me, long: Option<bool>| {
			Ok(if long.unwrap_or(false) {
				format!("{:x}", me.hash_u128())
			} else {
				Err("Short hash not supported".into_lua_err())?
			})
		});
		methods.add_method("perm", |lua, _me, ()| {
			Ok(
				#[cfg(unix)]
				lua.create_string(_me.mode.permissions(_me.is_dummy())),
				#[cfg(windows)]
				Ok::<_, mlua::Error>(mlua::Value::Nil),
			)
		});
	}
}
