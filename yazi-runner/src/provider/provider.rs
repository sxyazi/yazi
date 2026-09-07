use std::io;

use mlua::{ExternalError, FromLua, FromLuaMulti, IntoLua, ObjectLike, Value};
use tokio::{runtime::Handle, select, sync::mpsc};
use yazi_config::vfs::ServiceLua;
use yazi_shared::sendable::Sendable;
use yazi_shim::fs::Error as FsError;

use crate::{LuaCoroutine, Runner, loader::LOADER, provider::{ProvideJob, ProvideResult}};

impl Runner {
	pub async fn provide<T>(
		&'static self,
		service: &'static ServiceLua,
		job: ProvideJob,
	) -> ProvideResult<T>
	where
		T: FromLua + Send + 'static,
	{
		match LOADER.ensure(&service.name, |_| ()).await {
			Ok(()) => self.provide_do(service, job).await,
			Err(e) => FsError::other(e.to_string()).into(),
		}
	}

	async fn provide_do<T>(
		&'static self,
		service: &'static ServiceLua,
		job: ProvideJob,
	) -> ProvideResult<T>
	where
		T: FromLua + Send + 'static,
	{
		match tokio::task::spawn_blocking(move || {
			let lua = self.spawn(&service.name)?;

			Handle::current().block_on(async {
				let Value::Table(job) = job.into_lua(&lua)? else {
					return Err("ProvideJob should be a table".into_lua_err());
				};
				job.raw_set("args", Sendable::args_to_table_ref(&lua, &service.args)?)?;
				job.raw_set("opts", Sendable::args_to_table_ref(&lua, &service.opts)?)?;

				let values =
					LOADER.load(&lua, &service.name).await?.call_async_method("provide", job).await?;
				ProvideResult::from_lua_multi(values, &lua)
			})
		})
		.await
		{
			Ok(Ok(result)) => result,
			Ok(Err(error)) => error.into(),
			Err(error) => error.into(),
		}
	}

	pub async fn provide_stream<T>(
		&'static self,
		service: &'static ServiceLua,
		job: ProvideJob,
		tx: mpsc::Sender<io::Result<T>>,
	) where
		T: FromLua + Send + 'static,
	{
		if let Err(e) = self.provide_stream_do(service, job, tx.clone()).await {
			tx.send(Err(e)).await.ok();
		}
	}

	async fn provide_stream_do<T>(
		&'static self,
		service: &'static ServiceLua,
		job: ProvideJob,
		tx: mpsc::Sender<io::Result<T>>,
	) -> io::Result<()>
	where
		T: FromLua + Send + 'static,
	{
		LOADER.ensure(&service.name, |_| ()).await.map_err(io::Error::other)?;

		tokio::task::spawn_blocking(move || {
			let lua = self.spawn(&service.name)?;

			let future = async {
				let Value::Table(job) = job.into_lua(&lua)? else {
					return Err("ProvideJob should be a table".into_lua_err());
				};
				job.raw_set("args", Sendable::args_to_table_ref(&lua, &service.args)?)?;
				job.raw_set("opts", Sendable::args_to_table_ref(&lua, &service.opts)?)?;

				let f = LOADER.load(&lua, &service.name).await?.call_async_method("provide", job).await?;
				let mut co = LuaCoroutine::new(f).await?;
				while let Some(value) = co.next(&lua).await? {
					if tx.send(Ok(value)).await.is_err() {
						break;
					}
				}
				Ok(())
			};

			Handle::current().block_on(async {
				select! {
					_ = tx.closed() => Ok(()),
					result = future => result,
				}
			})
		})
		.await
		.map_err(io::Error::other)?
		.map_err(|e| FsError::try_from(e).map_or_else(io::Error::other, Into::into))
	}
}
