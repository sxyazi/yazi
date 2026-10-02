use mlua::{ExternalError, Lua, Table, UserData, UserDataFields, UserDataMethods};

use crate::{FsHash128, stat::{Stat, StatKind, StatMode}};

impl Stat {
	pub fn install(lua: &Lua) -> mlua::Result<()> {
		lua.globals().raw_set(
			"Stat",
			lua.create_function(|_, t: Table| {
				let kind = StatKind::from_bits(t.raw_get("kind").unwrap_or_default())
					.ok_or_else(|| "Invalid kind".into_lua_err())?;

				let mode = StatMode::try_from(t.raw_get::<u16>("mode")?)?;

				Ok(Self {
					kind,
					mode,
					len: t.raw_get("len").unwrap_or_default(),
					atime: t.raw_get("atime")?,
					btime: t.raw_get("btime")?,
					ctime: t.raw_get("ctime")?,
					mtime: t.raw_get("mtime")?,
					dev: t.raw_get("dev").unwrap_or_default(),
					uid: t.raw_get("uid").unwrap_or_default(),
					gid: t.raw_get("gid").unwrap_or_default(),
					nlink: t.raw_get("nlink").unwrap_or_default(),
				})
			})?,
		)
	}
}

impl UserData for Stat {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("mode", |_, me| Ok(me.mode.bits()));
		fields.add_field_method_get("is_dir", |_, me| Ok(me.is_dir()));
		fields.add_field_method_get("is_hidden", |_, me| Ok(me.is_hidden()));
		fields.add_field_method_get("is_link", |_, me| Ok(me.is_link()));
		fields.add_field_method_get("is_orphan", |_, me| Ok(me.is_orphan()));
		fields.add_field_method_get("is_dummy", |_, me| Ok(me.is_dummy()));
		fields.add_field_method_get("is_indirect", |_, me| Ok(me.is_indirect()));
		fields.add_field_method_get("is_block", |_, me| Ok(me.is_block()));
		fields.add_field_method_get("is_char", |_, me| Ok(me.is_char()));
		fields.add_field_method_get("is_fifo", |_, me| Ok(me.is_fifo()));
		fields.add_field_method_get("is_sock", |_, me| Ok(me.is_sock()));
		fields.add_field_method_get("is_exec", |_, me| Ok(me.is_exec()));
		fields.add_field_method_get("is_sticky", |_, me| Ok(me.is_sticky()));

		fields.add_field_method_get("len", |_, me| Ok(me.len));
		fields.add_field_method_get("atime", |_, me| Ok(me.atime));
		fields.add_field_method_get("btime", |_, me| Ok(me.btime));
		fields.add_field_method_get("ctime", |_, me| Ok(me.ctime));
		fields.add_field_method_get("mtime", |_, me| Ok(me.mtime));
		fields.add_field_method_get("dev", |_, me| Ok(me.dev));
		fields.add_field_method_get("uid", |_, me| Ok(me.uid));
		fields.add_field_method_get("gid", |_, me| Ok(me.gid));
		fields.add_field_method_get("nlink", |_, me| Ok(me.nlink));
	}

	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_method("hash", |lua, me, long: bool| {
			Ok(if long {
				lua.create_string(me.hash_u128_str(&mut [0; 26]))
			} else {
				Err("Short hash not supported".into_lua_err())?
			})
		});
		methods.add_method("perm", |_lua, _me, ()| {
			Ok(
				#[cfg(unix)]
				_lua.create_string(_me.mode.permissions(_me.is_dummy())),
				#[cfg(windows)]
				Ok::<_, mlua::Error>(mlua::Value::Nil),
			)
		});
	}
}
