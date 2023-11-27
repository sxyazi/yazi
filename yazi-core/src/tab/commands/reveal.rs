use yazi_config::keymap::Exec;
use yazi_shared::{fs::{expand_path, Url}, Layer};

use crate::{emit, files::{File, FilesOp}, manager::Manager, tab::Tab};

pub struct Opt {
	target: Url,
}

impl From<&Exec> for Opt {
	fn from(e: &Exec) -> Self {
		let mut target = Url::from(e.args.first().map(|s| s.as_str()).unwrap_or(""));
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
		emit!(Call(Exec::call("reveal", vec![target.to_string()]).vec(), Layer::Manager));
	}

	pub fn reveal(&mut self, opt: impl Into<Opt>) -> bool {
		let opt = opt.into() as Opt;

		let Some(parent) = opt.target.parent_url() else {
			return false;
		};

		let b = self.cd(parent.clone());
		emit!(Files(FilesOp::Creating(
			parent.clone(),
			File::from_dummy(opt.target.clone()).into_map()
		)));
		Manager::_hover(Some(opt.target));
		b
	}
}
