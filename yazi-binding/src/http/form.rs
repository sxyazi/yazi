use std::io;

use futures::TryStreamExt;
use http_body_util::{BodyExt, StreamBody, combinators::BoxBody};
use hyper::body::{Bytes, Frame};
use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};
use reqwest::multipart::{Form, Part};

use super::HttpPart;

pub(crate) struct HttpForm {
	form:   Option<Form>,
	fields: Vec<(String, String)>,
}

impl Default for HttpForm {
	fn default() -> Self { Self { form: Some(Form::new()), fields: Vec::new() } }
}

impl From<HttpForm> for Form {
	fn from(value: HttpForm) -> Self { value.form.unwrap() }
}

impl From<HttpForm> for BoxBody<Bytes, io::Error> {
	fn from(value: HttpForm) -> Self {
		let form: Form = value.into();
		StreamBody::new(form.into_stream().map_ok(Frame::data).map_err(io::Error::other)).boxed()
	}
}

impl HttpForm {
	pub(super) fn boundary(&self) -> &str { self.form.as_ref().unwrap().boundary() }

	pub(super) fn field(&mut self, name: String, value: String) {
		self.fields.push((name.clone(), value.clone()));
		self.form = self.form.take().map(|form| form.text(name, value));
	}

	pub(super) fn part(&mut self, part: HttpPart) -> reqwest::Result<()> {
		let name = part.name.clone();
		let part: Part = part.try_into()?;

		self.form = self.form.take().map(|form| form.part(name, part));
		Ok(())
	}

	pub(super) fn to_urlencoded(&self) -> String {
		self
			.fields
			.iter()
			.map(|(name, value)| format!("{}={}", encode(name), encode(value)))
			.collect::<Vec<_>>()
			.join("&")
	}
}

fn encode(value: &str) -> String {
	const SET: &AsciiSet = &NON_ALPHANUMERIC.remove(b'*').remove(b'-').remove(b'.').remove(b'_');

	let mut s = String::with_capacity(value.len());
	for part in utf8_percent_encode(value, SET) {
		s.push_str(if part == "%20" { "+" } else { part });
	}
	s
}
