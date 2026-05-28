use percent_encoding::{AsciiSet, CONTROLS};

// RFC 3986 path component: encode everything that is not a safe path character.
// Safe chars: unreserved (A-Za-z0-9 -._~) + sub-delims (!$&'()*+,;=) + : @ /
pub const RFC_3986: &AsciiSet = &CONTROLS
	.add(b' ')
	.add(b'"')
	.add(b'#')
	.add(b'%')
	.add(b'<')
	.add(b'>')
	.add(b'?')
	.add(b'[')
	.add(b'\\')
	.add(b']')
	.add(b'^')
	.add(b'`')
	.add(b'{')
	.add(b'|')
	.add(b'}');
