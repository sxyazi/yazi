use std::borrow::Cow;

use indexmap::IndexSet;
use mlua::{AnyUserData, FromLua, IntoLua, Lua, MetaMethod, MultiValue, ObjectLike, UserData, UserDataFields, UserDataMethods, Value};
use serde::{Deserialize, Serialize};
use yazi_macro::impl_data_any;
use yazi_shared::url::{UrlBuf, UrlBufCov};
use yazi_shim::mlua::get_metatable;

use super::Ember;

type Iter = yazi_binding::Iter<
	std::iter::Map<indexmap::set::IntoIter<UrlBufCov>, fn(UrlBufCov) -> UrlBuf>,
	UrlBuf,
>;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EmberYank<'a> {
	pub cut:  bool,
	pub urls: Cow<'a, IndexSet<UrlBufCov>>,
}

impl_data_any!(EmberYank<'static>, from_into_lua = inherit);

impl<'a> EmberYank<'a> {
	pub fn borrowed(cut: bool, urls: &'a IndexSet<UrlBufCov>) -> Ember<'a> {
		Self { cut, urls: Cow::Borrowed(urls) }.into()
	}
}

impl EmberYank<'static> {
	pub fn owned(cut: bool, _: &IndexSet<UrlBufCov>) -> Ember<'static> {
		Self { cut, urls: Default::default() }.into()
	}
}

impl<'a> From<EmberYank<'a>> for Ember<'a> {
	fn from(value: EmberYank<'a>) -> Self { Self::Yank(value) }
}

impl FromLua for EmberYank<'static> {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		match value {
			Value::UserData(ud) => ud.take::<EmberYankIter>()?.collect(lua),
			_ => Err(mlua::Error::FromLuaConversionError {
				from:    value.type_name(),
				to:      "EmberYank".to_owned(),
				message: Some("expected EmberYankIter userdata".to_owned()),
			}),
		}
	}
}

impl IntoLua for EmberYank<'_> {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		let len = self.urls.len();
		let iter = Iter::new(self.urls.into_owned().into_iter().map(UrlBuf::from), Some(len));
		EmberYankIter { cut: self.cut, len, inner: lua.create_userdata(iter)? }.into_lua(lua)
	}
}

// --- Iter
pub struct EmberYankIter {
	cut:   bool,
	len:   usize,
	inner: AnyUserData,
}

impl EmberYankIter {
	pub fn collect(self, lua: &Lua) -> mlua::Result<EmberYank<'static>> {
		Ok(EmberYank {
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

impl UserData for EmberYankIter {
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
