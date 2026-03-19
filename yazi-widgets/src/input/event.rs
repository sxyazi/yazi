use yazi_shared::Id;

#[derive(Debug)]
pub enum InputEvent {
	Submit(String),
	Cancel(String),

	Type(String),
	Trigger(String, Option<Id>),
}

impl InputEvent {
	pub fn is_submit(&self) -> bool { matches!(self, Self::Submit(_)) }
}
