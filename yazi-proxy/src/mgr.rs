use std::borrow::Cow;

use yazi_macro::{emit, relay};
use yazi_parser::mgr::{FilterOpt, FindDoOpt, OpenDoOpt, SearchOpt, UpdatePeekedOpt, UpdateSpottedOpt};
use yazi_shared::{SStr, url::UrlBuf};

pub struct MgrProxy;

impl MgrProxy {
	pub fn cd(target: &UrlBuf) {
		emit!(Call(relay!(mgr:cd, [target]).with("raw", true)));
	}

	pub fn reveal(target: &UrlBuf) {
		emit!(Call(relay!(mgr:reveal, [target]).with("raw", true).with("no-dummy", true)));
	}

	pub fn arrow(step: impl Into<SStr>) {
		emit!(Call(relay!(mgr:arrow, [step.into()])));
	}

	pub fn open_do(opt: OpenDoOpt) {
		emit!(Call(relay!(mgr:open_do).with_any("option", opt)));
	}

	pub fn remove_do(targets: Vec<UrlBuf>, permanently: bool) {
		emit!(Call(
			relay!(mgr:remove_do).with("permanently", permanently).with_any("targets", targets)
		));
	}

	pub fn find_do(opt: FindDoOpt) {
		emit!(Call(relay!(mgr:find_do).with_any("opt", opt)));
	}

	pub fn filter_do(opt: FilterOpt) {
		emit!(Call(relay!(mgr:filter_do).with_any("opt", opt)));
	}

	pub fn search_do(opt: SearchOpt) {
		emit!(Call(
			// TODO: use second positional argument instead of `args` parameter
			relay!(mgr:search_do, [opt.subject])
				.with("via", Cow::Borrowed(opt.via.into_str()))
				.with("args", opt.args_raw.into_owned())
		));
	}

	pub fn update_peeked(opt: UpdatePeekedOpt) {
		emit!(Call(relay!(mgr:update_peeked).with_any("opt", opt)));
	}

	pub fn update_spotted(opt: UpdateSpottedOpt) {
		emit!(Call(relay!(mgr:update_spotted).with_any("opt", opt)));
	}

	pub fn update_paged_by(page: usize, only_if: &UrlBuf) {
		emit!(Call(relay!(mgr:update_paged, [page]).with_any("only-if", only_if.clone())));
	}
}
