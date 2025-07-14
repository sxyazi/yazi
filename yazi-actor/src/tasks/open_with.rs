use anyhow::Result;
use yazi_macro::succ;
use yazi_proxy::options::OpenWithOpt;
use yazi_shared::event::Data;

use crate::{Actor, Ctx};

pub struct OpenWith;

impl Actor for OpenWith {
	type Options = OpenWithOpt;

	const NAME: &'static str = "open_with";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		succ!(cx.tasks.process_from_opener(
			opt.cwd,
			opt.opener,
			opt.targets.into_iter().map(|u| u.into_path().into_os_string()).collect(),
		));
	}
}
