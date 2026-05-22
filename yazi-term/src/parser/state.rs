use std::num::NonZeroU8;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum State {
	/// Normal ground state.
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
