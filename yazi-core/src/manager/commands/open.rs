use std::ffi::OsString;

use tracing::error;
use yazi_config::{popup::SelectCfg, ARGS, OPEN};
use yazi_plugin::isolate;
use yazi_shared::{emit, event::{EventQuit, Exec}, fs::{File, Url}, Layer, MIME_DIR};

use crate::{manager::Manager, select::Select, tasks::Tasks};

pub struct Opt {
	targets:     Option<Vec<(Url, Option<String>)>>,
	interactive: bool,
}

impl From<Exec> for Opt {
	fn from(mut e: Exec) -> Self {
		Self { targets: e.take_data(), interactive: e.named.contains_key("interactive") }
	}
}

impl Manager {
	pub fn open(&mut self, opt: impl Into<Opt>, tasks: &Tasks) {
		let selected = self.selected();
		if selected.is_empty() {
			return;
		} else if Self::quit_with_selected(&selected) {
			return;
		}

		let (mut done, mut todo) = (Vec::with_capacity(selected.len()), vec![]);
		for f in selected {
			if f.is_dir() {
				done.push((f.url(), Some(MIME_DIR.to_owned())));
			} else if self.mimetype.get(&f.url).is_some() {
				done.push((f.url(), None));
			} else {
				todo.push(f.clone());
			}
		}

		let mut opt = opt.into() as Opt;
		if todo.is_empty() {
			opt.targets = Some(done);
			return self.open_do(opt, tasks);
		}

		tokio::spawn(async move {
			done.extend(todo.iter().map(|f| (f.url(), None)));
			if let Err(e) = isolate::preload("mime", todo, true).await {
				error!("preload in watcher failed: {e}");
			}

			Self::_open_do(opt.interactive, done);
		});
	}

	#[inline]
	pub fn _open_do(interactive: bool, targets: Vec<(Url, Option<String>)>) {
		emit!(Call(
			Exec::call("open_do", vec![]).with_bool("interactive", interactive).with_data(targets),
			Layer::Manager
		));
	}

	pub fn open_do(&mut self, opt: impl Into<Opt>, tasks: &Tasks) {
		let opt = opt.into() as Opt;
		let Some(targets) = opt.targets else {
			return;
		};

		let targets: Vec<_> = targets
			.into_iter()
			.filter_map(|(u, m)| m.or_else(|| self.mimetype.get(&u).cloned()).map(|m| (u, m)))
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

	fn quit_with_selected(selected: &[&File]) -> bool {
		if ARGS.chooser_file.is_none() {
			return false;
		}

		let paths = selected.iter().fold(OsString::new(), |mut s, &f| {
			s.push(f.url.as_os_str());
			s.push("\n");
			s
		});

		emit!(Quit(EventQuit { selected: Some(paths), ..Default::default() }));
		true
	}
}
