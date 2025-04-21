use yazi_fs::Filter;
use yazi_macro::render;
use yazi_proxy::MgrProxy;

use super::filter::Opt;
use crate::tab::Tab;

impl Tab {
	#[yazi_codegen::command]
	pub fn filter_do(&mut self, opt: Opt) {
		let filter = if opt.query.is_empty() {
			None
		} else if let Ok(f) = Filter::new(&opt.query, opt.case) {
			Some(f)
		} else {
			return;
		};

		if opt.done {
			MgrProxy::update_paged(); // Update for paged files in next loop
		}

		let hovered = self.hovered().map(|f| f.urn_owned());
		if !self.current.files.set_filter(filter) {
			return;
		}

		self.current.repos(hovered.as_ref());
		if self.hovered().map(|f| f.urn()) != hovered.as_ref().map(|u| u.as_urn()) {
			self.hover(None);
			MgrProxy::peek(false);
			MgrProxy::watch();
		}

		render!();
	}
}
