use std::future::Future;

use anyhow::Result;
use tokio::task::LocalSet;

pub async fn run<T>(future: impl Future<Output = Result<T>>) -> Result<T> {
	let local = LocalSet::new();
	let future = local.run_until(future);

	#[cfg(windows)]
	{
		crate::MessageLoop::run(future).await?
	}
	#[cfg(not(windows))]
	future.await
}
