use std::num::NonZeroU8;

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
	/// Mid-UTF-8 character in ground: `n` continuation bytes still needed.
	Utf8(NonZeroU8),
	/// `\x1B` + mid-UTF-8 character: `n` continuation bytes still needed.
	AltUtf8(NonZeroU8),
}

#[derive(Debug, Default, PartialEq)]
pub(crate) struct StateOsc72 {
	pub(crate) r#type:   Option<u8>,
	pub(crate) x:        Option<i32>,
	pub(crate) y:        Option<i32>,
	pub(crate) op:       Option<u8>,
	pub(crate) payload:  Vec<u8>,
	pub(crate) has_more: bool,
}

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
