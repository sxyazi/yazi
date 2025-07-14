use std::{borrow::Cow, iter};

use anyhow::Result;
use tracing::error;
use yazi_config::{YAZI, popup::PickCfg};
use yazi_core::tab::Folder;
use yazi_fs::File;
use yazi_macro::{act, succ};
use yazi_parser::mgr::OpenOpt;
use yazi_plugin::isolate;
use yazi_proxy::{MgrProxy, PickProxy, TasksProxy, options::OpenDoOpt};
use yazi_shared::{MIME_DIR, event::{CmdCow, Data}, url::Url};

use crate::{Actor, Ctx, mgr::Quit};

pub struct Open;

impl Actor for Open {
	type Options = OpenOpt;

	const NAME: &'static str = "open";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		act!(mgr:escape_visual, cx)?;

		let Some(hovered) = cx.hovered().map(|h| h.url_owned()) else { succ!() };

		let mut selected =
			if opt.hovered { Box::new(iter::once(&hovered)) } else { cx.tab().selected_or_hovered() };
		if Quit::quit_with_selected(opt, &mut selected) {
			succ!();
		}

		let mut todo = vec![];
		let targets: Vec<_> = selected
			.cloned()
			.enumerate()
			.map(|(i, u)| {
				if cx.mgr.mimetype.contains(&u) {
					(u, "")
				} else if Self::guess_folder(cx, &u) {
					(u, MIME_DIR)
				} else {
					todo.push(i);
					(u, "")
				}
			})
			.collect();

		let cwd = cx.cwd().clone();
		if todo.is_empty() {
			return act!(mgr:open_do, cx, OpenDoOpt { cwd, hovered, targets, interactive: opt.interactive });
		}

		tokio::spawn(async move {
			let mut files = Vec::with_capacity(todo.len());
			for i in todo {
				if let Ok(f) = File::new(targets[i].0.clone()).await {
					files.push(f);
				}
			}

			for (fetcher, files) in YAZI.plugin.mime_fetchers(files) {
				if let Err(e) = isolate::fetch(CmdCow::from(&fetcher.run), files).await {
					error!("Fetch mime failed on opening: {e}");
				}
			}

			MgrProxy::open_do(OpenDoOpt { cwd, hovered, targets, interactive: opt.interactive });
		});
		succ!();
	}
}

// --- Do
pub struct OpenDo;

impl Actor for OpenDo {
	type Options = OpenDoOpt;

	const NAME: &'static str = "open_do";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		let mut targets = opt.targets;
		targets.iter_mut().filter(|(_, m)| m.is_empty()).for_each(|(u, m)| {
			*m = cx.mgr.mimetype.by_url(u).unwrap_or_default();
		});

		targets.retain(|(_, m)| !m.is_empty());
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

impl Open {
	fn guess_folder(cx: &Ctx, url: &Url) -> bool {
		let Some(p) = url.parent_url() else {
			return true;
		};

		let find = |folder: Option<&Folder>| {
			folder.is_some_and(|folder| {
				p == folder.url && folder.files.iter().any(|f| f.is_dir() && f.urn() == url.urn())
			})
		};

		find(Some(cx.current()))
			|| find(cx.parent())
			|| find(cx.hovered_folder())
			|| find(cx.tab().history.get(&p))
	}
}
