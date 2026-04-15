use mlua::{ExternalResult, ObjectLike};
use tokio::runtime::Handle;
use yazi_binding::Error;

use crate::{Runner, fetcher::{FetchJob, FetchState}, loader::LOADER};

impl Runner {
	pub async fn fetch(&'static self, job: FetchJob) -> mlua::Result<(FetchState, Option<Error>)> {
		if job.files.is_empty() {
			return Ok((FetchState::Bool(true), None));
		}
		LOADER.ensure(&job.fetcher.name, |_| ()).await.into_lua_err()?;

		tokio::task::spawn_blocking(move || {
			let lua = self.spawn(&job.fetcher.name)?;
			Handle::current().block_on(async {
				LOADER.load(&lua, &job.fetcher.name).await?.call_async_method("fetch", job).await
			})
		})
		.await
		.into_lua_err()?
	}
}
