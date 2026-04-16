use anyhow::Result;
use futures::StreamExt;
use hashbrown::HashSet;
use yazi_boot::ARGS;
use yazi_core::mgr::OpenDoOpt;
use yazi_fs::File;
use yazi_macro::{act, succ};
use yazi_parser::mgr::OpenForm;
use yazi_proxy::MgrProxy;
use yazi_shared::data::Data;
use yazi_vfs::VfsFile;

use crate::{Actor, Ctx, mgr::Quit};

pub struct Open;

impl Actor for Open {
	type Form = OpenForm;

	const NAME: &str = "open";

	fn act(cx: &mut Ctx, Self::Form { mut opt }: Self::Form) -> Result<Data> {
		if !opt.interactive && ARGS.chooser_file.is_some() {
			succ!(if !opt.targets.is_empty() {
				Quit::with_selected(opt.targets)
			} else if opt.hovered {
				Quit::with_selected(cx.hovered().map(|h| &h.url))
			} else {
				act!(mgr:escape_visual, cx)?;
				Quit::with_selected(cx.tab().selected_or_hovered())
			});
		}

		if opt.targets.is_empty() {
			opt.targets = if opt.hovered {
				cx.hovered().map(|h| vec![h.url.clone()]).unwrap_or_default()
			} else {
				act!(mgr:escape_visual, cx)?;
				cx.tab().selected_or_hovered().cloned().collect()
			};
		}
		if opt.targets.is_empty() {
			succ!();
		}

		let todo: HashSet<_> = opt
			.targets
			.iter()
			.enumerate()
			.filter(|&(_, u)| !cx.mgr.mimetype.contains(u))
			.map(|(i, _)| i)
			.collect();

		let cwd = opt.cwd.unwrap_or_else(|| cx.cwd().clone());
		let scheduler = cx.tasks.scheduler.clone();
		tokio::spawn(async move {
			let mut all = Vec::with_capacity(opt.targets.len());
			let mut part = Vec::with_capacity(todo.len());

			let it = futures::stream::iter(opt.targets)
				.enumerate()
				.map(|(i, url)| async move { File::new(url).await.ok().map(|file| (i, file)) })
				.buffered(3)
				.filter_map(|item| async move { item });

			futures::pin_mut!(it);
			while let Some((i, file)) = it.next().await {
				if todo.contains(&i) {
					part.push(file.clone());
				}
				all.push(file);
			}

			if !all.is_empty() && scheduler.fetch_mimetype(part).await {
				MgrProxy::open_do(OpenDoOpt { cwd, targets: all, interactive: opt.interactive });
			}
		});
		succ!();
	}
}
