use std::{borrow::Cow, ffi::{OsStr, OsString}};

pub trait IntoOsStr<'a> {
	type Error;

	fn into_os_str(self) -> Result<Cow<'a, OsStr>, Self::Error>;
}

impl<'a> IntoOsStr<'a> for Cow<'a, [u8]> {
	type Error = anyhow::Error;

	fn into_os_str(self) -> Result<Cow<'a, OsStr>, Self::Error> {
		#[cfg(unix)]
		{
			use std::os::unix::ffi::{OsStrExt, OsStringExt};
			Ok(match self {
				Cow::Borrowed(b) => Cow::Borrowed(OsStr::from_bytes(b)),
				Cow::Owned(b) => Cow::Owned(OsString::from_vec(b)),
			})
		}
		#[cfg(windows)]
		{
			Ok(match self {
				Cow::Borrowed(b) => Cow::Borrowed(OsStr::new(str::from_utf8(b)?)),
				Cow::Owned(b) => Cow::Owned(OsString::from(String::from_utf8(b)?)),
			})
		}
	}
}

impl<'a> IntoOsStr<'a> for &'a [u8] {
	type Error = anyhow::Error;

	fn into_os_str(self) -> Result<Cow<'a, OsStr>, Self::Error> { Cow::Borrowed(self).into_os_str() }
}

// --- OsStrJoin
pub trait OsStrJoin {
	fn join(&self, sep: &OsStr) -> OsString;
}

impl<T> OsStrJoin for Vec<T>
where
	T: AsRef<OsStr>,
{
	fn join(&self, sep: &OsStr) -> OsString {
		if self.is_empty() {
			return OsString::new();
		}

		let mut result = OsString::new();
		for (i, item) in self.iter().enumerate() {
			if i > 0 {
				result.push(sep);
			}
			result.push(item.as_ref());
		}
		result
	}
}

// --- OsStrSplit
pub trait OsStrSplit {
	fn rsplit_once<P: Pattern>(&self, predicate: P) -> Option<(&Self, &Self)>;
}

impl OsStrSplit for OsStr {
	fn rsplit_once<P: Pattern>(&self, pat: P) -> Option<(&Self, &Self)> {
		let bytes = self.as_encoded_bytes();
		for (i, &byte) in bytes.iter().enumerate().rev() {
			if !pat.predicate(byte) {
				continue;
			}

			let (a, b) = bytes.split_at(i);
			// SAFETY: These substrings were separated by a UTF-8 string.
			return Some(unsafe {
				(Self::from_encoded_bytes_unchecked(a), Self::from_encoded_bytes_unchecked(&b[1..]))
			});
		}
		None
	}
}

pub trait Pattern {
	fn predicate(&self, byte: u8) -> bool;
}

impl Pattern for char {
	fn predicate(&self, byte: u8) -> bool { *self == byte as Self }
}

impl Pattern for &[char] {
	fn predicate(&self, byte: u8) -> bool { self.contains(&(byte as char)) }
}
