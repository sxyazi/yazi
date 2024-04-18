use std::{any::Any, collections::HashMap, fmt::{self, Display}};

use super::Data;

#[derive(Debug, Default)]
pub struct Cmd {
	pub name: String,
	pub args: HashMap<String, Data>,
}

impl Cmd {
	#[inline]
	pub fn new(name: &str) -> Self { Self { name: name.to_owned(), ..Default::default() } }

	#[inline]
	pub fn args(name: &str, args: Vec<String>) -> Self {
		Self {
			name: name.to_owned(),
			args: args.into_iter().enumerate().map(|(i, s)| (i.to_string(), Data::String(s))).collect(),
		}
	}

	#[inline]
	pub fn with(mut self, name: impl ToString, value: impl ToString) -> Self {
		self.args.insert(name.to_string(), Data::String(value.to_string()));
		self
	}

	#[inline]
	pub fn with_bool(mut self, name: impl ToString, state: bool) -> Self {
		self.args.insert(name.to_string(), Data::Boolean(state));
		self
	}

	#[inline]
	pub fn with_any(mut self, name: impl ToString, data: impl Any + Send) -> Self {
		self.args.insert(name.to_string(), Data::Any(Box::new(data)));
		self
	}

	#[inline]
	pub fn with_name(mut self, name: impl ToString) -> Self {
		self.name = name.to_string();
		self
	}

	#[inline]
	pub fn get_str(&self, name: &str) -> Option<&str> { self.args.get(name).and_then(Data::as_str) }

	#[inline]
	pub fn get_bool(&self, name: &str) -> bool {
		self.args.get(name).and_then(Data::as_bool).unwrap_or(false)
	}

	#[inline]
	pub fn take_data(&mut self, name: &str) -> Option<Data> { self.args.remove(name) }

	#[inline]
	pub fn take_str(&mut self, name: &str) -> Option<String> {
		if let Some(Data::String(s)) = self.args.remove(name) { Some(s) } else { None }
	}

	#[inline]
	pub fn take_first_str(&mut self) -> Option<String> {
		if let Some(Data::String(s)) = self.args.remove("0") { Some(s) } else { None }
	}

	#[inline]
	pub fn take_any<T: 'static>(&mut self, name: &str) -> Option<T> {
		self.args.remove(name).and_then(|d| d.into_any())
	}

	pub fn shallow_clone(&self) -> Self {
		let args = self
			.args
			.iter()
			.filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), Data::String(s.to_owned()))))
			.collect();

		Self { name: self.name.clone(), args }
	}
}

impl Display for Cmd {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.name)?;
		for (k, v) in &self.args {
			if k.as_bytes().first().is_some_and(|b| b.is_ascii_digit()) {
				if let Some(s) = v.as_str() {
					write!(f, " {s}")?;
				}
			} else if v.as_bool().is_some_and(|b| b) {
				write!(f, " --{k}")?;
			} else if let Some(s) = v.as_str() {
				write!(f, " --{k}={s}")?;
			}
		}
		Ok(())
	}
}
