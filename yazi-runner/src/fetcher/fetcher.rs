use std::collections::HashSet;

use mlua::{ExternalError, ExternalResult, FromLua, Function, MultiValue, ObjectLike, Value};
use tokio::runtime::Handle;
use tracing::error;
use yazi_fs::{FsHash64, file::{FileRef, FileSig}};
use yazi_shim::fs::Error;

use crate::{Runner, fetcher::FetchJob, loader::LOADER};

pub struct FetchStatus {
	pub hash:    u64,
	pub success: bool,
	pub error:   Option<Error>,
}

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
			let file = values.pop_front().unwrap_or(Value::Nil);
			if file.is_nil() {
				break;
			}

			let hash = FileRef::from_lua(file, &lua)?.borrow(|f| Ok(FileSig(f).hash_u64()))?;
			if !pending.remove(&hash) {
				return Err("fetcher reported an unknown or duplicate file".into_lua_err());
			}

			let error = Error::from_lua(values.pop_front().unwrap_or(Value::Nil), &lua).ok();
			statuses.push(FetchStatus { hash, success: error.is_none(), error });

			values = next.call_async(true).await?;
		}

		if !pending.is_empty() {
			error!("Fetcher '{}' completed before reporting every file", fetcher.name);
		}

		for hash in pending {
			statuses.push(FetchStatus { hash, success: false, error: None });
		}
		Ok(statuses)
	}
}
