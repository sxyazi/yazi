use yazi_macro::{emit, relay};
use yazi_parser::which::ShowOpt;

pub struct WhichProxy;

impl WhichProxy {
	pub fn show(opt: ShowOpt) {
		emit!(Call(relay!(which:show).with_any("opt", opt)));
	}
}
