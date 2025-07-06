use std::{ops::Deref, time::{Duration, SystemTime, UNIX_EPOCH}};

use mlua::{ExternalError, FromLua, IntoLua, Lua, Table, UserData, UserDataFields, UserDataMethods};
use yazi_fs::cha::ChaKind;

#[derive(Clone, Copy, FromLua)]
pub struct Cha(pub yazi_fs::cha::Cha);

impl Deref for Cha {
	type Target = yazi_fs::cha::Cha;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl Cha {
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

				Self(yazi_fs::cha::Cha {
					kind,
					len: t.raw_get("len").unwrap_or_default(),
					atime: parse_time(t.raw_get("atime").ok())?,
					btime: parse_time(t.raw_get("btime").ok())?,
					#[cfg(unix)]
					ctime: parse_time(t.raw_get("ctime").ok())?,
					mtime: parse_time(t.raw_get("mtime").ok())?,
					#[cfg(unix)]
					mode: t.raw_get("mode").unwrap_or_default(),
					#[cfg(unix)]
					dev: t.raw_get("dev").unwrap_or_default(),
					#[cfg(unix)]
					uid: t.raw_get("uid").unwrap_or_default(),
					#[cfg(unix)]
					gid: t.raw_get("gid").unwrap_or_default(),
					#[cfg(unix)]
					nlink: t.raw_get("nlink").unwrap_or_default(),
				})
				.into_lua(lua)
			})?,
		)
	}
}

impl UserData for Cha {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
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

		#[cfg(unix)]
		{
			use std::ops::Not;
			fields.add_field_method_get("mode", |_, me| Ok(me.is_dummy().not().then_some(me.mode)));
			fields.add_field_method_get("dev", |_, me| Ok(me.is_dummy().not().then_some(me.dev)));
			fields.add_field_method_get("uid", |_, me| Ok(me.is_dummy().not().then_some(me.uid)));
			fields.add_field_method_get("gid", |_, me| Ok(me.is_dummy().not().then_some(me.gid)));
			fields.add_field_method_get("nlink", |_, me| Ok(me.is_dummy().not().then_some(me.nlink)));
		}

		fields.add_field_method_get("len", |_, me| Ok(me.len));
		fields.add_field_method_get("atime", |_, me| {
			Ok(me.atime.and_then(|t| t.duration_since(UNIX_EPOCH).map(|d| d.as_secs_f64()).ok()))
		});
		fields.add_field_method_get("btime", |_, me| {
			Ok(me.btime.and_then(|t| t.duration_since(UNIX_EPOCH).map(|d| d.as_secs_f64()).ok()))
		});
		#[cfg(unix)]
		fields.add_field_method_get("ctime", |_, me| {
			Ok(me.ctime.and_then(|t| t.duration_since(UNIX_EPOCH).map(|d| d.as_secs_f64()).ok()))
		});
		fields.add_field_method_get("mtime", |_, me| {
			Ok(me.mtime.and_then(|t| t.duration_since(UNIX_EPOCH).map(|d| d.as_secs_f64()).ok()))
		});
	}

	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_method("perm", |_, _me, ()| {
			Ok(
				#[cfg(unix)]
				Some(yazi_fs::permissions(_me.mode, _me.is_dummy())),
				#[cfg(windows)]
				None::<String>,
			)
		});
	}
}
