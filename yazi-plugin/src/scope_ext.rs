use mlua::{ExternalError, Function, Lua, MultiValue, UserDataMethods};
use tokio::select;
use yazi_binding::{r#async::{Async, AsyncHandle}, runtime, runtime::{RuntimeCo, RuntimeSeed}};
use yazi_runner::{RUNNER, Runner};
use yazi_shared::{scope::{Scope, ScopeInventory, ScopeLike, ScopeRef}, sendable::Sendable};

pub(crate) trait ScopeExt {
	fn spawn(self, lua: &Lua, f: Function, args: MultiValue) -> mlua::Result<Async>;

	fn spawn_blocking(self, lua: &Lua, f: Function, args: MultiValue) -> mlua::Result<Async>;
}

impl ScopeExt for Scope {
	fn spawn(self, lua: &Lua, f: Function, args: MultiValue) -> mlua::Result<Async> {
		let seed = RuntimeSeed::new(runtime!(lua)?.name()?, self);
		let future = RuntimeCo::new(&seed, lua.clone(), f.call_async(args));

		Ok(Async::Async(AsyncHandle::spawn(seed.name, async move {
			let result = select! {
				biased;
				_ = seed.scope.cancelled() => return None,
				r = future => r,
			};

			match result {
				Err(e) if Runner::is_cancelled(&e) => None,
				r => Some(r),
			}
		})))
	}

	fn spawn_blocking(self, lua: &Lua, f: Function, args: MultiValue) -> mlua::Result<Async> {
		let info = f.info();
		if info.what == "C" {
			return Err("Blocking task callback must be a Lua function".into_lua_err());
		}
		if info.num_upvalues > 1 || info.num_upvalues == 1 && f.environment().is_none() {
			return Err("Blocking task callback cannot capture local values".into_lua_err());
		}

		let seed = RuntimeSeed::new(runtime!(lua)?.name()?, self);
		let bytes = f.dump(false);
		let args = Sendable::values_to_list(lua, args)?;
		Ok(RUNNER.evaluate(seed, bytes, args))
	}
}

inventory::submit! {
	ScopeInventory {
		register: |registry| {
			registry.add_method("spawn", |lua, me, (f, args): (Function, MultiValue)| {
				me.to_scope().spawn(lua, f, args)
			});
			registry.add_function("spawn_blocking", |lua, (me, f, args): (ScopeRef, Function, MultiValue)| {
				let child = me.child();
				drop(me); // Release the borrow before argument conversion can consume the same scope userdata.
				child.spawn_blocking(lua, f, args)
			});
		}
	}
}
