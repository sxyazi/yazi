use std::{borrow::Cow, iter};

use tracing::error;
use yazi_config::{YAZI, popup::PickCfg};
use yazi_fs::File;
use yazi_parser::mgr::OpenOpt;
use yazi_plugin::isolate;
use yazi_proxy::{MgrProxy, PickProxy, TasksProxy, options::OpenDoOpt};
use yazi_shared::{MIME_DIR, event::CmdCow, url::Url};

use crate::{mgr::Mgr, tab::Folder, tasks::Tasks};

impl Mgr {
	#[yazi_codegen::command]
	pub fn open(&mut self, opt: OpenOpt, tasks: &Tasks) {
		if !self.active_mut().try_escape_visual() {
			return;
		}
		let Some(hovered) = self.hovered().map(|h| h.url_owned()) else {
			return;
		};

		let mut selected =
			if opt.hovered { Box::new(iter::once(&hovered)) } else { self.selected_or_hovered() };
		if Self::quit_with_selected(opt, &mut selected) {
			return;
		}

		let mut todo = vec![];
		let targets: Vec<_> = selected
			.cloned()
			.enumerate()
			.map(|(i, u)| {
				if self.mimetype.contains(&u) {
					(u, "")
				} else if self.guess_folder(&u) {
					(u, MIME_DIR)
				} else {
					todo.push(i);
					(u, "")
				}
			})
			.collect();

		let cwd = self.cwd().clone();
		if todo.is_empty() {
			return self
				.open_do(OpenDoOpt { cwd, hovered, targets, interactive: opt.interactive }, tasks);
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
	}

	#[yazi_codegen::command]
	pub fn open_do(&mut self, opt: OpenDoOpt, tasks: &Tasks) {
		let mut targets = opt.targets;
		targets.iter_mut().filter(|(_, m)| m.is_empty()).for_each(|(u, m)| {
			*m = self.mimetype.by_url(u).unwrap_or_default();
		});

		targets.retain(|(_, m)| !m.is_empty());
		if targets.is_empty() {
			return;
		} else if !opt.interactive {
			return tasks.process_from_files(opt.cwd, opt.hovered, targets);
		}

		let openers: Vec<_> = YAZI.opener.all(YAZI.open.common(&targets).into_iter());
		if openers.is_empty() {
			return;
		}

		let pick = PickProxy::show(PickCfg::open(openers.iter().map(|o| o.desc()).collect()));
		let urls = [opt.hovered].into_iter().chain(targets.into_iter().map(|(u, _)| u)).collect();
		tokio::spawn(async move {
			if let Ok(choice) = pick.await {
				TasksProxy::open_with(Cow::Borrowed(openers[choice]), opt.cwd, urls);
			}
		});
	}

	fn guess_folder(&self, url: &Url) -> bool {
		let Some(p) = url.parent_url() else {
			return true;
		};

		let find = |folder: Option<&Folder>| {
			folder.is_some_and(|folder| {
				p == folder.url && folder.files.iter().any(|f| f.is_dir() && f.urn() == url.urn())
			})
		};

		find(Some(self.current()))
			|| find(self.parent())
			|| find(self.hovered_folder())
			|| find(self.active().history.get(&p))
	}
}
