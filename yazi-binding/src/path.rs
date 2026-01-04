use std::ops::Deref;

use mlua::{ExternalError, ExternalResult, FromLua, Lua, MetaMethod, UserData, UserDataFields, UserDataMethods, UserDataRef, Value};
use yazi_shared::{path::{PathBufDyn, PathLike, StripPrefixError}, strand::{AsStrand, Strand, StrandCow}};

use crate::cached_field;

pub type PathRef = UserDataRef<Path>;

pub struct Path {
	inner: PathBufDyn,

	v_ext:    Option<Value>,
	v_name:   Option<Value>,
	v_parent: Option<Value>,
	v_stem:   Option<Value>,
}

impl Deref for Path {
	type Target = PathBufDyn;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl From<Path> for PathBufDyn {
	fn from(value: Path) -> Self { value.inner }
}

impl AsStrand for Path {
	fn as_strand(&self) -> Strand<'_> { self.inner.as_strand() }
}

impl AsStrand for &Path {
	fn as_strand(&self) -> Strand<'_> { self.inner.as_strand() }
}

impl Path {
	pub fn new(path: impl Into<PathBufDyn>) -> Self {
		Self {
			inner: path.into(),

			v_ext:    None,
			v_name:   None,
			v_parent: None,
			v_stem:   None,
		}
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
		Ok(Self::new(match other {
			Value::String(s) => {
				self.try_join(StrandCow::with(self.kind(), &*s.as_bytes())?).into_lua_err()?
			}
			Value::UserData(ref ud) => {
				let path = ud.borrow::<Self>()?;
				self.try_join(&*path).into_lua_err()?
			}
			_ => Err("must be a string or Path".into_lua_err())?,
		}))
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
			Ok(p) => Some(Self::new(p)),
			Err(StripPrefixError::Exotic | StripPrefixError::NotPrefix) => None,
			Err(e @ StripPrefixError::WrongEncoding) => Err(e.into_lua_err())?,
		})
	}
}

impl FromLua for Path {
	fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
		Ok(match value {
			Value::UserData(ud) => ud.take()?,
			_ => Err("Expected a Path".into_lua_err())?,
		})
	}
}

impl UserData for Path {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		cached_field!(fields, ext, |lua, me| {
			me.ext().map(|s| lua.create_string(s.encoded_bytes())).transpose()
		});
		cached_field!(fields, name, |lua, me| {
			me.name().map(|s| lua.create_string(s.encoded_bytes())).transpose()
		});
		cached_field!(fields, parent, |_, me| Ok(me.parent().map(Self::new)));
		cached_field!(fields, stem, |lua, me| {
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

		methods.add_meta_method(MetaMethod::Concat, |lua, lhs, rhs: mlua::String| {
			lua.create_string([lhs.encoded_bytes(), &rhs.as_bytes()].concat())
		});
		methods.add_meta_method(MetaMethod::Eq, |_, me, other: PathRef| Ok(me.inner == other.inner));
		methods
			.add_meta_method(MetaMethod::ToString, |lua, me, ()| lua.create_string(me.encoded_bytes()));
	}
}
