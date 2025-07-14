use std::borrow::Cow;

use yazi_macro::emit;
use yazi_shared::{SStr, event::Cmd, url::Url};

use crate::options::{OpenDoOpt, SearchOpt};

pub struct MgrProxy;

impl MgrProxy {
	pub fn cd(target: &Url) {
		emit!(Call(Cmd::args("mgr:cd", [target]).with("raw", true)));
	}

	pub fn reveal(target: &Url) {
		emit!(Call(Cmd::args("mgr:reveal", [target]).with("raw", true).with("no-dummy", true)));
	}

	pub fn arrow(step: impl Into<SStr>) {
		emit!(Call(Cmd::args("mgr:arrow", [step.into()])));
	}

	pub fn open_do(opt: OpenDoOpt) {
		emit!(Call(Cmd::new("mgr:open_do").with_any("option", opt)));
	}

	pub fn remove_do(targets: Vec<Url>, permanently: bool) {
		emit!(Call(
			Cmd::new("mgr:remove_do").with("permanently", permanently).with_any("targets", targets)
		));
	}

	pub fn search_do(opt: SearchOpt) {
		emit!(Call(
			// TODO: use second positional argument instead of `args` parameter
			Cmd::args("mgr:search_do", [opt.subject])
				.with("via", Cow::Borrowed(opt.via.into_str()))
				.with("args", opt.args_raw.into_owned())
		));
	}

	pub fn update_tasks(url: &Url) {
		emit!(Call(Cmd::new("mgr:update_tasks").with_any("urls", vec![url.clone()])));
	}

	pub fn update_paged_by(page: usize, only_if: &Url) {
		emit!(Call(Cmd::args("mgr:update_paged", [page]).with_any("only-if", only_if.clone())));
	}
}
