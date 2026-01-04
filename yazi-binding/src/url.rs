use std::ops::Deref;

use mlua::{AnyUserData, ExternalError, ExternalResult, FromLua, IntoLua, Lua, MetaMethod, UserData, UserDataFields, UserDataMethods, UserDataRef, Value};
use yazi_fs::{FsHash64, FsHash128, FsUrl};
use yazi_shared::{path::{PathLike, StripPrefixError}, scheme::SchemeCow, strand::{StrandLike, ToStrand}, url::{AsUrl, UrlCow, UrlLike}};

use crate::{Path, Scheme, cached_field, deprecate};

pub type UrlRef = UserDataRef<Url>;

const EXPECTED: &str = "expected a string, Url, or Path";

pub struct Url {
	inner: yazi_shared::url::UrlBuf,

	v_path:   Option<Value>,
	v_name:   Option<Value>,
	v_stem:   Option<Value>,
	v_ext:    Option<Value>,
	v_urn:    Option<Value>,
	v_base:   Option<Value>,
	v_parent: Option<Value>,

	v_scheme: Option<Value>,
	v_domain: Option<Value>,

	v_cache: Option<Value>,
}

impl Deref for Url {
	type Target = yazi_shared::url::UrlBuf;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl AsUrl for Url {
	fn as_url(&self) -> yazi_shared::url::Url<'_> { self.inner.as_url() }
}

impl AsUrl for &Url {
	fn as_url(&self) -> yazi_shared::url::Url<'_> { self.inner.as_url() }
}

impl From<Url> for yazi_shared::url::UrlBuf {
	fn from(value: Url) -> Self { value.inner }
}

impl From<Url> for yazi_shared::url::UrlBufCov {
	fn from(value: Url) -> Self { Self(value.inner) }
}

impl From<Url> for yazi_shared::url::UrlCow<'_> {
	fn from(value: Url) -> Self { value.inner.into() }
}

impl TryFrom<&[u8]> for Url {
	type Error = mlua::Error;

	fn try_from(value: &[u8]) -> mlua::Result<Self> { Ok(Self::new(UrlCow::try_from(value)?)) }
}

impl Url {
	pub fn new(url: impl Into<yazi_shared::url::UrlBuf>) -> Self {
		Self {
			inner: url.into(),

			v_path:   None,
			v_name:   None,
			v_stem:   None,
			v_ext:    None,
			v_urn:    None,
			v_base:   None,
			v_parent: None,

			v_scheme: None,
			v_domain: None,

			v_cache: None,
		}
	}

	pub fn install(lua: &Lua) -> mlua::Result<()> {
		lua.globals().raw_set(
			"Url",
			lua.create_function(|_, value: Value| {
				Ok(match value {
					Value::String(s) => Self::try_from(&*s.as_bytes())?,
					Value::UserData(ud) => {
						if let Ok(url) = ud.borrow::<Self>() {
							Self::new(&url.inner)
						} else if let Ok(path) = ud.borrow::<Path>() {
							Self::new(path.as_os().into_lua_err()?)
						} else {
							Err(EXPECTED.into_lua_err())?
						}
					}
					_ => Err(EXPECTED.into_lua_err())?,
				})
			})?,
		)
	}

	fn ends_with(&self, child: Value) -> mlua::Result<bool> {
		match child {
			Value::String(s) => self.try_ends_with(UrlCow::try_from(&*s.as_bytes())?).into_lua_err(),
			Value::UserData(ud) => self.try_ends_with(&*ud.borrow::<Self>()?).into_lua_err(),
			_ => Err("must be a string or Url".into_lua_err())?,
		}
	}

	fn hash(&self, long: Option<bool>) -> mlua::Result<String> {
		Ok(if long.unwrap_or(false) {
			format!("{:x}", self.hash_u128())
		} else {
			format!("{:x}", self.hash_u64())
		})
	}

	fn join(&self, lua: &Lua, other: Value) -> mlua::Result<Value> {
		match other {
			Value::String(s) => {
				let b = s.as_bytes();
				let (scheme, path) = SchemeCow::parse(&b)?;
				if scheme == self.scheme() {
					Self::new(self.try_join(path).into_lua_err()?).into_lua(lua)
				} else {
					Self::new(UrlCow::try_from((scheme, path))?).into_lua(lua)
				}
			}
			Value::UserData(ref ud) => {
				let url = ud.borrow::<Self>()?;
				if url.scheme() == self.scheme() {
					Self::new(self.try_join(url.loc()).into_lua_err()?).into_lua(lua)
				} else {
					Ok(other)
				}
			}
			_ => Err("must be a string or Url".into_lua_err())?,
		}
	}

