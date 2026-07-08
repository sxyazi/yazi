use mlua::{AnyUserData, ExternalError, ExternalResult, Lua, LuaString, MetaMethod, UserData, UserDataFields, UserDataMethods, UserDataRef, Value};
use yazi_shim::mlua::UserDataFieldsExt;

use crate::{LOG_LEVEL, path::{PathBufDyn, PathLike, StripPrefixError}, strand::{AsStrand, StrandCow}};

pub type PathRef = UserDataRef<PathBufDyn>;

impl PathBufDyn {
	pub fn install(lua: &Lua) -> mlua::Result<()> {
		lua.globals().raw_set(
			"Path",
			lua.create_table_from([(
				"os",
				lua.create_function(|_, s: LuaString| {
					Ok(Self::from(s.as_bytes().as_strand().as_os_path().into_lua_err()?))
				})?,
			)])?,
		)
	}

	fn ends_with(&self, child: Value) -> mlua::Result<bool> {
		match child {
			Value::String(s) => {
				self.try_ends_with(StrandCow::with(self.kind(), &*s.as_bytes())?).into_lua_err()
			}
			Value::UserData(ud) => self.try_ends_with(&*ud.borrow::<Self>()?).into_lua_err(),
			_ => Err("must be a string or Path".into_lua_err())?,
		}
	}

	fn join(&self, other: Value) -> mlua::Result<Self> {
		Ok(match other {
			Value::String(s) => {
				self.try_join(StrandCow::with(self.kind(), &*s.as_bytes())?).into_lua_err()?
			}
			Value::UserData(ref ud) => {
				let path = ud.borrow::<Self>()?;
				self.try_join(&*path).into_lua_err()?
			}
			_ => Err("must be a string or Path".into_lua_err())?,
		})
	}

	fn starts_with(&self, base: Value) -> mlua::Result<bool> {
		match base {
			Value::String(s) => {
				self.try_starts_with(StrandCow::with(self.kind(), &*s.as_bytes())?).into_lua_err()
			}
			Value::UserData(ud) => self.try_starts_with(&*ud.borrow::<Self>()?).into_lua_err(),
			_ => Err("must be a string or Path".into_lua_err())?,
		}
	}

	fn strip_prefix(&self, base: Value) -> mlua::Result<Option<Self>> {
		let strip = match base {
			Value::String(s) => self.try_strip_prefix(StrandCow::with(self.kind(), &*s.as_bytes())?),
			Value::UserData(ud) => self.try_strip_prefix(&*ud.borrow::<Self>()?),
			_ => Err("must be a string or Path".into_lua_err())?,
		};

		Ok(match strip {
			Ok(p) => Some(p.to_owned()),
			Err(StripPrefixError::Exotic | StripPrefixError::NotPrefix) => None,
			Err(e @ StripPrefixError::WrongEncoding) => Err(e.into_lua_err())?,
		})
	}
}

impl UserData for PathBufDyn {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_cached_field("ext", |lua, me| {
			me.ext().map(|s| lua.create_string(s.encoded_bytes())).transpose()
		});
		fields.add_cached_field("name", |lua, me| {
			me.name().map(|s| lua.create_string(s.encoded_bytes())).transpose()
		});
		fields.add_cached_field("parent", |_, me| Ok(me.parent().map(|p| p.to_owned())));
		fields.add_cached_field("stem", |lua, me| {
			me.stem().map(|s| lua.create_string(s.encoded_bytes())).transpose()
		});

		fields.add_field_method_get("is_absolute", |_, me| Ok(me.is_absolute()));
		fields.add_field_method_get("has_root", |_, me| Ok(me.has_root()));
	}

	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_method("ends_with", |_, me, child: Value| me.ends_with(child));
		methods.add_method("join", |_, me, other: Value| me.join(other));
		methods.add_method("starts_with", |_, me, base: Value| me.starts_with(base));
		methods.add_method("strip_prefix", |_, me, base: Value| me.strip_prefix(base));

		methods.add_meta_method(MetaMethod::Concat, |lua, lhs, rhs: LuaString| {
			lua.create_external_string([lhs.encoded_bytes(), &rhs.as_bytes()].concat())
		});
		methods.add_meta_method(MetaMethod::Eq, |_, me, other: PathRef| Ok(*me == *other));
		methods
			.add_meta_method(MetaMethod::ToString, |lua, me, ()| lua.create_string(me.encoded_bytes()));

		if !LOG_LEVEL.get().is_none() {
			methods.add_meta_function(MetaMethod::ToDebugString, |_, ud: AnyUserData| {
				Ok(format!("Path({:?}): {:?}", ud.to_pointer(), *ud.borrow::<Self>()?))
			});
		}
	}
}
