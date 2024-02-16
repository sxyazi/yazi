use std::ffi::OsString;

use tracing::error;
use yazi_config::{popup::SelectCfg, ARGS, OPEN};
use yazi_plugin::isolate;
use yazi_shared::{emit, event::{Cmd, EventQuit}, fs::{File, Url}, Layer, MIME_DIR};

use crate::{folder::Folder, manager::Manager, select::Select, tasks::Tasks};

pub struct Opt {
	targets:     Vec<(Url, String)>,
	interactive: bool,
	hovered:     bool,
}

impl From<Cmd> for Opt {
	fn from(mut c: Cmd) -> Self {
		Self {
			targets:     c.take_data().unwrap_or_default(),
			interactive: c.named.contains_key("interactive"),
			hovered:     c.named.contains_key("hovered"),
		}
	}
}

impl Manager {
	pub fn open(&mut self, opt: impl Into<Opt>, tasks: &Tasks) {
		let mut opt = opt.into() as Opt;

		let selected = if opt.hovered {
			self.hovered().map(|h| vec![&h.url]).unwrap_or_default()
		} else {
			self.selected_or_hovered()
		};

		if selected.is_empty() {
			return;
		} else if Self::quit_with_selected(&selected) {
			return;
		}

		let (mut done, mut todo) = (Vec::with_capacity(selected.len()), vec![]);
		for u in selected {
			if self.mimetype.get(u).is_some() {
				done.push((u.clone(), String::new()));
			} else if self.guess_folder(u) {
				done.push((u.clone(), MIME_DIR.to_owned()));
			} else {
				todo.push(u.clone());
			}
		}

		if todo.is_empty() {
			opt.targets = done;
			return self.open_do(opt, tasks);
		}

		tokio::spawn(async move {
			let mut files = Vec::with_capacity(todo.len());
			for u in todo {
				if let Ok(f) = File::from(u).await {
					files.push(f);
				}
			}

			done.extend(files.iter().map(|f| (f.url(), String::new())));
			if let Err(e) = isolate::preload("mime", files, true).await {
				error!("preload in watcher failed: {e}");
			}

			Self::_open_do(opt.interactive, done);
		});
	}

	#[inline]
	pub fn _open_do(interactive: bool, targets: Vec<(Url, String)>) {
		emit!(Call(
			Cmd::new("open_do").with_bool("interactive", interactive).with_data(targets),
			Layer::Manager
		));
	}

	pub fn open_do(&mut self, opt: impl Into<Opt>, tasks: &Tasks) {
		let opt = opt.into() as Opt;
		if opt.targets.is_empty() {
			return;
		}

		let targets: Vec<_> = opt
			.targets
			.into_iter()
			.filter_map(|(u, m)| {
				Some(m).filter(|m| !m.is_empty()).or_else(|| self.mimetype.get(&u).cloned()).map(|m| (u, m))
			})
			.collect();

		if targets.is_empty() {
			return;
		} else if !opt.interactive {
			tasks.file_open(&targets);
			return;
		}

		let openers: Vec<_> = OPEN.common_openers(&targets).into_iter().cloned().collect();
		if openers.is_empty() {
			return;
		}

		let urls = targets.into_iter().map(|(u, _)| u).collect();
		tokio::spawn(async move {
			let result = Select::_show(SelectCfg::open(openers.iter().map(|o| o.desc.clone()).collect()));
			if let Ok(choice) = result.await {
				Tasks::_open(urls, openers[choice].clone());
			}
		});
	}

	fn guess_folder(&self, url: &Url) -> bool {
		let Some(p) = url.parent_url() else {
			return true;
		};

		let find = |folder: Option<&Folder>| {
			folder.is_some_and(|folder| {
				folder.cwd == p && folder.files.iter().any(|f| f.is_dir() && f.url == *url)
			})
		};

		find(Some(self.current()))
			|| find(self.parent())
			|| find(self.hovered_folder())
			|| find(self.active().history.get(&p))
	}

	fn quit_with_selected(selected: &[&Url]) -> bool {
		if ARGS.chooser_file.is_none() {
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