	fn starts_with(&self, base: Value) -> mlua::Result<bool> {
		match base {
			Value::String(s) => self.try_starts_with(UrlCow::try_from(&*s.as_bytes())?).into_lua_err(),
			Value::UserData(ud) => self.try_starts_with(&*ud.borrow::<Self>()?).into_lua_err(),
			_ => Err("must be a string or Url".into_lua_err())?,
		}
	}

	fn strip_prefix(&self, base: Value) -> mlua::Result<Option<Path>> {
		let strip = match base {
			Value::String(s) => self.try_strip_prefix(UrlCow::try_from(&*s.as_bytes())?),
			Value::UserData(ud) => self.try_strip_prefix(&*ud.borrow::<Self>()?),
			_ => Err("must be a string or Url".into_lua_err())?,
		};

		Ok(match strip {
			Ok(p) => Some(Path::new(p)),
			Err(StripPrefixError::Exotic | StripPrefixError::NotPrefix) => None,
			Err(e @ StripPrefixError::WrongEncoding) => Err(e.into_lua_err())?,
		})
	}
}

impl FromLua for Url {
	fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
		Ok(match value {
			Value::UserData(ud) => ud.take()?,
			_ => Err("Expected a Url".into_lua_err())?,
		})
	}
}

impl UserData for Url {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		cached_field!(fields, path, |_, me| Ok(Path::new(me.loc())));
		cached_field!(fields, name, |lua, me| {
			me.name().map(|s| lua.create_string(s.encoded_bytes())).transpose()
		});
		cached_field!(fields, stem, |lua, me| {
			me.stem().map(|s| lua.create_string(s.encoded_bytes())).transpose()
		});
		cached_field!(fields, ext, |lua, me| {
			me.ext().map(|s| lua.create_string(s.encoded_bytes())).transpose()
		});
		cached_field!(fields, urn, |_, me| Ok(Path::new(me.urn())));
		cached_field!(fields, base, |_, me| {
			Ok(Some(me.base()).filter(|u| !u.loc().is_empty()).map(Self::new))
		});
		cached_field!(fields, parent, |_, me| Ok(me.parent().map(Self::new)));

		cached_field!(fields, scheme, |_, me| Ok(Scheme::new(me.scheme())));
		cached_field!(fields, domain, |lua, me| {
			me.scheme().domain().map(|s| lua.create_string(s)).transpose()
		});

		cached_field!(fields, cache, |_, me| Ok(me.cache().map(Path::new)));

		fields.add_field_method_get("frag", |lua, me| {
			deprecate!(lua, "`frag` property of Url is deprecated and renamed to `domain`, please use the new name instead, in your {}");
			me.scheme().domain().map(|s| lua.create_string(s)).transpose()
		});

		fields.add_field_method_get("is_regular", |_, me| Ok(me.is_regular()));
		fields.add_field_method_get("is_search", |_, me| Ok(me.is_search()));
		fields.add_field_method_get("is_archive", |_, me| Ok(me.is_archive()));
		fields.add_field_method_get("is_absolute", |_, me| Ok(me.is_absolute()));
		fields.add_field_method_get("has_root", |_, me| Ok(me.has_root()));
	}

	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_method("ends_with", |_, me, child: Value| me.ends_with(child));
		methods.add_method("hash", |_, me, long: Option<bool>| me.hash(long));
		methods.add_method("join", |lua, me, other: Value| me.join(lua, other));
		methods.add_method("starts_with", |_, me, base: Value| me.starts_with(base));
		methods.add_method("strip_prefix", |_, me, base: Value| me.strip_prefix(base));

		methods.add_function_mut("into_search", |_, (ud, domain): (AnyUserData, mlua::String)| {
			let url = ud.take::<Self>()?.inner.into_search(domain.to_str()?).into_lua_err()?;
			Ok(Self::new(url))
		});

		methods.add_meta_method(MetaMethod::Eq, |_, me, other: UrlRef| Ok(me.inner == other.inner));
		methods.add_meta_method(MetaMethod::ToString, |lua, me, ()| {
			lua.create_string(me.to_strand().encoded_bytes())
		});
		methods.add_meta_method(MetaMethod::Concat, |lua, lhs, rhs: mlua::String| {
			lua.create_string([lhs.to_strand().encoded_bytes(), &rhs.as_bytes()].concat())
		});
	}
}
