use std::borrow::Cow;

use anyhow::Result;
use yazi_config::{YAZI, popup::PickCfg};
use yazi_macro::succ;
use yazi_parser::mgr::OpenDoOpt;
use yazi_proxy::{PickProxy, TasksProxy};
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct OpenDo;

impl Actor for OpenDo {
	type Options = OpenDoOpt;

	const NAME: &str = "open_do";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		let targets: Vec<_> = opt
			.targets
			.into_iter()
			.map(|u| {
				let m = cx.mgr.mimetype.get(&u).unwrap_or_default();
				(u, m)
			})
			.filter(|(_, m)| !m.is_empty())
			.collect();

		if targets.is_empty() {
			succ!();
		} else if !opt.interactive {
			succ!(cx.tasks.process_from_files(opt.cwd, opt.hovered, targets));
		}

		let openers: Vec<_> = YAZI.opener.all(YAZI.open.common(&targets).into_iter());
		if openers.is_empty() {
			succ!();
		}

		let pick = PickProxy::show(PickCfg::open(openers.iter().map(|o| o.desc()).collect()));
		let urls = [opt.hovered].into_iter().chain(targets.into_iter().map(|(u, _)| u)).collect();
		tokio::spawn(async move {
			if let Ok(choice) = pick.await {
				TasksProxy::open_with(Cow::Borrowed(openers[choice]), opt.cwd, urls);
			}
		});
		succ!();
	}
}
