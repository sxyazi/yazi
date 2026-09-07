use std::{borrow::Cow, str::{self, Utf8Error}};

pub trait IntoUtf8<'a> {
	fn into_utf8(self) -> Result<Cow<'a, str>, Utf8Error>;
}

impl<'a> IntoUtf8<'a> for Cow<'a, [u8]> {
	fn into_utf8(self) -> Result<Cow<'a, str>, Utf8Error> {
		match self {
			Cow::Borrowed(b) => str::from_utf8(b).map(Cow::Borrowed),
			Cow::Owned(b) => String::from_utf8(b).map(Cow::Owned).map_err(|e| e.utf8_error()),
		}
	}
}
