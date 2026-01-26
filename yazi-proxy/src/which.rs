use yazi_macro::{emit, relay};
use yazi_parser::which::ActivateOpt;

pub struct WhichProxy;

impl WhichProxy {
	pub fn activate(opt: ActivateOpt) {
		emit!(Call(relay!(which:activate).with_any("opt", opt)));
	}
}
