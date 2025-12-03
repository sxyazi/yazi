use yazi_macro::{emit, relay};
use yazi_parser::cmp::ShowOpt;
use yazi_shared::Id;

pub struct CmpProxy;

impl CmpProxy {
	pub fn show(opt: ShowOpt) {
		emit!(Call(relay!(cmp:show).with_any("opt", opt)));
	}

	pub fn trigger(word: impl Into<String>, ticket: Id) {
		emit!(Call(relay!(cmp:trigger, [word.into()]).with("ticket", ticket)));
	}
}
