use std::borrow::Cow;

use anyhow::bail;
use hashbrown::HashSet;
use mlua::{AnyUserData, ExternalError, FromLua, IntoLua, Lua, MetaMethod, MultiValue, ObjectLike, UserData, UserDataFields, UserDataMethods, Value};
use serde::{Deserialize, Serialize};
use yazi_binding::get_metatable;
use yazi_shared::{event::CmdCow, url::UrlBufCov};

type Iter = yazi_binding::Iter<
	std::iter::Map<hashbrown::hash_set::IntoIter<UrlBufCov>, fn(UrlBufCov) -> yazi_binding::Url>,
	yazi_binding::Url,
>;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct UpdateYankedOpt<'a> {
	pub cut:  bool,
	pub urls: Cow<'a, HashSet<UrlBufCov>>,
}

impl TryFrom<CmdCow> for UpdateYankedOpt<'_> {
	type Error = anyhow::Error;

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		if let Some(opt) = c.take_any2("opt") {
			opt
		} else {
			bail!("'opt' is required for UpdateYankedOpt");
		}
	}
}

impl FromLua for UpdateYankedOpt<'_> {
	fn from_lua(_: Value, _: &Lua) -> mlua::Result<Self> { Err("unsupported".into_lua_err()) }
}

impl IntoLua for UpdateYankedOpt<'_> {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		let len = self.urls.len();
		let iter = Iter::new(self.urls.into_owned().into_iter().map(yazi_binding::Url::new), Some(len));
		UpdateYankedIter { cut: self.cut, len, inner: lua.create_userdata(iter)? }.into_lua(lua)
	}
}

// --- Iter
pub struct UpdateYankedIter {
	cut:   bool,
	len:   usize,
	inner: AnyUserData,
}

impl UpdateYankedIter {
	pub fn into_opt(self, lua: &Lua) -> mlua::Result<UpdateYankedOpt<'static>> {
		Ok(UpdateYankedOpt {
			cut:  self.cut,
			urls: Cow::Owned(
				self
					.inner
					.take::<Iter>()?
					.into_iter(lua)
					.map(|result| result.map(Into::into))
					.collect::<mlua::Result<_>>()?,
			),
		})
	}
}

impl UserData for UpdateYankedIter {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("cut", |_, me| Ok(me.cut));
	}

	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_method(MetaMethod::Len, |_, me, ()| Ok(me.len));

		methods.add_meta_method(MetaMethod::Pairs, |lua, me, ()| {
			get_metatable(lua, &me.inner)?
				.call_function::<MultiValue>(MetaMethod::Pairs.name(), me.inner.clone())
		});
	}
}
