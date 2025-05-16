use tokio::sync::mpsc;
use yazi_config::popup::InputCfg;
use yazi_macro::emit;
use yazi_shared::{Id, errors::InputError, event::Cmd};

use crate::options::CmpItem;

pub struct InputProxy;

impl InputProxy {
	#[inline]
	pub fn show(cfg: InputCfg) -> mpsc::UnboundedReceiver<Result<String, InputError>> {
		let (tx, rx) = mpsc::unbounded_channel();
		emit!(Call(Cmd::new("input:show").with_any("tx", tx).with_any("cfg", cfg)));
		rx
	}

	#[inline]
	pub fn complete(item: &CmpItem, ticket: Id) {
		emit!(Call(Cmd::new("input:complete").with_any("item", item.clone()).with("ticket", ticket)));
	}
}
