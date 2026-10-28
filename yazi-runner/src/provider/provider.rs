use std::{io, pin::pin};

use mlua::{FromLua, FromLuaMulti, ObjectLike};
use scopeguard::defer;
use tokio::{select, sync::mpsc};
use yazi_shared::scope::ScopeLike;
use yazi_shim::fs::Error as FsError;

use crate::{CoIter, Runner, loader::LOADER, provider::{ProvideJob, ProvideResult}};

impl Runner {
	pub async fn provide<T>(&'static self, job: ProvideJob) -> ProvideResult<T>
	where
		T: FromLua + Send + 'static,
	{
		let scope = job.to_scope();
		defer! { scope.cancel(); }

		match LOADER.ensure(&job.service.name, |_| ()).await {
			Ok(()) => self.provide_do(job).await,
			Err(e) => FsError::other(e.to_string()).into(),
		}
	}

	async fn provide_do<T>(&'static self, mut job: ProvideJob) -> ProvideResult<T>
	where
		T: FromLua + Send + 'static,
	{
		match tokio::task::spawn_blocking(move || {
			let scope = job.to_scope();
			// Keep the resource alive until the Lua VM has been destroyed.
			let _resource = job.resource.take();

			let lua = self.spawn(&job)?;
			scope.set_hook(&lua, || FsError::from(io::ErrorKind::Interrupted).into())?;

			Self::block_on(async {
				select! {
					biased;
					_ = scope.cancelled() => Err(FsError::from(io::ErrorKind::Interrupted).into()),
					r = async {
						let plugin = LOADER.load(&lua, &job.service.name).await?;
						let values = plugin.call_async_method("provide", job).await?;
						ProvideResult::from_lua_multi(values, &lua)
					} => r,
				}
			})
		})
		.await
		{
			Ok(Ok(result)) => result,
			Ok(Err(error)) => error.into(),
			Err(error) => error.into(),
		}
	}

	pub async fn provide_stream<T>(&'static self, job: ProvideJob, tx: mpsc::Sender<io::Result<T>>)
	where
		T: FromLua + Send + 'static,
	{
		let scope = job.to_scope();
		defer! { scope.cancel(); }

		let mut fut = pin!(self.provide_stream_do(job, tx.clone()));
		let result = select! {
			r = &mut fut => r,
			_ = tx.closed() => {
				scope.cancel();
				fut.await
			}
		};

		if let Err(e) = result {
			tx.send(Err(e)).await.ok();
		}
	}

	async fn provide_stream_do<T>(
		&'static self,
		mut job: ProvideJob,
		tx: mpsc::Sender<io::Result<T>>,
	) -> io::Result<()>
	where
		T: FromLua + Send + 'static,
	{
		LOADER.ensure(&job.service.name, |_| ()).await.map_err(io::Error::other)?;

		tokio::task::spawn_blocking(move || {
			let scope = job.to_scope();
			let _resource = job.resource.take();

			let lua = self.spawn(&job)?;
			scope.set_hook(&lua, || FsError::from(io::ErrorKind::Interrupted).into())?;

			let future = async {
				let plugin = LOADER.load(&lua, &job.service.name).await?;
				let mut co: CoIter = plugin.call_async_method("provide", job).await?;
				while let Some(value) = co.next(&lua).await? {
					if tx.send(Ok(value)).await.is_err() {
						break;
					}
				}
				Ok(())
			};

			Self::block_on(async {
				select! {
					biased;
					_ = scope.cancelled() => Ok(()),
					result = future => result,
				}
			})
		})
		.await
		.map_err(io::Error::from)?
		.map_err(|e: mlua::Error| FsError::try_from(e).map_or_else(io::Error::other, Into::into))
	}
}
