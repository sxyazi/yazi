use mlua::{ExternalError, FromLua, FromLuaMulti, IntoLua, ObjectLike, Value};
use tokio::runtime::Handle;
use yazi_shared::{data::Sendable, event::Cmd};

use crate::{Runner, loader::LOADER, provider::{ProvideResult, ProviderJob}};

impl Runner {
	pub async fn provide<T>(&'static self, run: &'static Cmd, job: ProviderJob) -> ProvideResult<T>
	where
		T: FromLua + Send + 'static,
	{
		match LOADER.ensure(&run.name, |_| ()).await {
			Ok(()) => self.provide_do(run, job).await,
			Err(e) => yazi_binding::Error::custom(e.to_string()).into(),
		}
	}

	async fn provide_do<T>(&'static self, run: &'static Cmd, job: ProviderJob) -> ProvideResult<T>
	where
		T: FromLua + Send + 'static,
	{
		match tokio::task::spawn_blocking(move || {
			let lua = self.spawn(&run.name)?;

			Handle::current().block_on(async {
				let Value::Table(job) = job.into_lua(&lua)? else {
					return Err("ProviderJob should be a table".into_lua_err());
				};
				job.raw_set("args", Sendable::args_to_table_ref(&lua, &run.args)?)?;

				let values = LOADER.load(&lua, &run.name).await?.call_async_method("provide", job).await?;
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
