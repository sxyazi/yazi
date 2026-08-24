use mlua::{ExternalError, FromLua, FromLuaMulti, IntoLua, ObjectLike, Value};
use tokio::runtime::Handle;
use yazi_config::vfs::ServiceLua;
use yazi_shared::data::Sendable;

use crate::{Runner, loader::LOADER, provider::{ProvideJob, ProvideResult}};

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
			Err(e) => yazi_shim::fs::Error::other(e.to_string()).into(),
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
}
