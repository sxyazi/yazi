use std::{borrow::Cow, path::{Path, PathBuf}};

use anyhow::Result;
use percent_encoding::{AsciiSet, CONTROLS, percent_decode, percent_encode};
use yazi_shared::path::{PathCow, PathDyn, PathKind};

const SET: &AsciiSet =
	&CONTROLS.add(b'"').add(b'*').add(b':').add(b'<').add(b'>').add(b'?').add(b'\\').add(b'|');

pub trait PercentEncoding<'a> {
	fn percent_encode(self) -> Cow<'a, Path>;

	fn percent_decode<K>(self, kind: K) -> Result<PathCow<'a>>
	where
		K: Into<PathKind>;
}

impl<'a> PercentEncoding<'a> for PathDyn<'a> {
	fn percent_encode(self) -> Cow<'a, Path> {
		match percent_encode(self.encoded_bytes(), SET).into() {
			Cow::Borrowed(s) => Path::new(s).into(),
			Cow::Owned(s) => PathBuf::from(s).into(),
		}
	}

	fn percent_decode<K>(self, kind: K) -> Result<PathCow<'a>>
	where
		K: Into<PathKind>,
	{
		PathCow::with(kind, percent_decode(self.encoded_bytes()))
	}
}
