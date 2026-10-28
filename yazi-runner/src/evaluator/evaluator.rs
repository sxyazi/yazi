use mlua::{ExternalError, MultiValue};
use scopeguard::{ScopeGuard, guard};
use tokio::{select, task};
use tokio_util::task::AbortOnDropHandle;
use yazi_binding::{r#async::{Async, AsyncHandle}, runtime::RuntimeSeed};
use yazi_shared::{data::Data, sendable::Sendable};

use crate::{Runner, evaluator::EvaluateError};

impl Runner {
	/// The caller must provide a scope owned by this blocking VM.
	pub fn evaluate(&'static self, seed: RuntimeSeed, bytes: Vec<u8>, args: Vec<Data>) -> Async {
		Async::AsyncBlocking(AsyncHandle::spawn(seed.name.clone(), async move {
			let guard = guard(seed.scope.clone(), |scope| scope.cancel());
			let result =
				AbortOnDropHandle::new(task::spawn_blocking(move || self.evaluate_do(seed, bytes, args)))
					.await;

			ScopeGuard::into_inner(guard);
			match result {
				Ok(Err(e)) if e.downcast_ref().is_some_and(EvaluateError::is_cancelled) => None,
				Ok(r) => Some(r),
				Err(e) if e.is_cancelled() => None,
				Err(e) => Some(Err(e.into_lua_err())),
			}
		}))
	}

	fn evaluate_do(
		&self,
		seed: RuntimeSeed,
		bytes: Vec<u8>,
		args: Vec<Data>,
	) -> mlua::Result<Vec<Data>> {
		let RuntimeSeed { name, scope } = seed.clone();

		let lua = self.spawn_with(seed)?;
		scope.set_hook(&lua, || EvaluateError::Cancelled.into_lua_err())?;

		let f = lua.load(bytes).set_name(format!("={name}:async-blocking")).into_function()?;
		let args = Sendable::list_to_values(&lua, args)?;

		Self::block_on(async {
			let values: MultiValue = select! {
				biased;
				_ = scope.cancelled() => return Err(EvaluateError::Cancelled.into_lua_err()),
				result = f.call_async(args) => result?,
			};

			Sendable::values_to_list(&lua, values)
		})
	}
}
