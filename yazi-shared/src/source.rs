use bitflags::bitflags;

bitflags! {
	#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
	pub struct Source: u8 {
		const KEY   = 0b00000001;
		const EMIT  = 0b00000010;

		const ACTOR = 0b00000100;
		const PROXY = 0b00001000;
	}
}
