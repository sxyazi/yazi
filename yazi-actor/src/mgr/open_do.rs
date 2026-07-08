use anyhow::Result;
use hashbrown::HashMap;
use indexmap::IndexSet;
use yazi_config::{YAZI, opener::OpenerRule};
use yazi_fs::{Splatter, file::File};
use yazi_macro::succ;
use yazi_parser::mgr::OpenDoForm;
use yazi_proxy::{PickProxy, TasksProxy};
use yazi_scheduler::process::ShellOpt;
use yazi_shared::{data::Data, url::UrlBuf};

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
			succ!(Self::match_and_open(opt.cwd, targets));
		}

		let openers: IndexSet<_> =
			YAZI.open.match_common(&targets).flat_map(|r| YAZI.opener.all(r)).collect();
		if openers.is_empty() {
			succ!();
		}

		let pick = PickProxy::show(YAZI.pick.open(openers.iter().map(|o| o.desc()).collect()));
		let urls: Vec<_> = targets.into_iter().map(|(file, _)| file.url).collect();
		tokio::spawn(async move {
			if let Some(choice) = pick.await {
				Self::open_with(&openers[choice], &opt.cwd, &urls);
			}
		});
		succ!();
	}
}

impl OpenDo {
	fn match_and_open(cwd: UrlBuf, targets: Vec<(File, &str)>) {
		let mut openers: HashMap<_, Vec<_>> = Default::default();
		for (file, mime) in targets {
			if let Some(open) = YAZI.open.matches(&file, mime)
				&& let Some(opener) = YAZI.opener.first(&open)
			{
				openers.entry(opener).or_default().push(file.url);
			}
		}
		for (opener, urls) in openers {
			Self::open_with(&opener, &cwd, &urls);
		}
	}

	fn open_with(opener: &OpenerRule, cwd: &UrlBuf, urls: &[UrlBuf]) {
		let size = if opener.spread { urls.len().max(1) } else { 1 };
		for urls in urls.chunks(size) {
			TasksProxy::process_open(ShellOpt {
				cwd:    cwd.clone(),
				cmd:    Splatter::new(urls).splat(&opener.run),
				block:  opener.block,
				orphan: opener.orphan,
			});
		}
	}
}
