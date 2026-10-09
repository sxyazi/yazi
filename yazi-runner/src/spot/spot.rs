use mlua::{ExternalError, HookTriggers, ObjectLike, VmState};
use tokio::{runtime::Handle, select};
use yazi_binding::Scope;
use yazi_macro::error;

use crate::{Runner, loader::LOADER, spot::SpotJob};

impl Runner {
	pub fn spot(&'static self, job: SpotJob) -> Scope {
		let scope = Scope::new();
		let (scope1, scope2) = (scope.clone(), scope.clone());

		tokio::task::spawn_blocking(move || {
			let future = async {
				LOADER.ensure(&job.spotter.name, |_| ()).await?;

				let lua = self.spawn(&job)?;
				lua.set_hook(
					HookTriggers::new().on_calls().on_returns().every_nth_instruction(2000),
					move |_, dbg| {
						if scope1.is_cancelled() && dbg.source().what != "C" {
							Err("Spot task cancelled".into_lua_err())
						} else {
							Ok(VmState::Continue)
						}
					},
				)?;

				let plugin = LOADER.load(&lua, &job.spotter.name).await?;
				if scope2.is_cancelled() { Ok(()) } else { plugin.call_async_method("spot", job).await }
			};

			Handle::current().block_on(async {
				select! {
					_ = scope2.cancelled() => {},
					Err(e) = future => if !e.to_string().contains("Spot task cancelled") {
						error!("{e}");
					},
					else => {}
				}
			});
		});

		scope
	}
}
