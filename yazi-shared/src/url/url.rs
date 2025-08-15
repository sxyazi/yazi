use crate::{loc::LocBuf, url::Scheme};

pub struct Url<'a> {
	pub loc:    &'a LocBuf,
	pub scheme: &'a Scheme,
}

impl<'a> Url<'a> {
	pub fn regular() {}
}
