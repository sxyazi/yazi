use yazi_macro::emit;
use yazi_shared::{Id, event::Cmd};

pub struct CmpProxy;

impl CmpProxy {
	pub fn close() {
		emit!(Call(Cmd::new("cmp:close")));
	}

	pub fn trigger(word: &str, ticket: Id) {
		emit!(Call(Cmd::args("cmp:trigger", [word]).with("ticket", ticket)));
	}
}
