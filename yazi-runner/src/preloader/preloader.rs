use mlua::{ExternalError, HookTriggers, ObjectLike, VmState};
use tokio::{runtime::Handle, select, sync::mpsc};

use crate::{Runner, loader::LOADER, preloader::{PreloadError, PreloadJob, PreloadState}};

impl Runner {
	pub async fn preload(
		&'static self,
		job: PreloadJob,
	) -> mpsc::Receiver<Result<PreloadState, PreloadError>> {
		let (tx, rx) = mpsc::channel(1);
		match LOADER.ensure(&job.action.name, |_| ()).await {
			Ok(()) => self.preload_do(job, tx),
			Err(e) => _ = tx.try_send(Err(e.into())),
		};
		rx
	}

	fn preload_do(
		&'static self,
		job: PreloadJob,
		tx: mpsc::Sender<Result<PreloadState, PreloadError>>,
	) {
		let tx_ = tx.clone();
		tokio::task::spawn_blocking(move || {
			let future = async {
				let lua = self.spawn(&job.action.name)?;
				lua.set_hook(
					HookTriggers::new().on_calls().on_returns().every_nth_instruction(2000),
					move |_, dbg| {
						if tx.is_closed() && dbg.source().what != "C" {
							Err(PreloadError::Cancelled.into_lua_err())
						} else {
							Ok(VmState::Continue)
						}
					},
				)?;

				let plugin = LOADER.load(&lua, &job.action.name).await?;
				if tx_.is_closed() {
					Err(PreloadError::Cancelled.into_lua_err())
				} else {
					plugin.call_async_method("preload", job).await
				}
			};

			Handle::current().block_on(async {
				select! {
					_ = tx_.closed() => {},
					r = future => match r {
						Ok(state) => _ = tx_.send(Ok(state)).await,
						Err(err) => {
							if let Some(e) = err.downcast_ref::<PreloadError>() {
								tx_.send(Err(e.clone())).await.ok();
							} else {
								tx_.send(Err(err.into())).await.ok();
							}
						},
					},
				}
			})
		});
	}
}
