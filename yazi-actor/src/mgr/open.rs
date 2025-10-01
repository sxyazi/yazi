use anyhow::Result;
use yazi_boot::ARGS;
use yazi_fs::File;
use yazi_macro::{act, succ};
use yazi_parser::mgr::{OpenDoOpt, OpenOpt};
use yazi_proxy::MgrProxy;
use yazi_shared::{data::Data, url::UrlCow};
use yazi_vfs::VfsFile;

use crate::{Actor, Ctx, mgr::Quit};

pub struct Open;

impl Actor for Open {
	type Options = OpenOpt;

	const NAME: &str = "open";

	fn act(cx: &mut Ctx, mut opt: Self::Options) -> Result<Data> {
		act!(mgr:escape_visual, cx)?;

		let Some(hovered) = cx.hovered().map(|h| h.url_owned()).map(UrlCow::from) else { succ!() };

		if !opt.interactive && ARGS.chooser_file.is_some() {
			succ!(if !opt.targets.is_empty() {
				Quit::with_selected(opt.targets)
			} else if opt.hovered {
				Quit::with_selected([hovered])
			} else {
				Quit::with_selected(cx.tab().selected_or_hovered())
			});
		}

		if opt.targets.is_empty() {
			opt.targets = if opt.hovered {
				vec![hovered.clone()]
			} else {
				cx.tab().selected_or_hovered().cloned().map(Into::into).collect()
			};
		}

		let todo: Vec<_> = opt
			.targets
			.iter()
			.enumerate()
			.filter(|&(_, u)| !cx.mgr.mimetype.contains(u))
			.map(|(i, _)| i)
			.collect();

		let cwd = cx.cwd().clone();
		if todo.is_empty() {
			return act!(mgr:open_do, cx, OpenDoOpt { cwd, hovered, targets: opt.targets, interactive: opt.interactive });
		}

		let scheduler = cx.tasks.scheduler.clone();
		tokio::spawn(async move {
			let mut files = Vec::with_capacity(todo.len());
			for i in todo {
				if let Ok(f) = File::new(&opt.targets[i]).await {
					files.push(f);
				}
			}
			if scheduler.fetch_mimetype(files).await {
				MgrProxy::open_do(OpenDoOpt {
					cwd,
					hovered,
					targets: opt.targets,
					interactive: opt.interactive,
				});
			}
		});
		succ!();
	}
}
