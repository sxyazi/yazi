use std::collections::HashSet;

use mlua::{ExternalError, ExternalResult, ObjectLike};
use tokio::select;
use yazi_macro::error;

use crate::{CoIter, Runner, fetcher::{FetchError, FetchJob, FetchStatus}, loader::LOADER};

impl Runner {
	pub async fn fetch(&'static self, job: FetchJob) -> mlua::Result<Vec<FetchStatus>> {
		if job.files.is_empty() {
			return Ok(Default::default());
		}

		LOADER.ensure(&job.name, |_| ()).await?;
		tokio::task::spawn_blocking(move || {
			let scope = job.scope.clone();
			Self::block_on(async {
				select! {
					biased;
					_ = scope.cancelled() => Err(FetchError::Cancelled.into_lua_err()),
					r = self.fetch_do(job) => r,
				}
			})
		})
		.await
		.into_lua_err()?
	}

	async fn fetch_do(&self, job: FetchJob) -> mlua::Result<Vec<FetchStatus>> {
		let fetcher = job.fetcher.clone();
		let mut pending: HashSet<_> = job.files.hashes().collect();
		let mut statuses = Vec::with_capacity(pending.len());

		let lua = self.spawn(&job)?;

		job.scope.set_hook(&lua, || FetchError::Cancelled.into_lua_err())?;

		let plugin = LOADER.load(&lua, &fetcher.name).await?;
		if job.scope.is_cancelled() {
			return Err(FetchError::Cancelled.into_lua_err());
		}

		let mut co: CoIter = plugin.call_async_method("fetch", job).await?;
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
