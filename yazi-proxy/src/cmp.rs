use yazi_macro::emit;
use yazi_shared::{Id, event::Cmd};

pub struct CmpProxy;

impl CmpProxy {
	#[inline]
	pub fn close() {
		emit!(Call(Cmd::new("cmp:close")));
	}

	#[inline]
	pub fn trigger(word: &str, ticket: Id) {
		emit!(Call(Cmd::args("cmp:trigger", &[word]).with("ticket", ticket)));
	}
}
