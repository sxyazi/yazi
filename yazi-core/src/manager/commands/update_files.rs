use yazi_shared::{event::Exec, fs::FilesOp};

use crate::{folder::Folder, manager::Manager, tasks::Tasks};

pub struct Opt {
	op: FilesOp,
}

impl TryFrom<&Exec> for Opt {
	type Error = ();

	fn try_from(e: &Exec) -> Result<Self, Self::Error> { Ok(Self { op: e.take_data().ok_or(())? }) }
}

impl Manager {
	fn handle_read(&mut self, op: FilesOp) -> bool {
		let url = op.url().clone();
		let cwd = self.cwd().to_owned();
		let hovered = self.hovered().map(|h| h.url());

		let mut b = if cwd == url {
			self.current_mut().update(op)
		} else if matches!(self.parent(), Some(p) if p.cwd == url) {
			self.active_mut().parent.as_mut().unwrap().update(op)
		} else if matches!(self.hovered(), Some(h) if h.url == url) {
			self.active_mut().history.entry(url.clone()).or_insert_with(|| Folder::from(&url));
			self.active_mut().apply_files_attrs(true);
			self.active_mut().history.get_mut(&url).unwrap().update(op)
		} else {
			self.active_mut().history.entry(url.clone()).or_insert_with(|| Folder::from(&url)).update(op);
			false
		};

		b |= self.active_mut().parent.as_mut().is_some_and(|p| p.hover(&cwd));
		b |= hovered.as_ref().is_some_and(|h| self.current_mut().hover(h));

		if hovered.as_ref() != self.hovered().map(|h| &h.url) {
			b |= self.hover(None);
		}
		b
	}

	fn handle_ioerr(&mut self, op: FilesOp) -> bool {
		let url = op.url();
		let op = FilesOp::Full(url.clone(), Vec::new());

		if url == self.cwd() {
			self.current_mut().update(op);
			self.active_mut().leave(());
			true
		} else if matches!(self.parent(), Some(p) if &p.cwd == url) {
			self.active_mut().parent.as_mut().unwrap().update(op)
		} else {
			false
		}
	}

	pub fn update_files(&mut self, opt: impl TryInto<Opt>, tasks: &Tasks) -> bool {
		let Ok(opt) = opt.try_into() else {
			return false;
		};

		let calc = !matches!(opt.op, FilesOp::Size(..) | FilesOp::IOErr(_) | FilesOp::Deleting(..));
		let b = match opt.op {
			FilesOp::IOErr(..) => self.handle_ioerr(opt.op),
			_ => self.handle_read(opt.op),
		};

		if calc {
			tasks.preload_sorted(&self.current().files);
		}

		b
	}
}
