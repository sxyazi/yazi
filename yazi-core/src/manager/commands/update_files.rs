use yazi_shared::{event::Exec, fs::FilesOp, render};

use crate::{folder::Folder, manager::Manager, tasks::Tasks};

pub struct Opt {
	op: FilesOp,
}

impl TryFrom<&Exec> for Opt {
	type Error = ();

	fn try_from(e: &Exec) -> Result<Self, Self::Error> { Ok(Self { op: e.take_data().ok_or(())? }) }
}

impl Manager {
	fn update_parent(&mut self, op: FilesOp) {
		let cwd = self.cwd().clone();
		let leave = matches!(op, FilesOp::Deleting(_, ref urls) if urls.contains(&cwd));

		if let Some(p) = self.active_mut().parent.as_mut() {
			render!(p.update(op));
			render!(p.hover(&cwd));
		}

		if leave {
			self.active_mut().leave(());
		}
	}

	fn update_current(&mut self, op: FilesOp, tasks: &Tasks) {
		let hovered = self.hovered().map(|h| h.url());
		let calc = !matches!(op, FilesOp::Size(..) | FilesOp::Deleting(..));

		render!(self.current_mut().update(op));
		render!(hovered.as_ref().is_some_and(|h| self.current_mut().hover(h)));

		if hovered.as_ref() != self.hovered().map(|h| &h.url) {
			self.hover(None);
		}
		if calc {
			tasks.preload_sorted(&self.current().files);
		}
	}

	fn update_hovered(&mut self, op: FilesOp) {
		let url = op.url();
		self.active_mut().history.entry(url.clone()).or_insert_with(|| Folder::from(url)).update(op);

		self.peek(true);
	}

	fn update_history(&mut self, op: FilesOp) {
		let leave = self.parent().and_then(|f| f.cwd.parent_url().map(|p| (&f.cwd, p))).is_some_and(
			|(p, pp)| matches!(op, FilesOp::Deleting(ref parent, ref urls) if *parent == pp && urls.contains(p)),
		);

		let url = op.url();
		self.active_mut().history.entry(url.clone()).or_insert_with(|| Folder::from(url)).update(op);

		if leave {
			self.active_mut().leave(());
		}
	}

	pub fn update_files(&mut self, opt: impl TryInto<Opt>, tasks: &Tasks) {
		let Ok(opt) = opt.try_into() else {
			return;
		};

		let mut ops = vec![opt.op];
		for u in self.watcher.linked.read().from_dir(ops[0].url()) {
			ops.push(ops[0].chroot(u));
		}

		for op in ops {
			let url = op.url();
			if self.cwd() == url {
				self.update_current(op, tasks);
			} else if matches!(self.parent(), Some(p) if p.cwd == *url) {
				self.update_parent(op);
			} else if matches!(self.hovered(), Some(h) if h.url == *url) {
				self.update_hovered(op);
			} else {
				self.update_history(op);
			}
		}

		self.active_mut().apply_files_attrs();
	}
}
