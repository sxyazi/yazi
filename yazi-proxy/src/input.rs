use tokio::sync::mpsc;
use yazi_config::popup::InputCfg;
use yazi_macro::emit;
use yazi_shared::{errors::InputError, event::Cmd};

pub struct InputProxy;

impl InputProxy {
	pub fn show(cfg: InputCfg) -> mpsc::UnboundedReceiver<Result<String, InputError>> {
		let (tx, rx) = mpsc::unbounded_channel();
		emit!(Call(Cmd::new("input:show").with_any("tx", tx).with_any("cfg", cfg)));
		rx
	}
}
