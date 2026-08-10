use mlua::{ExternalError, HookTriggers, IntoLua, ObjectLike, VmState};
use tokio::{runtime::Handle, select};
use yazi_binding::Scope;
use yazi_config::plugin::SpotterArc;
use yazi_fs::file::File;
use yazi_macro::error;
use yazi_shared::{data::Sendable, id::Ids, pool::Symbol};

use crate::{Runner, loader::LOADER};

static IDS: Ids = Ids::new();

impl Runner {
	pub fn spot(
		&'static self,
		spotter: SpotterArc,
		file: File,
		mime: Symbol<str>,
		skip: usize,
	) -> Scope {
		let scope = Scope::new();
		let (scope1, scope2) = (scope.clone(), scope.clone());

		tokio::task::spawn_blocking(move || {
			let future = async {
				LOADER.ensure(&spotter.name, |_| ()).await?;

				let lua = self.spawn(&spotter.name)?;
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

				let plugin = LOADER.load(&lua, &spotter.name).await?;
				let job = lua.create_table_from([
					("id", IDS.next().into_lua(&lua)?),
					("args", Sendable::args_to_table_ref(&lua, &spotter.args)?.into_lua(&lua)?),
					("file", file.into_lua(&lua)?),
					("mime", mime.into_lua(&lua)?),
					("skip", skip.into_lua(&lua)?),
				])?;

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
