use mlua::{ExternalError, HookTriggers, ObjectLike, VmState};
use tokio::{runtime::Handle, select, sync::mpsc};

use crate::{Runner, loader::LOADER, previewer::{PeekError, PeekJob}};

impl Runner {
	pub async fn peek(&'static self, job: &PeekJob) -> mpsc::Receiver<Result<(), PeekError>> {
		let (tx, rx) = mpsc::channel(1);
		match LOADER.ensure(&job.action.name, |c| c.sync_peek).await {
			Ok(true) => _ = tx.try_send(Err(PeekError::ShouldSync)),
			Ok(false) => self.peek_do(job, tx),
			Err(e) => _ = tx.try_send(Err(e.into())),
		}
		rx
	}

	fn peek_do(&'static self, job: &PeekJob, tx: mpsc::Sender<Result<(), PeekError>>) {
		let (tx_, job) = (tx.clone(), job.clone());
		tokio::task::spawn_blocking(move || {
			let future = async {
				let lua = self.spawn(&job.action.name)?;
				lua.set_hook(
					HookTriggers::new().on_calls().on_returns().every_nth_instruction(2000),
					move |_, dbg| {
						if tx.is_closed() && dbg.source().what != "C" {
							Err(PeekError::Cancelled.into_lua_err())
						} else {
							Ok(VmState::Continue)
						}
					},
				)?;

				let plugin = LOADER.load(&lua, &job.action.name).await?;
				if tx_.is_closed() {
					Err(PeekError::Cancelled.into_lua_err())
				} else {
					plugin.call_async_method("peek", job).await
				}
			};

			Handle::current().block_on(async {
				select! {
					_ = tx_.closed() => {},
					r = future => match r {
						Ok(()) => _ = tx_.send(Ok(())).await,
						Err(err) => {
							if let Some(e) = err.downcast_ref::<PeekError>() {
								tx_.send(Err(e.clone())).await.ok();
							} else {
								tx_.send(Err(err.into())).await.ok();
							}
						},
					},
				}
			});
		});
	}
}
