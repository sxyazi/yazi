use yazi_shared::{emit, event::Cmd, fs::{expand_path, File, FilesOp, Url}, Layer};

use crate::{manager::Manager, tab::Tab};

pub struct Opt {
	target: Url,
}

impl From<Cmd> for Opt {
	fn from(mut c: Cmd) -> Self {
		let mut target = Url::from(c.take_first().unwrap_or_default());
		if target.is_regular() {
			target.set_path(expand_path(&target))
		}

		Self { target }
	}
}
impl From<Url> for Opt {
	fn from(target: Url) -> Self { Self { target } }
}

impl Tab {
	#[inline]
	pub fn _reveal(target: &Url) {
		emit!(Call(Cmd::args("reveal", vec![target.to_string()]), Layer::Manager));
	}

	pub fn reveal(&mut self, opt: impl Into<Opt>) {
		let opt = opt.into() as Opt;

		let Some(parent) = opt.target.parent_url() else {
			return;
		};

		self.cd(parent.clone());
		FilesOp::Creating(parent, vec![File::from_dummy(opt.target.clone())]).emit();
		Manager::_hover(Some(opt.target));
	}
}
