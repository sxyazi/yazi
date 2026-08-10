use compact_str::CompactString;
use mlua::{ExternalError, HookTriggers, VmState};
use tokio::{runtime::Handle, select};
use yazi_binding::Scope;
use yazi_shared::data::{Data, Sendable};

use crate::Runner;

pub(super) struct EvaluateJob {
	pub(super) runner: &'static Runner,
	pub(super) scope:  Scope,
	pub(super) name:   CompactString,
	pub(super) bytes:  Vec<u8>,
	pub(super) arg:    Data,
}

impl EvaluateJob {
	pub(super) fn eval(self) {
		let Self { runner, scope, name, bytes, arg } = self;
		let result = (|| -> mlua::Result<()> {
			let lua = runner.spawn_with(&scope, &name)?;

			let scope_ = scope.clone();
			lua.set_hook(
				HookTriggers::new().on_calls().on_returns().every_nth_instruction(2000),
				move |_, _| {
					if scope_.is_cancelled() {
						Err("async blocking task cancelled".into_lua_err())
					} else {
						Ok(VmState::Continue)
					}
				},
			)?;

			let f = lua.load(bytes).set_name(format!("={name}:async-blocking")).into_function()?;
			let arg = Sendable::data_to_value(&lua, arg)?;
			Handle::current().block_on(async {
				select! {
					_ = scope.cancelled() => Ok(()),
					result = f.call_async(arg) => result,
				}
			})
		})();

		if let Err(e) = result
			&& !scope.is_cancelled()
		{
			yazi_macro::error!("Failed to execute async blocking task in `{name}` plugin: {e}");
		}
	}
}
