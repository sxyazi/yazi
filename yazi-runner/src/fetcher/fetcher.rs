use std::collections::HashSet;

use mlua::{ExternalError, ExternalResult, FromLuaMulti, Function, MultiValue, ObjectLike, Value};
use tokio::runtime::Handle;
use yazi_macro::error;

use crate::{Runner, fetcher::{FetchJob, FetchStatus}, loader::LOADER};

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

		let next: Function = plugin.call_async_method("fetch", job).await?;
		let mut values: MultiValue = next.call_async(()).await?;
		loop {
			if values.front().is_none_or(Value::is_nil) {
				break;
			}

			let status = FetchStatus::from_lua_multi(values, &lua)?;
			if !pending.remove(&status.hash) {
				return Err("fetcher reported an unknown or duplicate file".into_lua_err());
			}

			statuses.push(status);
			values = next.call_async(true).await?;
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
