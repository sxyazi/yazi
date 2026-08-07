use std::{future, mem};

use mlua::UserData;
use tokio_util::sync::CancellationToken;
use yazi_codegen::FromLuaOwned;
use yazi_macro::impl_data_any;

#[derive(Clone, Debug, Default, FromLuaOwned)]
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

impl UserData for Scope {}
