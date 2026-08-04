yazi_macro::mod_flat!(csi ground osc parser report state);

#[cfg(windows)]
yazi_macro::mod_flat!(windows);

pub(super) fn strip_osc_terminator(seq: &[u8]) -> crate::Result<&[u8]> {
	seq
		.strip_suffix(b"\x1b\\")
		.or_else(|| seq.strip_suffix(b"\x07"))
		.ok_or(crate::ParseError::Invalid)
}
