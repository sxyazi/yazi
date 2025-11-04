pub trait PathInner<'a>: Copy {
	fn len(self) -> usize { self.encoded_bytes().len() }

	fn encoded_bytes(self) -> &'a [u8];
}

impl<'a> PathInner<'a> for &'a std::ffi::OsStr {
	fn encoded_bytes(self) -> &'a [u8] { self.as_encoded_bytes() }
}

impl<'a> PathInner<'a> for &'a [u8] {
	fn encoded_bytes(self) -> &'a [u8] { self }
}
