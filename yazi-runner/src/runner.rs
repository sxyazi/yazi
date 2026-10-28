use std::future::Future;

use mlua::Lua;
use tokio::{runtime::Handle, task::LocalSet};
use yazi_binding::runtime::{Runtime, RuntimeSeed};

use crate::{PluginJob, entry::EntryError, evaluator::EvaluateError, fetcher::FetchError, preloader::PreloadError, previewer::PeekError, spot::SpotError};

pub struct Runner {
	pub(super) setter: fn(&Lua) -> mlua::Result<()>,
}

impl Runner {
	pub(crate) fn spawn(&self, job: &impl PluginJob) -> mlua::Result<Lua> {
		self.spawn_with(RuntimeSeed::new(job.name().as_ref(), job))
	}

	pub(crate) fn spawn_with(&self, seed: RuntimeSeed) -> mlua::Result<Lua> {
		let lua = Lua::new();
		lua.set_app_data(Runtime::new(seed));

		(self.setter)(&lua)?;
		Ok(lua)
	}

	pub(crate) fn block_on<T>(future: impl Future<Output = T>) -> T {
		Handle::current().block_on(LocalSet::new().run_until(future))
	}

	pub fn is_cancelled(error: &mlua::Error) -> bool {
		error.downcast_ref().is_some_and(PeekError::is_cancelled)
			|| error.downcast_ref().is_some_and(PreloadError::is_cancelled)
			|| error.downcast_ref().is_some_and(SpotError::is_cancelled)
			|| error.downcast_ref().is_some_and(EntryError::is_cancelled)
			|| error.downcast_ref().is_some_and(FetchError::is_cancelled)
			|| error.downcast_ref().is_some_and(EvaluateError::is_cancelled)
	}
}
