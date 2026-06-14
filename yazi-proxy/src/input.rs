use tokio::sync::mpsc;
use yazi_config::popup::InputCfg;
use yazi_dds::Pubsub;
use yazi_macro::{emit, err, relay};
use yazi_shim::strum::IntoStr;
use yazi_widgets::input::{InputCallback, InputEvent};

pub struct InputProxy;

impl InputProxy {
	pub fn show(cfg: InputCfg) -> mpsc::UnboundedReceiver<InputEvent> {
		let (tx, rx) = mpsc::unbounded_channel();
		let cb: Box<dyn InputCallback> = Box::new(move |event| {
			err!(Pubsub::pub_after_input((&event).into_str(), event.value()));
			tx.send(event).ok();
		});

		emit!(Call(relay!(input:show).with_any("cb", cb).with_any("cfg", cfg)));
		rx
	}
}
