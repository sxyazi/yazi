use anyhow::Result;
use hashbrown::HashMap;
use indexmap::IndexSet;
use yazi_config::{YAZI, popup::PickCfg};
use yazi_macro::succ;
use yazi_parser::mgr::OpenDoForm;
use yazi_proxy::{PickProxy, TasksProxy};
use yazi_scheduler::process::ProcessOpt;
use yazi_shared::{data::Data, url::{UrlBuf, UrlCow}};

use crate::{Actor, Ctx};

pub struct OpenDo;

impl Actor for OpenDo {
	type Form = OpenDoForm;

	const NAME: &str = "open_do";

	fn act(cx: &mut Ctx, Self::Form { opt }: Self::Form) -> Result<Data> {
		let targets: Vec<_> = opt
			.targets
			.into_iter()
			.map(|file| {
				let mime = cx.mgr.mimetype.get(&file.url).unwrap_or_default();
				(file, mime)
			})
			.filter(|(_, m)| !m.is_empty())
			.collect();

		if targets.is_empty() {
			succ!();
		} else if !opt.interactive {
			succ!(Self::match_and_open(cx, opt.cwd, targets));
		}

		let openers: IndexSet<_> =
			YAZI.open.match_common(&targets).flat_map(|r| YAZI.opener.all(r)).collect();
		if openers.is_empty() {
			succ!();
		}

		let pick = PickProxy::show(PickCfg::open(openers.iter().map(|o| o.desc()).collect()));
		let urls: Vec<_> = [UrlCow::default()]
			.into_iter()
			.chain(targets.into_iter().map(|(file, _)| file.url.into()))
			.collect();
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
	fn match_and_open(cx: &Ctx, cwd: UrlBuf, targets: Vec<(yazi_fs::File, &str)>) {
		let mut openers = HashMap::new();
		for (file, mime) in targets {
			if let Some(open) = YAZI.open.matches(&file, mime)
				&& let Some(opener) = YAZI.opener.first(&open)
			{
				openers.entry(opener).or_insert_with(|| vec![UrlCow::default()]).push(file.url.into());
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
