use std::{future, mem};

use mlua::{FromLua, Lua, UserData, UserDataMethods, UserDataOwned, Value};
use tokio_util::sync::CancellationToken;
use yazi_macro::impl_data_any;

#[derive(Clone, Debug, Default)]
pub struct Scope(Option<CancellationToken>);

impl_data_any!(Scope, from_into_lua = inherit);

impl From<&Scope> for Scope {
	fn from(value: &Scope) -> Self { value.clone() }
}

impl From<CancellationToken> for Scope {
	fn from(value: CancellationToken) -> Self { Self(Some(value)) }
}

impl From<Option<CancellationToken>> for Scope {
	fn from(value: Option<CancellationToken>) -> Self { Self(value) }
}

impl Scope {
	pub fn new() -> Self { Self(Some(CancellationToken::new())) }

	pub fn child(&self) -> Self {
		Self(Some(match &self.0 {
			Some(token) => token.child_token(),
			None => CancellationToken::new(),
		}))
	}

	pub fn cancel(&self) {
		if let Some(token) = &self.0 {
			token.cancel();
		}
	}

	pub fn is_cancelled(&self) -> bool {
		self.0.as_ref().is_some_and(CancellationToken::is_cancelled)
	}

	pub async fn cancelled(&self) {
		if let Some(token) = &self.0 {
			token.cancelled().await;
		} else {
			future::pending::<()>().await;
		}
	}

	pub fn take(&mut self) -> Self { mem::take(self) }
}

impl FromLua for Scope {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		if value.is_nil() {
			Ok(Self::default())
		} else {
			<UserDataOwned<Self> as FromLua>::from_lua(value, lua).map(|ud| ud.0)
		}
	}
}

impl UserData for Scope {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_method("cancel", |_, me, ()| Ok(me.cancel()));
		methods.add_method("cancelled", |_, me, ()| Ok(me.is_cancelled()));
		methods.add_method("child", |_, me, ()| Ok(me.child()));
	}
}
