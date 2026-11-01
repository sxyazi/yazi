use mlua::{ExternalError, ObjectLike};
use tokio::select;

use crate::{Runner, loader::LOADER, preloader::{PreloadError, PreloadJob, PreloadState}};

impl Runner {
	pub async fn preload(job: PreloadJob) -> Result<PreloadState, PreloadError> {
		LOADER.ensure(&job.preloader.name, |_| ()).await?;

		tokio::task::spawn_blocking(move || {
			let scope = job.scope.clone();
			Self::block_on(async {
				select! {
					biased;
					_ = scope.cancelled() => Err(PreloadError::Cancelled),
					r = Self::preload_do(job) => r.map_err(|e| {
						e.downcast_ref::<PreloadError>().cloned().unwrap_or_else(|| e.into())
					}),
				}
			})
		})
		.await
		.unwrap_or(Err(PreloadError::Cancelled))
	}

	async fn preload_do(job: PreloadJob) -> mlua::Result<PreloadState> {
		let lua = Self::spawn(&job)?;

		job.scope.set_hook(&lua, || PreloadError::Cancelled.into_lua_err())?;

		let plugin = LOADER.load(&lua, &job.preloader.name).await?;
		if job.scope.is_cancelled() {
			Err(PreloadError::Cancelled.into_lua_err())
		} else {
			plugin.call_async_method("preload", job).await
		}
	}
}
