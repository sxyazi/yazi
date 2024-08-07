use yazi_shared::{emit, event::Cmd, fs::Url, Layer};

use crate::options::SearchOpt;

pub struct TabProxy;

impl TabProxy {
	#[inline]
	pub fn cd(target: &Url) {
		emit!(Call(Cmd::args("cd", vec![target.to_string()]), Layer::Manager));
	}

	#[inline]
	pub fn reveal(target: &Url) {
		emit!(Call(Cmd::args("reveal", vec![target.to_string()]), Layer::Manager));
	}

	#[inline]
	pub fn search_do(opt: SearchOpt) {
		emit!(Call(
			Cmd::args("search_do", vec![opt.subject]).with("via", opt.via).with("args", opt.args_raw),
			Layer::Manager
		));
	}
}
