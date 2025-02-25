use yazi_macro::emit;
use yazi_shared::{Layer, event::Cmd, url::Url};

use crate::options::SearchOpt;

pub struct TabProxy;

impl TabProxy {
	#[inline]
	pub fn cd(target: &Url) {
		emit!(Call(Cmd::args("cd", &[target]), Layer::Mgr));
	}

	#[inline]
	pub fn reveal(target: &Url) {
		emit!(Call(Cmd::args("reveal", &[target]), Layer::Mgr));
	}

	#[inline]
	pub fn arrow(step: isize) {
		emit!(Call(Cmd::args("arrow", &[step]), Layer::Mgr));
	}

	#[inline]
	pub fn search_do(opt: SearchOpt) {
		emit!(Call(
			// TODO: use second positional argument instead of `args` parameter
			Cmd::args("search_do", &[opt.subject]).with("via", opt.via).with("args", opt.args_raw),
			Layer::Mgr
		));
	}
}
