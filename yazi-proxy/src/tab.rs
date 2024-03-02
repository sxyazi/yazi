use yazi_shared::{emit, event::Cmd, fs::Url, Layer};

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
}
