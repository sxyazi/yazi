use yazi_shared::{emit, event::Cmd, fs::Url, Layer};

use crate::options::OpenDoOpt;

pub struct ManagerProxy;

impl ManagerProxy {
	#[inline]
	pub fn peek(force: bool) {
		emit!(Call(Cmd::new("peek").with_bool("force", force), Layer::Manager));
	}

	#[inline]
	pub fn hover(url: Option<Url>, tab: usize) {
		emit!(Call(
			Cmd::args("hover", &url.map_or_else(Vec::new, |u| vec![u])).with("tab", tab),
			Layer::Manager
		));
	}

	#[inline]
	pub fn refresh() {
		emit!(Call(Cmd::new("refresh"), Layer::Manager));
	}

	#[inline]
	pub fn open_do(opt: OpenDoOpt) {
		emit!(Call(Cmd::new("open_do").with_any("option", opt), Layer::Manager));
	}

	#[inline]
	pub fn remove_do(targets: Vec<Url>, permanently: bool) {
		emit!(Call(
			Cmd::new("remove_do").with_bool("permanently", permanently).with_any("targets", targets),
			Layer::Manager
		));
	}

	#[inline]
	pub fn update_task(url: &Url) {
		emit!(Call(Cmd::new("update_task").with_any("url", url.clone()), Layer::Manager));
	}

	#[inline]
	pub fn update_paged() {
		emit!(Call(Cmd::new("update_paged"), Layer::Manager));
	}

	#[inline]
	pub fn update_paged_by(page: usize, only_if: &Url) {
		emit!(Call(
			Cmd::args("update_paged", &[page]).with_any("only-if", only_if.clone()),
			Layer::Manager
		));
	}
}
