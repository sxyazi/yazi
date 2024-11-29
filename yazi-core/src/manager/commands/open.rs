use std::{borrow::Cow, ffi::OsString};

use tracing::error;
use yazi_boot::ARGS;
use yazi_config::{OPEN, popup::PickCfg};
use yazi_fs::Folder;
use yazi_macro::emit;
use yazi_plugin::isolate;
use yazi_proxy::{ManagerProxy, TasksProxy, options::OpenDoOpt};
use yazi_shared::{MIME_DIR, event::{Cmd, CmdCow, EventQuit}, fs::{File, Url}};

use crate::{manager::Manager, tasks::Tasks};

#[derive(Clone, Copy)]
struct Opt {
	interactive: bool,
	hovered:     bool,
}

impl From<CmdCow> for Opt {
	fn from(c: CmdCow) -> Self {
		Self { interactive: c.bool("interactive"), hovered: c.bool("hovered") }
	}
}

impl Manager {
	#[yazi_codegen::command]
	pub fn open(&mut self, opt: Opt, tasks: &Tasks) {
		if !self.active_mut().try_escape_visual() {
			return;
		}
		let Some(hovered) = self.hovered().map(|h| h.url_owned()) else {
			return;
		};

		let selected =
			if opt.hovered { vec![&hovered] } else { self.selected_or_hovered(true).collect() };

		if Self::quit_with_selected(opt, &selected) {
			return;
		}

		let (mut done, mut todo) = (Vec::with_capacity(selected.len()), vec![]);
		for u in selected {
			if self.mimetype.contains(u) {
				done.push((u.clone(), String::new()));
			} else if self.guess_folder(u) {
				done.push((u.clone(), MIME_DIR.to_owned()));
			} else {
				todo.push(u.clone());
			}
		}

		if todo.is_empty() {
			return self
				.open_do(OpenDoOpt { hovered, targets: done, interactive: opt.interactive }, tasks);
		}

		tokio::spawn(async move {
			let mut files = Vec::with_capacity(todo.len());
			for u in todo {
				if let Ok(f) = File::from(u).await {
					files.push(f);
				}
			}

			done.extend(files.iter().map(|f| (f.url_owned(), String::new())));
			if let Err(e) = isolate::fetch(Cmd::new("mime").into(), files).await {
				error!("Fetch `mime` failed in opening: {e}");
			}

			ManagerProxy::open_do(OpenDoOpt { hovered, targets: done, interactive: opt.interactive });
		});
	}

	#[yazi_codegen::command]
	pub fn open_do(&mut self, opt: OpenDoOpt, tasks: &Tasks) {
		let targets: Vec<_> = opt
			.targets
			.into_iter()
			.filter_map(|(u, m)| {
				Some(m).filter(|m| !m.is_empty()).or_else(|| self.mimetype.get_owned(&u)).map(|m| (u, m))
			})
			.collect();

		if targets.is_empty() {
			return;
		} else if !opt.interactive {
			return tasks.process_from_files(opt.hovered, targets);
		}

		let openers: Vec<_> = OPEN.common_openers(&targets);
		if openers.is_empty() {
			return;
		}

		let urls = [opt.hovered].into_iter().chain(targets.into_iter().map(|(u, _)| u)).collect();
		tokio::spawn(async move {
			let result = yazi_proxy::PickProxy::show(PickCfg::open(
				openers.iter().map(|o| o.desc.clone()).collect(),
			));
			if let Ok(choice) = result.await {
				TasksProxy::open_with(urls, Cow::Borrowed(openers[choice]));
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

	fn quit_with_selected(opt: Opt, selected: &[&Url]) -> bool {
		if opt.interactive || ARGS.chooser_file.is_none() {
			return false;
		}

		let paths = selected.iter().fold(OsString::new(), |mut s, &u| {
			s.push(u.as_os_str());
			s.push("\n");
			s
		});

		emit!(Quit(EventQuit { selected: Some(paths), ..Default::default() }));
		true
	}
}
