use yazi_macro::emit;
use yazi_parser::cmp::ShowOpt;
use yazi_shared::{Id, event::Cmd};

pub struct CmpProxy;

impl CmpProxy {
	pub fn show(opt: ShowOpt) {
		emit!(Call(Cmd::new("cmp:show").with_any("opt", opt)));
	}

	pub fn trigger(word: &str, ticket: Id) {
		emit!(Call(Cmd::args("cmp:trigger", [word]).with("ticket", ticket)));
	}
}
