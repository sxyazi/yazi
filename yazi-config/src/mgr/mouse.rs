use anyhow::{Result, bail};
use bitflags::bitflags;
use crossterm::event::MouseEventKind;
use serde::{Deserialize, Serialize};

bitflags! {
	#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
	#[serde(try_from = "Vec<String>", into = "Vec<String>")]
	pub struct MouseEvents: u8 {
		const CLICK  = 0b00001;
		const SCROLL = 0b00010;
		const TOUCH  = 0b00100;
		const MOVE   = 0b01000;
		const DRAG   = 0b10000;
	}
}

impl MouseEvents {
	pub const fn draggable(self) -> bool { self.contains(Self::DRAG) }
}

impl TryFrom<Vec<String>> for MouseEvents {
	type Error = anyhow::Error;

	fn try_from(value: Vec<String>) -> Result<Self, Self::Error> {
		value.into_iter().try_fold(Self::empty(), |aac, s| {
			Ok(match s.as_str() {
				"click" => aac | Self::CLICK,
				"scroll" => aac | Self::SCROLL,
				"touch" => aac | Self::TOUCH,
				"move" => aac | Self::MOVE,
				"drag" => aac | Self::DRAG,
				_ => bail!("Invalid mouse event: {s}"),
			})
		})
	}
}

impl From<MouseEvents> for Vec<String> {
	fn from(value: MouseEvents) -> Self {
		let events = [
			(MouseEvents::CLICK, "click"),
			(MouseEvents::SCROLL, "scroll"),
			(MouseEvents::TOUCH, "touch"),
			(MouseEvents::MOVE, "move"),
			(MouseEvents::DRAG, "drag"),
		];
		events.into_iter().filter(|v| value.contains(v.0)).map(|v| v.1.to_owned()).collect()
	}
}

impl From<crossterm::event::MouseEventKind> for MouseEvents {
	fn from(value: crossterm::event::MouseEventKind) -> Self {
		match value {
			MouseEventKind::Down(_) | MouseEventKind::Up(_) => Self::CLICK,
			MouseEventKind::ScrollDown | MouseEventKind::ScrollUp => Self::SCROLL,
			MouseEventKind::ScrollLeft | MouseEventKind::ScrollRight => Self::TOUCH,
			MouseEventKind::Moved => Self::MOVE,
			MouseEventKind::Drag(_) => Self::DRAG,
		}
	}
}
