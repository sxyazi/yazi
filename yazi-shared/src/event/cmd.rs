use std::{any::Any, collections::HashMap, fmt::{self, Display}, mem, str::FromStr};

use anyhow::bail;
use serde::{Deserialize, de};

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
	pub fn args(name: &str, args: &[impl ToString]) -> Self {
		Self {
			name: name.to_owned(),
			args: args
				.iter()
				.enumerate()
				.map(|(i, s)| (i.to_string(), Data::String(s.to_string())))
				.collect(),
		}
	}

	// --- With
	#[inline]
	pub fn with(mut self, name: impl ToString, value: impl ToString) -> Self {
		self.args.insert(name.to_string(), Data::String(value.to_string()));
		self
	}

	#[inline]
	pub fn with_opt(mut self, name: impl ToString, value: Option<impl ToString>) -> Self {
		if let Some(v) = value {
			self.args.insert(name.to_string(), Data::String(v.to_string()));
		}
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

	// --- Get
	#[inline]
	pub fn get(&self, name: &str) -> Option<&Data> { self.args.get(name) }

	#[inline]
	pub fn str(&self, name: &str) -> Option<&str> { self.get(name).and_then(Data::as_str) }

	#[inline]
	pub fn bool(&self, name: &str) -> bool { self.maybe_bool(name).unwrap_or(false) }

	#[inline]
	pub fn maybe_bool(&self, name: &str) -> Option<bool> { self.get(name).and_then(Data::as_bool) }

	#[inline]
	pub fn first(&self) -> Option<&Data> { self.get("0") }

	// --- Take
	#[inline]
	pub fn take(&mut self, name: &str) -> Option<Data> { self.args.remove(name) }

	#[inline]
	pub fn take_str(&mut self, name: &str) -> Option<String> {
		if let Some(Data::String(s)) = self.take(name) { Some(s) } else { None }
	}

	#[inline]
	pub fn take_first(&mut self) -> Option<Data> { self.take("0") }

	#[inline]
	pub fn take_first_str(&mut self) -> Option<String> {
		if let Some(Data::String(s)) = self.take_first() { Some(s) } else { None }
	}

	#[inline]
	pub fn take_any<T: 'static>(&mut self, name: &str) -> Option<T> {
		self.args.remove(name).and_then(|d| d.into_any())
	}

	// --- Clone
	pub fn shallow_clone(&self) -> Self {
		Self {
			name: self.name.clone(),
			args: self
				.args
				.iter()
				.filter_map(|(k, v)| v.shallow_clone().map(|v| (k.clone(), v)))
				.collect(),
		}
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

impl FromStr for Cmd {
	type Err = anyhow::Error;

	#[allow(clippy::explicit_counter_loop)]
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let mut args = shell_words::split(s)?;
		if args.is_empty() || args[0].is_empty() {
			bail!("command name cannot be empty");
		}

		let mut cmd = Cmd { name: mem::take(&mut args[0]), ..Default::default() };
		let mut i = 0usize;
		for arg in args.into_iter().skip(1) {
			let Some(arg) = arg.strip_prefix("--") else {
				cmd.args.insert(i.to_string(), Data::String(arg));
				i += 1;
				continue;
			};

			let mut parts = arg.splitn(2, '=');
			let Some(key) = parts.next().map(|s| s.to_owned()) else {
				bail!("invalid argument: {arg}");
			};

			if let Some(val) = parts.next() {
				cmd.args.insert(key, Data::String(val.to_owned()));
			} else {
				cmd.args.insert(key, Data::Boolean(true));
			}
		}
		Ok(cmd)
	}
}

impl<'de> Deserialize<'de> for Cmd {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		<_>::from_str(&String::deserialize(deserializer)?).map_err(de::Error::custom)
	}
}
