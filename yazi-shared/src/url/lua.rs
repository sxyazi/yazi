use mlua::{AnyUserData, ExternalError, ExternalResult, IntoLua, Lua, LuaString, MetaMethod, UserData, UserDataFields, UserDataMethods, UserDataRef, UserDataRegistry, Value};
use yazi_shim::mlua::UserDataFieldsExt;

use crate::{LOG_LEVEL, path::{PathBufDyn, PathLike, StripPrefixError}, scheme::{SchemeCow, SchemeLike}, strand::{StrandLike, ToStrand}, url::{UrlBuf, UrlBufInventory, UrlCow, UrlLike}};

pub type UrlRef = UserDataRef<UrlBuf>;

const EXPECTED: &str = "expected a string, Url, or Path";

impl UrlBuf {
	pub fn install(lua: &Lua) -> mlua::Result<()> {
		lua.globals().raw_set(
			"Url",
			lua.create_function(|_, value: Value| {
				Ok(match value {
					Value::String(s) => UrlCow::try_from(&*s.as_bytes())?.into(),
					Value::UserData(ud) => {
						if let Ok(url) = ud.borrow::<Self>() {
							url.clone()
						} else if let Ok(path) = ud.borrow::<PathBufDyn>() {
							path.as_os().into_lua_err()?.into()
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

	fn join(&self, lua: &Lua, other: Value) -> mlua::Result<Value> {
		match other {
			Value::String(s) => {
				let b = s.as_bytes();
				let (scheme, path) = SchemeCow::parse(&b)?;
				if scheme.covariant(self.scheme()) {
					self.try_join(path).into_lua_err()?.into_lua(lua)
				} else {
					UrlCow::try_from((scheme, path))?.into_owned().into_lua(lua)
				}
			}
			Value::UserData(ref ud) => {
				if let Ok(url) = ud.borrow::<Self>() {
					if url.scheme().covariant(self.scheme()) {
						self.try_join(url.loc()).into_lua_err()?.into_lua(lua)
					} else {
						Ok(other)
					}
				} else if let Ok(path) = ud.borrow::<PathBufDyn>() {
					self.try_join(&*path).into_lua_err()?.into_lua(lua)
				} else {
					Err(EXPECTED.into_lua_err())?
				}
			}
			_ => Err(EXPECTED.into_lua_err())?,
		}
	}

	fn starts_with(&self, base: Value) -> mlua::Result<bool> {
		match base {
			Value::String(s) => self.try_starts_with(UrlCow::try_from(&*s.as_bytes())?).into_lua_err(),
			Value::UserData(ud) => self.try_starts_with(&*ud.borrow::<Self>()?).into_lua_err(),
			_ => Err("must be a string or Url".into_lua_err())?,
		}
	}

	fn strip_prefix(&self, base: Value) -> mlua::Result<Option<PathBufDyn>> {
		let strip = match base {
			Value::String(s) => self.try_strip_prefix(UrlCow::try_from(&*s.as_bytes())?),
			Value::UserData(ud) => self.try_strip_prefix(&*ud.borrow::<Self>()?),
			_ => Err("must be a string or Url".into_lua_err())?,
		};

		Ok(match strip {
			Ok(p) => Some(p.to_owned()),
			Err(StripPrefixError::Exotic | StripPrefixError::NotPrefix) => None,
			Err(e @ StripPrefixError::WrongEncoding) => Err(e.into_lua_err())?,
		})
	}
}

impl UserData for UrlBuf {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_cached_field("path", |_, me| Ok(me.loc().to_owned()));
		fields.add_cached_field("name", |lua, me| {
			me.name().map(|s| lua.create_string(s.encoded_bytes())).transpose()
		});
		fields.add_cached_field("stem", |lua, me| {
			me.stem().map(|s| lua.create_string(s.encoded_bytes())).transpose()
		});
		fields.add_cached_field("ext", |lua, me| {
			me.ext().map(|s| lua.create_string(s.encoded_bytes())).transpose()
		});
		fields.add_cached_field("urn", |_, me| Ok(me.urn().to_owned()));
		fields.add_cached_field("base", |_, me| {
			Ok(Some(me.base()).filter(|u| !u.loc().is_empty()).map(Self::from))
		});
		fields.add_cached_field("parent", |_, me| Ok(me.parent().map(Self::from)));

		fields.add_cached_field("scheme", |_, me| Ok(me.scheme().to_owned()));
		fields.add_cached_field("domain", |lua, me| {
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
		methods.add_method("join", |lua, me, other: Value| me.join(lua, other));
		methods.add_method("starts_with", |_, me, base: Value| me.starts_with(base));
		methods.add_method("strip_prefix", |_, me, base: Value| me.strip_prefix(base));

		methods.add_method_once("into_search", |_, me, domain: LuaString| {
			me.into_search(domain.to_str()?).into_lua_err()
		});

		methods.add_meta_method(MetaMethod::Eq, |_, me, other: UrlRef| Ok(*me == *other));
		methods.add_meta_method(MetaMethod::ToString, |lua, me, ()| {
			lua.create_string(me.to_strand().encoded_bytes())
		});
		methods.add_meta_method(MetaMethod::Concat, |lua, lhs, rhs: LuaString| {
			lua.create_external_string([lhs.to_strand().encoded_bytes(), &rhs.as_bytes()].concat())
		});

		if !LOG_LEVEL.get().is_none() {
			methods.add_meta_function(MetaMethod::ToDebugString, |_, ud: AnyUserData| {
				Ok(format!("Url({:?}): {:?}", ud.to_pointer(), *ud.borrow::<Self>()?))
			});
		}
	}

	fn register(registry: &mut UserDataRegistry<Self>) {
		Self::add_fields(registry);
		Self::add_methods(registry);

		for inv in inventory::iter::<UrlBufInventory>() {
			(inv.register)(registry);
		}
	}
}
