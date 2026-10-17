use mlua::{ExternalError, HookTriggers, ObjectLike, VmState};
use tokio::{runtime::Handle, select};

use crate::{Runner, loader::LOADER, previewer::{PeekError, PeekJob}};

impl Runner {
	pub async fn peek(&'static self, job: &PeekJob) -> Result<(), PeekError> {
		if LOADER.ensure(&job.previewer.name, |c| c.sync_peek).await? {
			return Err(PeekError::ShouldSync);
		}

		let job = job.clone();
		tokio::task::spawn_blocking(move || {
			let scope = job.scope.clone();
			Handle::current().block_on(async {
				select! {
					_ = scope.cancelled() => Err(PeekError::Cancelled),
					r = self.peek_do(job) => r.map_err(|e| {
						e.downcast_ref::<PeekError>().cloned().unwrap_or_else(|| e.into())
					}),
				}
			})
		})
		.await
		.unwrap_or(Err(PeekError::Cancelled))
	}

	async fn peek_do(&self, job: PeekJob) -> mlua::Result<()> {
		let lua = self.spawn(&job)?;
		let scope = job.scope.clone();

		lua.set_hook(
			HookTriggers::new().on_calls().on_returns().every_nth_instruction(2000),
			move |_, dbg| {
				if scope.is_cancelled() && dbg.source().what != "C" {
					Err(PeekError::Cancelled.into_lua_err())
				} else {
					Ok(VmState::Continue)
				}
			},
		)?;

		let plugin = LOADER.load(&lua, &job.previewer.name).await?;
		if job.scope.is_cancelled() {
			Err(PeekError::Cancelled.into_lua_err())
		} else {
			plugin.call_async_method("peek", job).await
		}
	}
}
