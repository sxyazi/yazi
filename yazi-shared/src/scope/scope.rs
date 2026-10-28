use std::future;

use mlua::{FromLua, HookTriggers, Lua, MetaMethod, UserData, UserDataFields, UserDataMethods, UserDataOwned, UserDataRef, UserDataRegistry, Value, VmState};
use tokio_util::sync::CancellationToken;
use yazi_macro::impl_data_any;

use super::{ScopeInventory, ScopeLike};
use crate::{data::Data, id::Id};

pub type ScopeRef = UserDataRef<Scope>;

#[derive(Clone, Debug, Default)]
pub struct Scope {
	pub tab: Id,
	token:   Option<CancellationToken>,
}

impl_data_any!(Scope, from_into_lua = inherit);

impl Default for &Scope {
	fn default() -> Self {
		static SCOPE: Scope = Scope { tab: Id::ZERO, token: None };
		&SCOPE
	}
}

impl From<CancellationToken> for Scope {
	fn from(value: CancellationToken) -> Self { Self { tab: Id::ZERO, token: Some(value) } }
}

impl From<Option<CancellationToken>> for Scope {
	fn from(value: Option<CancellationToken>) -> Self { Self { tab: Id::ZERO, token: value } }
}

impl From<Scope> for Data {
	fn from(value: Scope) -> Self { Self::Any(Box::new(value)) }
}

impl TryFrom<Data> for Scope {
	type Error = anyhow::Error;

	fn try_from(value: Data) -> Result<Self, Self::Error> {
		if let Data::Any(_) = value {
			value.into_any2()
		} else {
			Ok(Self::default().with_tab(value.try_into()?))
		}
	}
}

impl Scope {
	pub fn new(tab: Id) -> Self { Self { tab, token: Some(CancellationToken::new()) } }

	pub fn for_tab(&self, tab: Id) -> Self { self.clone().with_tab(tab.nz().unwrap_or(self.tab)) }

	pub fn with_tab(mut self, tab: Id) -> Self {
		self.tab = tab;
		self
	}

	pub fn cancellable(mut self) -> Self {
		self.token.get_or_insert_with(CancellationToken::new);
		self
	}

	pub fn child(&self) -> Self {
		let token =
			self.token.as_ref().map_or_else(CancellationToken::new, CancellationToken::child_token);

		Self { tab: self.tab, token: Some(token) }
	}

	pub fn cancel(&self) {
		if let Some(token) = &self.token {
			token.cancel();
		}
	}

	pub fn detached(&self) -> Self { Self::new(self.tab) }

	pub fn is_cancelled(&self) -> bool {
		self.token.as_ref().is_some_and(CancellationToken::is_cancelled)
	}

	pub async fn cancelled(&self) {
		if let Some(token) = &self.token {
			token.cancelled().await;
		} else {
			future::pending::<()>().await;
		}
	}

	pub fn set_hook(&self, lua: &Lua, error: fn() -> mlua::Error) -> mlua::Result<()> {
		let Some(token) = self.token.clone() else { return Ok(()) };

		// Async calls create coroutines that must inherit the cancellation hook.
		lua.set_global_hook(HookTriggers::new().every_nth_instruction(1000), move |_, _| {
			if token.is_cancelled() { Err(error()) } else { Ok(VmState::Continue) }
		})
	}
}

impl ScopeLike for Scope {
	fn scope(&self) -> &Scope { self }
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
	fn register(registry: &mut UserDataRegistry<Self>) {
		Self::add_fields(registry);
		Self::add_methods(registry);

		for inv in inventory::iter::<ScopeInventory>() {
			(inv.register)(registry);
		}
	}

	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("tab", |_, me| Ok(me.tab));
	}

	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_method("cancel", |_, me, ()| Ok(me.cancel()));
		methods.add_method("cancelled", |_, me, ()| Ok(me.is_cancelled()));
		methods.add_method("child", |_, me, ()| Ok(me.child()));

		methods
			.add_async_meta_method(MetaMethod::Call, |_, me, ()| async move { Ok(me.cancelled().await) });
	}
}
