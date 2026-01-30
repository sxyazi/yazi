use std::ops::{Deref, DerefMut};

use crate::Adapter;

pub(super) struct Adapters(Vec<Adapter>);

impl Deref for Adapters {
	type Target = Vec<Adapter>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl DerefMut for Adapters {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl From<&yazi_emulator::Emulator> for Adapters {
	fn from(value: &yazi_emulator::Emulator) -> Self { value.kind.either_into() }
}

impl From<yazi_emulator::Brand> for Adapters {
	fn from(value: yazi_emulator::Brand) -> Self {
		use yazi_emulator::Brand as B;

		use crate::Adapter as A;

		Self(match value {
			B::Kitty => vec![A::Kgp],
			B::Konsole => vec![A::KgpOld],
			B::Iterm2 => vec![A::Iip, A::Sixel],
			B::WezTerm => vec![A::Iip, A::Sixel],
			B::Foot => vec![A::Sixel],
			B::Ghostty => vec![A::Kgp],
			B::Microsoft => vec![A::Sixel],
			B::Warp => vec![A::Iip, A::KgpOld],
			B::Rio => vec![A::Iip, A::Sixel],
			B::BlackBox => vec![A::Sixel],
			B::VSCode => vec![A::Iip, A::Sixel],
			B::Tabby => vec![A::Iip, A::Sixel],
			B::Hyper => vec![A::Iip, A::Sixel],
			B::Mintty => vec![A::Iip],
			B::Tmux => vec![],
			B::VTerm => vec![],
			B::Apple => vec![],
			B::Urxvt => vec![],
			B::Bobcat => vec![A::Iip, A::Sixel],
		})
	}
}

impl From<yazi_emulator::Unknown> for Adapters {
	fn from(value: yazi_emulator::Unknown) -> Self {
		use Adapter as A;

		Self(match (value.kgp, value.sixel) {
			(true, true) => vec![A::Sixel, A::KgpOld],
			(true, false) => vec![A::KgpOld],
			(false, true) => vec![A::Sixel],
			(false, false) => vec![],
		})
	}
}
