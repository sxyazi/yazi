use yazi_proxy::ManagerProxy;
use yazi_shared::{event::{Cmd, Data}, fs::{expand_path, File, FilesOp, Url}};

use crate::tab::Tab;

pub struct Opt {
	target: Url,
}

impl From<Cmd> for Opt {
	fn from(mut c: Cmd) -> Self {
		let mut target = c.take_first().and_then(Data::into_url).unwrap_or_default();
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
	pub fn reveal(&mut self, opt: impl Into<Opt>) {
		let opt = opt.into() as Opt;

		let Some(parent) = opt.target.parent_url() else {
			return;
		};

		self.cd(parent.clone());
		FilesOp::Creating(parent, vec![File::from_dummy(opt.target.clone(), None)]).emit();
		ManagerProxy::hover(Some(opt.target), self.idx);
	}
}
