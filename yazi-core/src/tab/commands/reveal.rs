use yazi_fs::{File, FilesOp};
use yazi_parser::tab::RevealOpt;
use yazi_proxy::MgrProxy;

use crate::tab::Tab;

impl Tab {
	#[yazi_codegen::command]
	pub fn reveal(&mut self, opt: RevealOpt) {
		let Some((parent, child)) = opt.target.pair() else {
			return;
		};

		self.cd((parent.clone(), opt.source));
		self.current.hover(child.as_urn());

		if !opt.no_dummy && self.hovered().is_none_or(|f| &child != f.urn()) {
			let op = FilesOp::Creating(parent, vec![File::from_dummy(opt.target.clone(), None)]);
			self.current.update_pub(self.id, op);
		}

		self.hover(Some(opt.target));
		MgrProxy::peek(false);
		MgrProxy::watch();
	}
}
