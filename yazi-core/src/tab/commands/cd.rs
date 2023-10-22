use std::mem;

use yazi_shared::Url;

use crate::{emit, files::{File, FilesOp}, input::InputOpt, tab::Tab};

impl Tab {
	// TODO: change to sync, and remove `Event::Cd`
	pub async fn cd(&mut self, mut target: Url) -> bool {
		let Ok(file) = File::from(target.clone()).await else {
			return false;
		};

		let mut hovered = None;
		if !file.is_dir() {
			hovered = Some(file.url());
			target = target.parent_url().unwrap();
			emit!(Files(FilesOp::Creating(target.clone(), file.into_map())));
		}

		// Already in target
		if self.current.cwd == target {
			if let Some(h) = hovered {
				emit!(Hover(h));
			}
			return false;
		}

		// Take parent to history
		if let Some(rep) = self.parent.take() {
			self.history.insert(rep.cwd.clone(), rep);
		}

		// Current
		let rep = self.history_new(&target);
		let rep = mem::replace(&mut self.current, rep);
		if rep.cwd.is_regular() {
			self.history.insert(rep.cwd.clone(), rep);
		}

		// Parent
		if let Some(parent) = target.parent_url() {
			self.parent = Some(self.history_new(&parent));
		}

		// Hover the file
		if let Some(h) = hovered {
			emit!(Hover(h));
		}

		// Backstack
		if target.is_regular() {
			self.backstack.push(target.clone());
		}

		emit!(Refresh);
		true
	}

	pub fn cd_interactive(&mut self, target: Url) -> bool {
		tokio::spawn(async move {
			let mut result =
				emit!(Input(InputOpt::top("Change directory:").with_value(target.to_string_lossy())));

			if let Some(Ok(s)) = result.recv().await {
				emit!(Cd(Url::from(s.trim())));
			}
		});
		false
	}
}
