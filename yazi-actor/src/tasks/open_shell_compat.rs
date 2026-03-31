use anyhow::Result;
use yazi_macro::succ;
use yazi_parser::tasks::ProcessOpenForm;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct OpenShellCompat;

// TODO: remove
impl Actor for OpenShellCompat {
	type Form = ProcessOpenForm;

	const NAME: &str = "open_shell_compat";

	fn act(cx: &mut Ctx, Self::Form { opt, .. }: Self::Form) -> Result<Data> {
		succ!(cx.tasks.open_shell_compat(opt));
	}
}
