use mlua::{ExternalError, HookTriggers, ObjectLike, VmState};
use tokio::{runtime::Handle, select};
use yazi_macro::error;

use crate::{Runner, loader::LOADER, spot::{SpotError, SpotJob}};

impl Runner {
	pub fn spot(&'static self, job: SpotJob) {
		tokio::task::spawn_blocking(move || {
			let scope = job.scope.clone();
			Handle::current().block_on(async {
				select! {
					_ = scope.cancelled() => {},
					Err(e) = self.spot_do(job) => if !e.downcast_ref().is_some_and(SpotError::is_cancelled) {
						error!("{e}");
					},
					else => {}
				}
			});
		});
	}

	async fn spot_do(&self, job: SpotJob) -> mlua::Result<()> {
		LOADER.ensure(&job.spotter.name, |_| ()).await?;

		let lua = self.spawn(&job)?;
		let scope = job.scope.clone();

		lua.set_hook(
			HookTriggers::new().on_calls().on_returns().every_nth_instruction(2000),
			move |_, dbg| {
				if scope.is_cancelled() && dbg.source().what != "C" {
					Err(SpotError::Cancelled.into_lua_err())
				} else {
					Ok(VmState::Continue)
				}
			},
		)?;

		let plugin = LOADER.load(&lua, &job.spotter.name).await?;
		if job.scope.is_cancelled() { Ok(()) } else { plugin.call_async_method("spot", job).await }
	}
}
