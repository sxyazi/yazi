use mlua::{ExternalError, HookTriggers, ObjectLike, VmState};
use tokio::{runtime::Handle, select};

use crate::{Runner, loader::LOADER, preloader::{PreloadError, PreloadJob, PreloadState}};

impl Runner {
	pub async fn preload(&'static self, job: PreloadJob) -> Result<PreloadState, PreloadError> {
		LOADER.ensure(&job.preloader.name, |_| ()).await?;

		tokio::task::spawn_blocking(move || {
			let scope = job.scope.clone();
			Handle::current().block_on(async {
				select! {
					_ = scope.cancelled() => Err(PreloadError::Cancelled),
					r = self.preload_do(job) => r.map_err(|e| {
						e.downcast_ref::<PreloadError>().cloned().unwrap_or_else(|| e.into())
					}),
				}
			})
		})
		.await
		.unwrap_or(Err(PreloadError::Cancelled))
	}

	async fn preload_do(&self, job: PreloadJob) -> mlua::Result<PreloadState> {
		let lua = self.spawn(&job)?;
		let scope = job.scope.clone();

		lua.set_hook(
			HookTriggers::new().on_calls().on_returns().every_nth_instruction(2000),
			move |_, dbg| {
				if scope.is_cancelled() && dbg.source().what != "C" {
					Err(PreloadError::Cancelled.into_lua_err())
				} else {
					Ok(VmState::Continue)
				}
			},
		)?;

		let plugin = LOADER.load(&lua, &job.preloader.name).await?;
		if job.scope.is_cancelled() {
			Err(PreloadError::Cancelled.into_lua_err())
		} else {
			plugin.call_async_method("preload", job).await
		}
	}
}
