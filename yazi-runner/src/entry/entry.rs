use mlua::{ExternalError, ExternalResult, HookTriggers, ObjectLike, VmState};
use tokio::{runtime::Handle, select};

use crate::{Runner, entry::EntryJob, loader::LOADER};

impl Runner {
	pub async fn entry(&'static self, job: EntryJob) -> mlua::Result<()> {
		LOADER.ensure(&job.plugin, |_| ()).await?;

		tokio::task::spawn_blocking(move || {
			let scope = job.scope.clone();
			let lua = self.spawn(&job)?;

			// The plugin runs on a blocking thread, which cannot be aborted once
			// started. Instead bail out cooperatively: interrupt the Lua VM via a
			// hook and race the plugin call against the scope, so that cancelling
			// the task tears the plugin down -- and with it any child process it
			// spawned (e.g. `7zz`), which is killed on drop.
			let scope_ = scope.clone();
			lua.set_hook(
				HookTriggers::new().on_calls().on_returns().every_nth_instruction(2000),
				move |_, _| {
					if scope_.is_cancelled() {
						Err("Entry task cancelled".into_lua_err())
					} else {
						Ok(VmState::Continue)
					}
				},
			)?;

			Handle::current().block_on(async {
				select! {
					_ = scope.cancelled() => Ok(()),
					r = async {
						LOADER.load(&lua, &job.plugin).await?.call_async_method("entry", job).await
					} => r,
				}
			})
		})
		.await
		.into_lua_err()?
	}
}

#[cfg(test)]
mod tests {
	use std::time::Duration;

	use tokio::sync::mpsc;
	use yazi_binding::{Scope, runtime::{Runtime, RuntimeSeed}};
	use yazi_shim::SStr;

	use crate::{PluginJob, entry::EntryJob};

	/// Regression test for sxyazi/yazi#4404.
	///
	/// The built-in `extract` opener runs through the entry path
	/// (`ya pub extract` -> `ya.task("plugin", ...)`). That plugin spawns `7zz`
	/// and blocks on it. Cancelling the task from the `w` task manager must stop
	/// the plugin -- and with it the child process.
	///
	/// Before the fix, `EntryJob` carried no scope and `Runner::spawn` passed
	/// `Scope::default()`, so `rt.scope()` inside an entry plugin was dead:
	/// cancelling the task flipped the task handle but nothing reached the
	/// plugin, so `7zz` kept running to completion and the files still landed on
	/// disk.
	///
	/// This asserts the API-level invariant the fix restores: an `EntryJob`
	/// exposes a *live* scope to the runner, and that exact scope is what the
	/// runtime frame (`rt.scope()` / `ya.scope()`) reads back and what the
	/// runner's cancellation guards observe.
	#[test]
	fn entry_job_scope_is_live_and_reaches_the_runtime_frame() {
		let scope = Scope::new();
		let job = EntryJob { plugin: SStr::from("extract"), scope: scope.clone(), ..Default::default() };

		// The job must hand a scope to the runner instead of `None` -- `None`
		// is what made `Runner::spawn` fall back to a detached default scope.
		let exposed = job.scope().expect("entry job must carry a scope");
		assert!(!exposed.is_cancelled(), "a fresh entry scope must not be pre-cancelled");

		// Rebuild the seed exactly the way `Runner::spawn` does and read the
		// scope back out of the runtime frame, which is what `ya.scope()` does.
		let runtime = Runtime::new(RuntimeSeed::new(
			job.tab(),
			job.name().as_ref(),
			job.scope().unwrap_or_default(),
		));
		let seen = runtime.scope();
		assert!(!seen.is_cancelled());

		// Cancelling the task must be visible through both handles.
		scope.cancel();
		assert!(exposed.is_cancelled(), "cancelling the task must cancel the entry scope");
		assert!(seen.is_cancelled(), "`rt.scope()` must reflect the entry task's cancellation");
	}

	/// The behaviour the user actually sees: a plugin body that waits on its
	/// scope must unblock the moment the task is cancelled, instead of running to
	/// completion. This mirrors the `select! { _ = scope.cancelled() => .. }`
	/// guard now used by `Runner::entry`, upstream of which the `7zz` child is
	/// killed on drop.
	#[tokio::test]
	async fn cancelling_entry_scope_unblocks_a_scope_aware_plugin_body() {
		let scope = Scope::new();
		let (tx, mut rx) = mpsc::channel::<()>(1);

		let scope_ = scope.clone();
		let handle = tokio::spawn(async move {
			tokio::select! {
				_ = scope_.cancelled() => {
					let _ = tx.send(()).await;
				}
				_ = std::future::pending::<()>() => {}
			}
		});

		scope.cancel();
		tokio::time::timeout(Duration::from_secs(5), rx.recv())
			.await
			.expect("entry plugin must observe cancellation instead of running to completion")
			.expect("channel must yield");

		handle.await.unwrap();
	}
}
