use std::collections::HashSet;

use mlua::{ExternalError, ExternalResult, ObjectLike};
use tokio::runtime::Handle;
use yazi_macro::error;

use crate::{LuaCoroutine, Runner, fetcher::{FetchJob, FetchStatus}, loader::LOADER};

impl Runner {
	pub async fn fetch(&'static self, job: FetchJob) -> mlua::Result<Vec<FetchStatus>> {
		if job.files.is_empty() {
			return Ok(Default::default());
		}

		LOADER.ensure(&job.name, |_| ()).await?;
		tokio::task::spawn_blocking(move || Handle::current().block_on(self.fetch_do(job)))
			.await
			.into_lua_err()?
	}

	async fn fetch_do(&self, job: FetchJob) -> mlua::Result<Vec<FetchStatus>> {
		let fetcher = job.fetcher.clone();
		let mut pending: HashSet<_> = job.files.hashes().collect();
		let mut statuses = Vec::with_capacity(pending.len());

		let lua = self.spawn(&fetcher.name)?;
		let plugin = LOADER.load(&lua, &fetcher.name).await?;

		let f = plugin.call_async_method("fetch", job).await?;
		let mut co = LuaCoroutine::new(f).await?;
		while let Some(status) = co.next::<FetchStatus>(&lua).await? {
			if !pending.remove(&status.hash) {
				return Err("fetcher reported an unknown or duplicate file".into_lua_err());
			}

			statuses.push(status);
		}

		if !pending.is_empty() {
			error!("Fetcher '{}' completed before reporting every file", fetcher.name);
		}

		for hash in pending {
			statuses.push(FetchStatus { hash, retry: true, error: None });
		}
		Ok(statuses)
	}
}
