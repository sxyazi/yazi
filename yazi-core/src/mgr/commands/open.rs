use std::borrow::Cow;

use tracing::error;
use yazi_config::{YAZI, popup::PickCfg};
use yazi_fs::File;
use yazi_plugin::isolate;
use yazi_proxy::{MgrProxy, PickProxy, TasksProxy, options::OpenDoOpt};
use yazi_shared::{MIME_DIR, event::CmdCow, url::Url};

use crate::{mgr::Mgr, tab::Folder, tasks::Tasks};

#[derive(Clone, Copy)]
pub(super) struct Opt {
	pub(super) interactive: bool,
	pub(super) hovered:     bool,
}

impl From<CmdCow> for Opt {
	fn from(c: CmdCow) -> Self {
		Self { interactive: c.bool("interactive"), hovered: c.bool("hovered") }
	}
}

impl Mgr {
	#[yazi_codegen::command]
	pub fn open(&mut self, opt: Opt, tasks: &Tasks) {
		if !self.active_mut().try_escape_visual() {
			return;
		}
		let Some(hovered) = self.hovered().map(|h| h.url_owned()) else {
			return;
		};

		let selected = if opt.hovered { vec![&hovered] } else { self.selected_or_hovered().collect() };
		if Self::quit_with_selected(opt, &selected) {
			return;
		}

		let cwd = self.cwd().clone();
		let (mut done, mut todo) = (Vec::with_capacity(selected.len()), vec![]);
		for u in selected {
			if self.mimetype.contains(u) {
				done.push((u.clone(), ""));
			} else if self.guess_folder(u) {
				done.push((u.clone(), MIME_DIR));
			} else {
				todo.push(u.clone());
			}
		}

		if todo.is_empty() {
			return self
				.open_do(OpenDoOpt { cwd, hovered, targets: done, interactive: opt.interactive }, tasks);
		}

		tokio::spawn(async move {
			let mut files = Vec::with_capacity(todo.len());
			for u in todo {
				if let Ok(f) = File::new(u).await {
					files.push(f);
				}
			}

			done.extend(files.iter().map(|f| (f.url_owned(), "")));
			for (fetcher, files) in YAZI.plugin.mime_fetchers(files) {
				if let Err(e) = isolate::fetch(CmdCow::from(&fetcher.run), files).await {
					error!("Fetch mime failed on opening: {e}");
				}
			}

			MgrProxy::open_do(OpenDoOpt { cwd, hovered, targets: done, interactive: opt.interactive });
		});
	}

	#[yazi_codegen::command]
	pub fn open_do(&mut self, opt: OpenDoOpt, tasks: &Tasks) {
		let targets: Vec<_> = opt
			.targets
			.into_iter()
			.filter_map(|(u, m)| {
				Some(m).filter(|m| !m.is_empty()).or_else(|| self.mimetype.by_url(&u)).map(|m| (u, m))
			})
			.collect();

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
