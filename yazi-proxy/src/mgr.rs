use yazi_macro::emit;
use yazi_shared::{event::Cmd, url::Url};

use crate::options::OpenDoOpt;

pub struct MgrProxy;

impl MgrProxy {
	pub fn spot(skip: Option<usize>) {
		emit!(Call(Cmd::new("mgr:spot").with_opt("skip", skip)));
	}

	pub fn peek(force: bool) {
		emit!(Call(Cmd::new("mgr:peek").with("force", force)));
	}

	pub fn watch() {
		emit!(Call(Cmd::new("mgr:watch")));
	}

	pub fn refresh() {
		emit!(Call(Cmd::new("mgr:refresh")));
	}

	pub fn open_do(opt: OpenDoOpt) {
		emit!(Call(Cmd::new("mgr:open_do").with_any("option", opt)));
	}

	pub fn remove_do(targets: Vec<Url>, permanently: bool) {
		emit!(Call(
			Cmd::new("mgr:remove_do").with("permanently", permanently).with_any("targets", targets)
		));
	}

	pub fn update_tasks(url: &Url) {
		emit!(Call(Cmd::new("mgr:update_tasks").with_any("urls", vec![url.clone()])));
	}

	pub fn update_paged() {
		emit!(Call(Cmd::new("mgr:update_paged")));
	}

	pub fn update_paged_by(page: usize, only_if: &Url) {
		emit!(Call(Cmd::args("mgr:update_paged", [page]).with_any("only-if", only_if.clone())));
	}
}
