use mlua::{ExternalResult, ObjectLike};
use tokio::select;

use crate::{Runner, entry::{EntryError, EntryJob}, loader::LOADER};

impl Runner {
	pub async fn entry(&'static self, job: EntryJob) -> mlua::Result<()> {
		LOADER.ensure(&job.plugin, |_| ()).await?;

		if job.scope.is_cancelled() {
			return Err(EntryError::Cancelled.into());
		}

		tokio::task::spawn_blocking(move || {
			let scope = job.scope.clone();
			Self::block_on(async {
				select! {
					biased;
					_ = scope.cancelled() => Err(EntryError::Cancelled.into()),
					r = self.entry_do(job) => r,
				}
			})
		})
		.await
		.into_lua_err()?
	}

	async fn entry_do(&self, job: EntryJob) -> mlua::Result<()> {
		let lua = self.spawn(&job)?;

		job.scope.set_hook(&lua, || EntryError::Cancelled.into())?;

		let plugin = LOADER.load(&lua, &job.plugin).await?;
		if job.scope.is_cancelled() {
			return Err(EntryError::Cancelled.into());
		}

		plugin.call_async_method("entry", job).await
	}
}
