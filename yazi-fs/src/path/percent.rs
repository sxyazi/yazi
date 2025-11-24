use std::{borrow::Cow, path::{Path, PathBuf}};

use percent_encoding::{AsciiSet, CONTROLS, percent_decode, percent_encode};
use yazi_shared::path::PathDyn;

const SET: &AsciiSet =
	&CONTROLS.add(b'"').add(b'*').add(b':').add(b'<').add(b'>').add(b'?').add(b'\\').add(b'|');

pub trait PercentEncoding {
	fn percent_encode(&self) -> Cow<'_, Path>;

	fn percent_decode(&self) -> Cow<'_, [u8]>;
}

impl PercentEncoding for Path {
	fn percent_encode(&self) -> Cow<'_, Path> {
		match percent_encode(self.as_os_str().as_encoded_bytes(), SET).into() {
			Cow::Borrowed(_) => self.into(),
			Cow::Owned(s) => PathBuf::from(s).into(),
		}
	}

	fn percent_decode(&self) -> Cow<'_, [u8]> {
		match percent_decode(self.as_os_str().as_encoded_bytes()).into() {
			Cow::Borrowed(_) => self.as_os_str().as_encoded_bytes().into(),
			Cow::Owned(s) => s.into(),
		}
	}
}

impl PercentEncoding for PathDyn<'_> {
	fn percent_encode(&self) -> Cow<'_, Path> {
		match self {
			PathDyn::Os(p) => p.percent_encode(),
			PathDyn::Unix(_) => todo!(),
		}
	}

	fn percent_decode(&self) -> Cow<'_, [u8]> {
		match self {
			PathDyn::Os(p) => p.percent_decode(),
			PathDyn::Unix(_) => todo!(),
		}
	}
}
