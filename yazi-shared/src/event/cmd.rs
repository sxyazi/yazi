use std::{any::Any, borrow::Cow, fmt::{self, Display}, mem, str::FromStr};

use anyhow::{Result, anyhow, bail};
use hashbrown::HashMap;
use serde::{Deserialize, de};

use crate::{Layer, SStr, Source, data::{Data, DataKey}};

#[derive(Debug, Default)]
pub struct Cmd {
	pub name:   SStr,
	pub args:   HashMap<DataKey, Data>,
	pub layer:  Layer,
	pub source: Source,
}

impl Cmd {
	pub fn new<N>(name: N, source: Source, default: Option<Layer>) -> Result<Self>
	where
		N: Into<SStr>,
	{
		let cow: SStr = name.into();
		let (layer, name) = match cow.find(':') {
			None => (default.ok_or_else(|| anyhow!("Cannot infer layer from command name: {cow}"))?, cow),
			Some(i) => (cow[..i].parse()?, match cow {
				Cow::Borrowed(s) => Cow::Borrowed(&s[i + 1..]),
				Cow::Owned(mut s) => {
					s.drain(..i + 1);
					Cow::Owned(s)
				}
			}),
		};

		Ok(Self { name, args: Default::default(), layer, source })
	}

	pub fn new_relay<N>(name: N) -> Self
	where
		N: Into<SStr>,
	{
		Self::new(name, Source::Relay, None).unwrap_or(Self::null())
	}

	pub fn new_relay_args<N, D, I>(name: N, args: I) -> Self
	where
		N: Into<SStr>,
		D: Into<Data>,
		I: IntoIterator<Item = D>,
	{
		let mut cmd = Self::new(name, Source::Relay, None).unwrap_or(Self::null());
		cmd.args =
			args.into_iter().enumerate().map(|(i, a)| (DataKey::Integer(i as i64), a.into())).collect();
		cmd
	}

	fn null() -> Self { Self { name: Cow::Borrowed("null"), ..Default::default() } }

	pub fn len(&self) -> usize { self.args.len() }

	pub fn is_empty(&self) -> bool { self.args.is_empty() }

	// --- With
	pub fn with(mut self, name: impl Into<DataKey>, value: impl Into<Data>) -> Self {
		self.args.insert(name.into(), value.into());
		self
	}

	pub fn with_seq<I>(mut self, values: I) -> Self
	where
		I: IntoIterator,
		I::Item: Into<Data>,
	{
		for (i, v) in values.into_iter().enumerate() {
			self.args.insert(DataKey::Integer(i as i64), v.into());
		}
		self
	}

	pub fn with_any(mut self, name: impl Into<DataKey>, data: impl Any + Send + Sync) -> Self {
		self.args.insert(name.into(), Data::Any(Box::new(data)));
		self
	}

	// --- Get
	pub fn get<'a, T>(&'a self, name: impl Into<DataKey>) -> Result<T>
	where
		T: TryFrom<&'a Data>,
		T::Error: Into<anyhow::Error>,
	{
		let name = name.into();
		match self.args.get(&name) {
			Some(data) => data.try_into().map_err(Into::into),
			None => bail!("argument not found: {:?}", name),
		}
	}

	pub fn str(&self, name: impl Into<DataKey>) -> &str { self.get(name).unwrap_or_default() }

	pub fn bool(&self, name: impl Into<DataKey>) -> bool { self.get(name).unwrap_or(false) }

	pub fn first<'a, T>(&'a self) -> Result<T>
	where
		T: TryFrom<&'a Data>,
		T::Error: Into<anyhow::Error>,
	{
		self.get(0)
	}

	pub fn second<'a, T>(&'a self) -> Result<T>
	where
		T: TryFrom<&'a Data>,
		T::Error: Into<anyhow::Error>,
	{
		self.get(1)
	}

	pub fn seq<'a, T>(&'a self) -> Vec<T>
	where
		T: TryFrom<&'a Data>,
	{
		let mut seq = Vec::with_capacity(self.len());
		for i in 0..self.len() {
			if let Ok(data) = self.get::<&Data>(i)
				&& let Ok(v) = data.try_into()
			{
				seq.push(v);
			} else {
				break;
			}
		}
		seq
	}

	// --- Take
	pub fn take<T>(&mut self, name: impl Into<DataKey>) -> Result<T>
	where
		T: TryFrom<Data>,
		T::Error: Into<anyhow::Error>,
	{
		let name = name.into();
		match self.args.remove(&name) {
			Some(data) => data.try_into().map_err(Into::into),
			None => bail!("argument not found: {:?}", name),
		}
	}

	pub fn take_first<T>(&mut self) -> Result<T>
	where
		T: TryFrom<Data>,
		T::Error: Into<anyhow::Error>,
	{
		self.take(0)
	}

	pub fn take_second<T>(&mut self) -> Result<T>
	where
		T: TryFrom<Data>,
		T::Error: Into<anyhow::Error>,
	{
		self.take(1)
	}

	pub fn take_seq<T>(&mut self) -> Vec<T>
	where
		T: TryFrom<Data>,
	{
		let mut seq = Vec::with_capacity(self.len());
		for i in 0..self.len() {
			if let Ok(data) = self.take::<Data>(i)
				&& let Ok(v) = data.try_into()
			{
				seq.push(v);
			} else {
				break;
			}
		}
		seq
	}

	pub fn take_any<T: 'static>(&mut self, name: impl Into<DataKey>) -> Option<T> {
		self.args.remove(&name.into())?.into_any()
	}

	pub fn take_any2<T: 'static>(&mut self, name: impl Into<DataKey>) -> Option<Result<T>> {
		self.args.remove(&name.into()).map(Data::into_any2)
	}

	// Parse
	pub fn parse_args<I>(words: I, last: Option<String>) -> Result<HashMap<DataKey, Data>>
	where
		I: IntoIterator<Item = String>,
	{
		let mut i = 0i64;
		words
			.into_iter()
			.map(|s| (s, true))
			.chain(last.into_iter().map(|s| (s, false)))
			.map(|(word, normal)| {
				let Some(arg) = word.strip_prefix("--").filter(|&s| normal && !s.is_empty()) else {
					i += 1;
					return Ok((DataKey::Integer(i - 1), word.into()));
				};

				let mut parts = arg.splitn(2, '=');
				let key = parts.next().expect("at least one part");
				let val = parts.next().map_or(Data::Boolean(true), Data::from);

				Ok((DataKey::from(key.to_owned()), val))
			})
			.collect()
	}
}

impl Display for Cmd {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.name)?;

		for i in 0..self.args.len() {
			let Ok(s) = self.get::<&str>(i) else { break };
			write!(f, " {s}")?;
		}

		for (k, v) in &self.args {
			if let DataKey::String(k) = k {
				if v.try_into().is_ok_and(|b| b) {
					write!(f, " --{k}")?;
				} else if let Some(s) = v.as_str() {
					write!(f, " --{k}={s}")?;
				}
			}
		}

		Ok(())
	}
}

impl FromStr for Cmd {
	type Err = anyhow::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let (mut words, last) = crate::shell::unix::split(s, true)?;
		if words.is_empty() || words[0].is_empty() {
			bail!("command name cannot be empty");
		}

		let mut me = Self::new(mem::take(&mut words[0]), Default::default(), Some(Default::default()))?;
		me.args = Self::parse_args(words.into_iter().skip(1), last)?;
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
