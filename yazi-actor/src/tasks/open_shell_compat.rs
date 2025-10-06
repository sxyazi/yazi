use anyhow::Result;
use yazi_macro::succ;
use yazi_parser::tasks::ProcessOpenOpt;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct OpenShellCompat;

// TODO: remove
impl Actor for OpenShellCompat {
	type Options = ProcessOpenOpt;

	const NAME: &str = "open_shell_compat";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		succ!(cx.tasks.open_shell_compat(opt));
	}
}
