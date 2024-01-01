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
	// TODO: refactor this
	fn handle_read(&mut self, op: FilesOp) {
		let url = op.url().clone();
		let cwd = self.cwd().to_owned();
		let hovered = self.hovered().map(|h| h.url());

		if cwd == url {
			render!(self.current_mut().update(op));
		} else if matches!(self.parent(), Some(p) if p.cwd == url) {
			render!(self.active_mut().parent.as_mut().unwrap().update(op));
		} else if matches!(self.hovered(), Some(h) if h.url == url) {
			self.active_mut().history.entry(url.clone()).or_insert_with(|| Folder::from(&url));
			self.active_mut().apply_files_attrs(true);
			render!(self.active_mut().history.get_mut(&url).unwrap().update(op));
			self.peek(true);
		} else {
			self.active_mut().history.entry(url.clone()).or_insert_with(|| Folder::from(&url)).update(op);
		}

		render!(self.active_mut().parent.as_mut().is_some_and(|p| p.hover(&cwd)));
		render!(hovered.as_ref().is_some_and(|h| self.current_mut().hover(h)));

		if hovered.as_ref() != self.hovered().map(|h| &h.url) {
			self.hover(None);
		}
	}

	fn handle_ioerr(&mut self, op: FilesOp) {
		let url = op.url();
		let op = FilesOp::Full(url.clone(), vec![]);

		if url == self.cwd() {
			self.current_mut().update(op);
			self.active_mut().leave(());
			render!();
		} else if matches!(self.parent(), Some(p) if &p.cwd == url) {
			render!(self.active_mut().parent.as_mut().unwrap().update(op));
		}
	}

	pub fn update_files(&mut self, opt: impl TryInto<Opt>, tasks: &Tasks) {
		let Ok(opt) = opt.try_into() else {
			return;
		};
		let calc = !matches!(opt.op, FilesOp::Size(..) | FilesOp::IOErr(_) | FilesOp::Deleting(..));

		let mut ops = vec![opt.op];
		for u in self.watcher.linked.read().from_dir(ops[0].url()) {
			ops.push(ops[0].chroot(u));
		}

		for op in ops {
			match op {
				FilesOp::IOErr(..) => self.handle_ioerr(op),
				_ => self.handle_read(op),
			};
		}

		if calc {
			tasks.preload_sorted(&self.current().files);
		}
	}
}
