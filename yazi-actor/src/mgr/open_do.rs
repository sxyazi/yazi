use anyhow::Result;
use hashbrown::HashMap;
use yazi_config::{YAZI, popup::PickCfg};
use yazi_macro::succ;
use yazi_parser::mgr::OpenDoForm;
use yazi_proxy::{PickProxy, TasksProxy};
use yazi_scheduler::process::ProcessOpt;
use yazi_shared::{data::Data, url::UrlCow};

use crate::{Actor, Ctx};

pub struct OpenDo;

impl Actor for OpenDo {
	type Form = OpenDoForm;

	const NAME: &str = "open_do";

	fn act(cx: &mut Ctx, Self::Form { opt }: Self::Form) -> Result<Data> {
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
			succ!(Self::match_and_open(cx, opt.cwd, targets));
		}

		let openers: Vec<_> = YAZI.opener.all(YAZI.open.common(&targets).into_iter()).collect();
		if openers.is_empty() {
			succ!();
		}

		let pick = PickProxy::show(PickCfg::open(openers.iter().map(|o| o.desc()).collect()));
		let urls: Vec<_> =
			[UrlCow::default()].into_iter().chain(targets.into_iter().map(|(u, _)| u)).collect();
		tokio::spawn(async move {
			if let Some(choice) = pick.await {
				TasksProxy::open_shell_compat(ProcessOpt {
					cwd:    opt.cwd,
					cmd:    openers[choice].run.clone().into(),
					args:   urls,
					block:  openers[choice].block,
					orphan: openers[choice].orphan,
					spread: openers[choice].spread,
				});
			}
		});
		succ!();
	}
}

impl OpenDo {
	// TODO: remove
	fn match_and_open(cx: &Ctx, cwd: UrlCow<'static>, targets: Vec<(UrlCow<'static>, &str)>) {
		let mut openers = HashMap::new();
		for (url, mime) in targets {
			if let Some(opener) = YAZI.opener.first(YAZI.open.all(&url, mime)) {
				openers.entry(opener).or_insert_with(|| vec![UrlCow::default()]).push(url);
			}
		}
		for (opener, args) in openers {
			cx.tasks.open_shell_compat(ProcessOpt {
				cwd: cwd.clone(),
				cmd: opener.run.clone().into(),
				args,
				block: opener.block,
				orphan: opener.orphan,
				spread: opener.spread,
			});
		}
	}
}
