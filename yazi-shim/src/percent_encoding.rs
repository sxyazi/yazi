use percent_encoding::{AsciiSet, NON_ALPHANUMERIC};

// RFC 3986 path component: encode everything that is not a safe path character.
pub const RFC_3986: &AsciiSet =
	&NON_ALPHANUMERIC.remove(b'-').remove(b'.').remove(b'_').remove(b'~');
