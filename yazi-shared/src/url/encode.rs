use std::{fmt::{self, Display, Write}, ops::Deref};

use percent_encoding::{AsciiSet, CONTROLS, percent_encode};
use yazi_shim::PercentEncoder;

use crate::{auth::EncodePrefix, spec::{Encode as EncodeSpec, EncodePorts}, url::Url};

impl<'a> Url<'a> {
	pub fn encode(self) -> Encode<'a> { Encode(self) }
}

// --- Encode
#[derive(Clone, Copy)]
pub struct Encode<'a>(pub(crate) Url<'a>);

impl<'a> Deref for Encode<'a> {
	type Target = Url<'a>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl Display for Encode<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		const SET: &AsciiSet = &CONTROLS.add(b'%');

		let loc = percent_encode(self.loc().encoded_bytes(), SET);
		let auth = self.auth();

		if auth.is_regular() {
			return write!(f, "{}{loc}", auth.encode(true));
		}

		write!(f, "{}{}{}", auth.encode(true), EncodePorts(self.0), EncodePrefix(auth))?;
		if self.is_view() {
			let source = self.base().physical();
			if !source.is_regular() {
				write!(PercentEncoder::new(f, SET), "{}", EncodeSpec(source))?;
			}
		}

		loc.fmt(f)
	}
}
