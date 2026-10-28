use mlua::{ExternalError, ObjectLike};
use tokio::select;
use yazi_macro::error;

use crate::{Runner, loader::LOADER, spot::{SpotError, SpotJob}};

impl Runner {
	pub fn spot(&'static self, job: SpotJob) {
		tokio::task::spawn_blocking(move || {
			let scope = job.scope.clone();

			Self::block_on(async {
				select! {
					biased;
					_ = scope.cancelled() => {},
					result = self.spot_do(job) => {
						if let Err(e) = result
							&& !e.downcast_ref().is_some_and(SpotError::is_cancelled)
						{
							error!("{e}");
						}
					}
				}
			});
		});
	}

	async fn spot_do(&self, job: SpotJob) -> mlua::Result<()> {
		LOADER.ensure(&job.spotter.name, |_| ()).await?;

		let lua = self.spawn(&job)?;

		job.scope.set_hook(&lua, || SpotError::Cancelled.into_lua_err())?;

		let plugin = LOADER.load(&lua, &job.spotter.name).await?;
		if job.scope.is_cancelled() { Ok(()) } else { plugin.call_async_method("spot", job).await }
	}
}
