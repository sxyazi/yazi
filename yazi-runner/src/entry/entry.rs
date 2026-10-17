use mlua::{ExternalResult, HookTriggers, ObjectLike, VmState};
use tokio::{runtime::Handle, select};

use crate::{Runner, entry::{EntryError, EntryJob}, loader::LOADER};

impl Runner {
	pub async fn entry(&'static self, job: EntryJob) -> mlua::Result<()> {
		LOADER.ensure(&job.plugin, |_| ()).await?;

		if job.scope.is_cancelled() {
			return Err(EntryError::Cancelled.into());
		}

		tokio::task::spawn_blocking(move || {
			let scope = job.scope.clone();
			Handle::current().block_on(async {
				select! {
					_ = scope.cancelled() => Err(EntryError::Cancelled.into()),
					r = self.entry_do(job) => r,
				}
			})
		})
		.await
		.into_lua_err()?
	}

	async fn entry_do(&self, job: EntryJob) -> mlua::Result<()> {
		let lua = self.spawn(&job)?;
		let scope = job.scope.clone();

		lua.set_hook(
			HookTriggers::new().on_calls().on_returns().every_nth_instruction(2000),
			move |_, dbg| {
				if scope.is_cancelled() && dbg.source().what != "C" {
					Err(EntryError::Cancelled.into())
				} else {
					Ok(VmState::Continue)
				}
			},
		)?;

		let plugin = LOADER.load(&lua, &job.plugin).await?;
		if job.scope.is_cancelled() {
			return Err(EntryError::Cancelled.into());
		}

		plugin.call_async_method("entry", job).await
	}
}
