use mlua::{AnyUserData, BorrowedBytes, ExternalError, FromLua, IntoLua, Lua, LuaString, MetaMethod, Table, UserData, UserDataFields, UserDataMethods, UserDataRef, UserDataRegistry, Value};
use yazi_shim::{log::LOG_LEVEL, mlua::UserDataFieldsExt};

use crate::{auth::{AuthArc, Scheme, View}, domain::Domain, path::{PathBufDyn, StripPrefixError}, spec::Spec, strand::{StrandCow, StrandLike, ToStrand}, url::{AsUrl, UrlBuf, UrlBufInventory, UrlCow, UrlLike}};

pub type UrlRef = UserDataRef<UrlBuf>;

impl TryFrom<Value> for UrlBuf {
	type Error = mlua::Error;

	fn try_from(value: Value) -> Result<Self, Self::Error> {
		match value {
			Value::String(s) => Ok(UrlCow::try_from(&*s.as_bytes())?.into()),
			Value::UserData(ud) => ud.try_into(),
			_ => Err("expected a string, Path, or Url".into_lua_err()),
		}
	}
}

impl TryFrom<Table> for UrlBuf {
	type Error = mlua::Error;

	fn try_from(t: Table) -> Result<Self, Self::Error> {
		let url: Self = t.raw_get::<Value>(1)?.try_into()?;
		let scheme: Scheme = t.raw_get("scheme")?;
		let domain: Domain<'static> = t.raw_get("domain")?;

		let mut auth = AuthArc::get(&scheme, &domain)?;
		if auth.kind.is_view() {
			auth = auth.with_view(View { source: url.auth().clone(), data: t.raw_get("data")? });
		}

		let (uri, urn) = if auth.kind.is_view() { (0, 0) } else { Spec::retrieve_ports(url.as_url()) };
		Ok(UrlBuf::try_from((Spec { auth, uri, urn }, url.into_loc()))?)
	}
}

impl TryFrom<AnyUserData> for UrlBuf {
	type Error = mlua::Error;

	fn try_from(ud: AnyUserData) -> Result<Self, Self::Error> {
		if let Ok(url) = ud.take::<Self>() {
			Ok(url)
		} else if let Ok(path) = ud.take::<PathBufDyn>() {
			Ok(path.into_os()?.into())
		} else {
			Err("expected a Path or Url".into_lua_err())
		}
	}
}

impl UrlBuf {
	pub fn install(lua: &Lua) -> mlua::Result<()> {
		lua.globals().raw_set("Url", lua.create_function(|_, value: Self| Ok(value))?)
	}

	fn ends_with(&self, child: Value) -> mlua::Result<bool> {
		Ok(match child {
			Value::String(s) => self.try_ends_with(UrlCow::try_from(&*s.as_bytes())?)?,
			Value::UserData(ud) => self.try_ends_with(&*ud.borrow::<Self>()?)?,
			_ => Err("must be a string or Url".into_lua_err())?,
		})
	}

	fn join(&self, lua: &Lua, other: Value) -> mlua::Result<Value> {
		match other {
			Value::String(s) => {
				let bytes = s.as_bytes();
				self.try_join(StrandCow::with(self.loc().kind(), &*bytes)?)?.into_lua(lua)
			}
			Value::UserData(ud) if let Ok(path) = ud.borrow::<PathBufDyn>() => {
				self.try_join(&*path)?.into_lua(lua)
			}
			Value::UserData(ref ud) if ud.is::<Self>() => self.resolve(lua, other),
			_ => Err("expected a string or Path".into_lua_err()),
		}
	}

	fn resolve(&self, lua: &Lua, other: Value) -> mlua::Result<Value> {
		match other {
			Value::UserData(ref ud) if let Ok(url) = ud.borrow::<Self>() => {
				if self.auth().covariant(url.auth()) {
					self.try_join(url.loc())?.into_lua(lua)
				} else {
					Ok(other)
				}
			}
			_ => Err("expected a Url".into_lua_err()),
		}
	}

	fn starts_with(&self, base: Value) -> mlua::Result<bool> {
		Ok(match base {
			Value::String(s) => self.try_starts_with(UrlCow::try_from(&*s.as_bytes())?)?,
			Value::UserData(ud) => self.try_starts_with(&*ud.borrow::<Self>()?)?,
			_ => Err("must be a string or Url".into_lua_err())?,
		})
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
			Err(e @ StripPrefixError::WrongEncoding) => Err(e)?,
		})
	}
}

impl FromLua for UrlBuf {
	fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
		match value {
			Value::Table(t) => t.try_into(),
			Value::String(_) | Value::UserData(_) => value.try_into(),
			_ => Err("expected a string, table, Path, or Url".into_lua_err()),
		}
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
		fields.add_cached_field("key", |_, me| Ok(me.key().to_owned()));
		fields.add_cached_field("base", |_, me| {
			Ok(Some(me.base()).filter(|u| !u.loc().is_empty()).map(Self::from))
		});
		fields.add_cached_field("parent", |_, me| Ok(me.parent().map(Self::from)));
		fields.add_cached_field("physical", |_, me| Ok(Self::from(me.physical())));
		fields.add_cached_field("trail", |_, me| Ok(Self::from(me.trail())));

		fields.add_cached_field("spec", |_, me| Ok(me.spec()));

		fields.add_field_method_get("is_absolute", |_, me| Ok(me.is_absolute()));
		fields.add_field_method_get("has_root", |_, me| Ok(me.has_root()));
	}

	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_method("ends_with", |_, me, child: Value| me.ends_with(child));
		methods.add_method("join", |lua, me, other: Value| me.join(lua, other));
		methods.add_method("resolve", |lua, me, other: Value| me.resolve(lua, other));
		methods.add_method("starts_with", |_, me, base: Value| me.starts_with(base));
		methods.add_method("strip_prefix", |_, me, base: Value| me.strip_prefix(base));

		methods.add_method_once("with_domain", |_, me, domain: BorrowedBytes| {
			Ok(me.with_domain(domain.to_vec()))
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
				Ok(format!("Url({:?}): {}", ud.to_pointer(), *ud.borrow::<Self>()?))
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
