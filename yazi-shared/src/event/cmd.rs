use std::{any::Any, borrow::Cow, collections::HashMap, fmt::{self, Display}, str::FromStr};

use anyhow::{Result, bail};
use serde::{Deserialize, de};

use super::{Data, DataKey};
use crate::{Layer, url::Url};

#[derive(Debug, Default)]
pub struct Cmd {
	pub name:  String,
	pub args:  HashMap<DataKey, Data>,
	pub layer: Layer,
}

impl Cmd {
	pub fn new(s: &str) -> Self {
		Self::new_or(s, Default::default())
			.unwrap_or_else(|_| Self { name: "null".to_owned(), ..Default::default() })
	}

	pub fn new_or(s: &str, default: Layer) -> Result<Self> {
		let (layer, name) = match s.split_once(':') {
			Some((l, n)) => (l.parse()?, n),
			None => (default, s),
		};

		Ok(Self { name: name.to_owned(), args: Default::default(), layer })
	}

	pub fn args(name: &str, args: &[impl ToString]) -> Self {
		let mut me = Self::new(name);
		me.args = args
			.iter()
			.enumerate()
			.map(|(i, s)| (DataKey::Integer(i as i64), Data::String(s.to_string())))
			.collect();
		me
	}

	#[inline]
	pub fn len(&self) -> usize { self.args.len() }

	#[inline]
	pub fn is_empty(&self) -> bool { self.args.is_empty() }

	// --- With
	#[inline]
	pub fn with(mut self, name: impl Into<DataKey>, value: impl Into<Data>) -> Self {
		self.args.insert(name.into(), value.into());
		self
	}

	#[inline]
	pub fn with_opt(mut self, name: impl Into<DataKey>, value: Option<impl Into<Data>>) -> Self {
		if let Some(v) = value {
			self.args.insert(name.into(), v.into());
		}
		self
	}

	#[inline]
	pub fn with_bool(mut self, name: impl Into<DataKey>, state: bool) -> Self {
		self.args.insert(name.into(), Data::Boolean(state));
		self
	}

	#[inline]
	pub fn with_any(mut self, name: impl Into<DataKey>, data: impl Any + Send + Sync) -> Self {
		self.args.insert(name.into(), Data::Any(Box::new(data)));
		self
	}

	// --- Get
	#[inline]
	pub fn get(&self, name: impl Into<DataKey>) -> Option<&Data> { self.args.get(&name.into()) }

	#[inline]
	pub fn str(&self, name: impl Into<DataKey>) -> Option<&str> {
		self.get(name).and_then(Data::as_str)
	}

	#[inline]
	pub fn bool(&self, name: impl Into<DataKey>) -> bool { self.maybe_bool(name).unwrap_or(false) }

	#[inline]
	pub fn maybe_bool(&self, name: impl Into<DataKey>) -> Option<bool> {
		self.get(name).and_then(Data::as_bool)
	}

	#[inline]
	pub fn first(&self) -> Option<&Data> { self.get(0) }

	#[inline]
	pub fn first_str(&self) -> Option<&str> { self.str(0) }

	#[inline]
	pub fn second(&self) -> Option<&Data> { self.get(1) }

	#[inline]
	pub fn second_str(&self) -> Option<&str> { self.str(1) }

	// --- Take
	#[inline]
	pub fn take(&mut self, name: impl Into<DataKey>) -> Option<Data> {
		self.args.remove(&name.into())
	}

	#[inline]
	pub fn take_str(&mut self, name: impl Into<DataKey>) -> Option<String> {
		if let Some(Data::String(s)) = self.take(name) { Some(s) } else { None }
	}

	#[inline]
	pub fn take_first(&mut self) -> Option<Data> { self.take(0) }

	#[inline]
	pub fn take_first_str(&mut self) -> Option<String> {
		if let Some(Data::String(s)) = self.take_first() { Some(s) } else { None }
	}

	#[inline]
	pub fn take_first_url(&mut self) -> Option<Url> { self.take_first()?.into_url() }

	#[inline]
	pub fn take_any<T: 'static>(&mut self, name: impl Into<DataKey>) -> Option<T> {
		self.args.remove(&name.into()).and_then(|d| d.into_any())
	}

	// Parse
	pub fn parse_args(
		words: impl Iterator<Item = String>,
		last: Option<String>,
		obase: bool,
	) -> Result<HashMap<DataKey, Data>> {
		let mut i = 0i64;
		words
			.into_iter()
			.map(|s| (s, true))
			.chain(last.into_iter().map(|s| (s, false)))
			.map(|(word, normal)| {
				let Some(arg) = word.strip_prefix("--").filter(|_| normal) else {
					i += 1;
					return Ok((DataKey::Integer(i - obase as i64), Data::String(word)));
				};

				let mut parts = arg.splitn(2, '=');
				let Some(key) = parts.next().map(|s| s.to_owned()) else {
					bail!("invalid argument: {arg}");
				};

				let val = if let Some(val) = parts.next() {
					Data::String(val.to_owned())
				} else {
					Data::Boolean(true)
				};

				Ok((DataKey::String(Cow::Owned(key)), val))
			})
			.collect()
	}
}

impl Display for Cmd {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.name)?;
		for (k, v) in &self.args {
			match k {
				DataKey::Integer(_) => {
					if let Some(s) = v.as_str() {
						write!(f, " {s}")?;
					}
				}
				DataKey::String(k) => {
					if v.as_bool().is_some_and(|b| b) {
						write!(f, " --{k}")?;
					} else if let Some(s) = v.as_str() {
						write!(f, " --{k}={s}")?;
					}
				}
				_ => {}
			}
		}
		Ok(())
	}
}

impl FromStr for Cmd {
	type Err = anyhow::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let (words, last) = crate::shell::split_unix(s, true)?;
		if words.is_empty() || words[0].is_empty() {
			bail!("command name cannot be empty");
		}

		let mut me = Self::new(&words[0]);
		me.args = Cmd::parse_args(words.into_iter().skip(1), last, true)?;
		Ok(me)
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
