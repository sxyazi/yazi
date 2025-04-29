use std::borrow::Cow;

use mlua::{ExternalError, ExternalResult, HookTriggers, IntoLua, ObjectLike, VmState};
use tokio::{runtime::Handle, select};
use tokio_util::sync::CancellationToken;
use tracing::error;
use yazi_binding::Id;
use yazi_dds::Sendable;
use yazi_shared::{Ids, event::Cmd};

use super::slim_lua;
use crate::{file::File, loader::LOADER};

static IDS: Ids = Ids::new();

pub fn spot(
	cmd: &'static Cmd,
	file: yazi_fs::File,
	mime: Cow<'static, str>,
	skip: usize,
) -> CancellationToken {
	let ct = CancellationToken::new();
	let (ct1, ct2) = (ct.clone(), ct.clone());

	tokio::task::spawn_blocking(move || {
		let future = async {
			LOADER.ensure(&cmd.name, |_| ()).await.into_lua_err()?;

			let lua = slim_lua(&cmd.name)?;
			lua.set_hook(
				HookTriggers::new().on_calls().on_returns().every_nth_instruction(2000),
				move |_, dbg| {
					if ct1.is_cancelled() && dbg.source().what != "C" {
						Err("Spot task cancelled".into_lua_err())
					} else {
						Ok(VmState::Continue)
					}
				},
			);

			let plugin = LOADER.load_once(&lua, &cmd.name)?;
			let job = lua.create_table_from([
				("id", Id(IDS.next()).into_lua(&lua)?),
				("args", Sendable::args_to_table_ref(&lua, &cmd.args)?.into_lua(&lua)?),
				("file", File::new(file).into_lua(&lua)?),
				("mime", mime.into_lua(&lua)?),
				("skip", skip.into_lua(&lua)?),
			])?;

			if ct2.is_cancelled() { Ok(()) } else { plugin.call_async_method("spot", job).await }
		};

		let result = Handle::current().block_on(async {
			select! {
				_ = ct2.cancelled() => Ok(()),
				r = future => r,
			}
		});

		if let Err(e) = result {
			if !e.to_string().contains("Spot task cancelled") {
				error!("{e}");
			}
		}
	});

	ct
}
