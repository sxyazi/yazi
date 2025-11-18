// --- BytePredictor
pub trait BytePredictor {
	fn predicate(&self, byte: u8) -> bool;
}

// --- Utf8BytePredictor
pub trait Utf8BytePredictor {
	fn predicate(&self, byte: u8) -> bool;
}

// --- AnyAsciiChar
pub struct AnyAsciiChar<'a>(&'a [u8]);

impl<'a> AnyAsciiChar<'a> {
	pub fn new(chars: &'a [u8]) -> Option<Self> {
		if chars.iter().all(|&b| b <= 0x7f) { Some(Self(chars)) } else { None }
	}
}

impl Utf8BytePredictor for AnyAsciiChar<'_> {
	fn predicate(&self, byte: u8) -> bool { self.0.contains(&byte) }
}

impl<T> BytePredictor for T
where
	T: Utf8BytePredictor,
{
	fn predicate(&self, byte: u8) -> bool { self.predicate(byte) }
}
