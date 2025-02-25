use yazi_macro::emit;
use yazi_shared::{event::Cmd, url::Url};

use crate::options::SearchOpt;

pub struct TabProxy;

impl TabProxy {
	#[inline]
	pub fn cd(target: &Url) {
		emit!(Call(Cmd::args("mgr:cd", &[target])));
	}

	#[inline]
	pub fn reveal(target: &Url) {
		emit!(Call(Cmd::args("mgr:reveal", &[target])));
	}

	#[inline]
	pub fn arrow(step: isize) {
		emit!(Call(Cmd::args("mgr:arrow", &[step])));
	}

	#[inline]
	pub fn search_do(opt: SearchOpt) {
		emit!(Call(
			// TODO: use second positional argument instead of `args` parameter
			Cmd::args("mgr:search_do", &[opt.subject]).with("via", opt.via).with("args", opt.args_raw)
		));
	}
}
