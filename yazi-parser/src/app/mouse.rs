use crossterm::event::MouseEvent;

pub struct MouseOpt {
	pub event: MouseEvent,
}

impl From<MouseEvent> for MouseOpt {
	fn from(event: MouseEvent) -> Self { Self { event } }
}
