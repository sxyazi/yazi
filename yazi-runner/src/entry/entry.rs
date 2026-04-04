use mlua::{ExternalResult, ObjectLike};
use tokio::runtime::Handle;

use crate::{Runner, entry::EntryJob, loader::LOADER};

impl Runner {
	pub async fn entry(&'static self, job: EntryJob) -> mlua::Result<()> {
		LOADER.ensure(&job.plugin, |_| ()).await.into_lua_err()?;

		tokio::task::spawn_blocking(move || {
			let lua = self.spawn(&job.plugin)?;
			Handle::current().block_on(async {
				LOADER.load(&lua, &job.plugin).await?.call_async_method("entry", job).await
			})
		})
		.await
		.into_lua_err()?
	}
}
