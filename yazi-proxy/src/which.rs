use yazi_macro::emit;
use yazi_parser::which::ShowOpt;
use yazi_shared::event::Cmd;

pub struct WhichProxy;

impl WhichProxy {
	pub fn show(opt: ShowOpt) {
		emit!(Call(Cmd::new("which:show").with_any("opt", opt)));
	}
}
