use yazi_binding::position::{Offset, Origin, Position};

pub struct Help;

impl Help {
	pub fn position() -> Position {
		Position::new(Origin::Center, Offset { x: 0, y: 0, width: 80, height: 25 })
	}
}
