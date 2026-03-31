use yazi_core::cmp::CmpOpt;
use yazi_macro::{emit, relay};
use yazi_shared::Id;

pub struct CmpProxy;

impl CmpProxy {
	pub fn show(opt: CmpOpt) {
		emit!(Call(relay!(cmp:show).with_any("opt", opt)));
	}

	pub fn trigger(word: impl Into<String>, ticket: Option<Id>) {
		emit!(Call(relay!(cmp:trigger, [word.into()]).with_opt("ticket", ticket)));
	}
}
