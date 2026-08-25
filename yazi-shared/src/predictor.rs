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
	#[cfg(windows)]
	pub const SEP: Self = Self(b"/\\");
	#[cfg(not(windows))]
	pub const SEP: Self = Self(b"/");
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
