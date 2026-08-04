use std::{num::NonZeroU8, time::Duration};

#[derive(Debug, Default, PartialEq)]
pub(crate) enum State {
	/// Normal ground state.
	#[default]
	Ground,
	/// Saw `\x1B`, waiting for the next byte.
	Esc,
	/// Saw `\x1B O`, waiting for the SS3 key byte.
	EscO,
	/// Inside a CSI sequence (`\x1B[` … final-byte in 0x40–0x7E).
	Csi,
	/// Saw `\x1B[M`, collecting 3 raw mouse bytes (CB Cx Cy).
	NormalMouse,
	/// Inside bracketed paste (`\x1B[200~` … `\x1B[201~`).
	BracketedPaste,
	/// Inside an OSC sequence (`\x1B]` … BEL or ST).
	Osc,
	/// Inside an OSC 72 (DnD) sequence (`\x1B]72;` … ST).
	Osc72(StateOsc72),
	/// Inside an OSC 5522 (Clipboard) sequence (`\x1B]5522;` … ST).
	Osc5522(StateOsc5522),
	/// Inside OSC, just saw `\x1B` (potential start of ST = `\x1B\\`).
	OscSt,
	/// Inside a DCS sequence (`\x1BP` … ST).
	Dcs,
	/// Inside DCS, just saw `\x1B` (potential start of ST).
	DcsSt,
	/// Inside an APC sequence (`\x1B_` … ST).
	Apc,
	/// Inside APC, just saw `\x1B` (potential start of ST).
	ApcSt,
	/// Mid-UTF-8 character in ground: `n` continuation bytes still needed.
	Utf8(NonZeroU8),
	/// `\x1B` + mid-UTF-8 character: `n` continuation bytes still needed.
	AltUtf8(NonZeroU8),
}

impl State {
	pub(super) const fn limit(&self) -> usize {
		match self {
			Self::Csi => 4096,
			Self::BracketedPaste => 16 << 20,
			Self::Osc
			| Self::Osc72(_)
			| Self::Osc5522(_)
			| Self::OscSt
			| Self::Dcs
			| Self::DcsSt
			| Self::Apc
			| Self::ApcSt => 1 << 20,
			_ => usize::MAX,
		}
	}

	pub(super) const fn timeout(&self) -> Option<Duration> {
		match self {
			Self::Ground | Self::BracketedPaste => None,
			Self::Esc => Some(Duration::from_millis(10)),
			_ => Some(Duration::from_secs(1)),
		}
	}
}

// --- StateOsc72
#[derive(Debug, Default, PartialEq)]
pub(crate) struct StateOsc72 {
	pub(crate) r#type:   Option<u8>,
	pub(crate) x:        Option<i32>,
	pub(crate) y:        Option<i32>,
	pub(crate) op:       Option<u8>,
	pub(crate) payload:  Vec<u8>,
	pub(crate) has_more: bool,
}

// --- StateOsc5522
#[derive(Debug, Default, PartialEq)]
pub(crate) struct StateOsc5522 {
	pub(crate) status:   Option<Osc5522Status>,
	pub(crate) read:     bool,
	pub(crate) primary:  bool,
	pub(crate) mime:     Vec<Vec<u8>>,
	pub(crate) payload:  Vec<Vec<u8>>,
	pub(crate) pw:       Vec<u8>,
	pub(crate) idx:      usize,
	pub(crate) has_more: bool,
}

#[derive(Debug, Default, PartialEq)]
pub(crate) enum Osc5522Status {
	#[default]
	OK,
	DATA,
	DONE,
	ENOSYS,
	EPERM,
	EBUSY,
	EIO,
	EINVAL,
}
