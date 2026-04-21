use std::{borrow::Cow, fmt::{self, Display}, ops::{Deref, DerefMut}, str::FromStr};

use anyhow::{Result, bail};
use serde_with::DeserializeFromStr;

use crate::{Layer, SStr, Source, data::{Data, DataAny, DataKey}, event::{Cmd, Replier}};

#[derive(Clone, Debug, Default, DeserializeFromStr)]
pub struct Action {
	pub cmd:    Cmd,
	pub layer:  Layer,
	pub source: Source,
}

impl Deref for Action {
	type Target = Cmd;

	fn deref(&self) -> &Self::Target { &self.cmd }
}

impl DerefMut for Action {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.cmd }
}

impl Action {
	pub fn new<N>(name: N, source: Source, layer: Layer) -> Result<Self>
	where
		N: Into<SStr>,
	{
		let cow: SStr = name.into();
		let (layer, name) = match cow.find(':') {
			None => (layer, cow),
			Some(i) => (cow[..i].parse()?, match cow {
				Cow::Borrowed(s) => Cow::Borrowed(&s[i + 1..]),
				Cow::Owned(mut s) => {
					s.drain(..i + 1);
					Cow::Owned(s)
				}
			}),
		};

		Ok(Self { cmd: Cmd { name, args: Default::default() }, layer, source })
	}

	pub fn new_relay<N>(name: N) -> Self
	where
		N: Into<SStr>,
	{
		Self::new(name, Source::Relay, Layer::Null).unwrap_or(Self::null())
	}

	pub fn new_relay_args<N, D, I>(name: N, args: I) -> Self
	where
		N: Into<SStr>,
		D: Into<Data>,
		I: IntoIterator<Item = D>,
	{
		let mut action = Self::new(name, Source::Relay, Layer::Null).unwrap_or(Self::null());
		action.args =
			args.into_iter().enumerate().map(|(i, a)| (DataKey::Integer(i as i64), a.into())).collect();
		action
	}

	fn null() -> Self { Self { cmd: Cmd::null(), layer: Layer::Null, source: Source::Unknown } }

	pub fn len(&self) -> usize { self.args.len() }

	pub fn is_empty(&self) -> bool { self.args.is_empty() }

	// --- With
	pub fn with(mut self, name: impl Into<DataKey>, value: impl Into<Data>) -> Self {
		self.args.insert(name.into(), value.into());
		self
	}

	pub fn with_list<I>(mut self, name: impl Into<DataKey>, values: I) -> Self
	where
		I: IntoIterator,
		I::Item: Into<Data>,
	{
		self.args.insert(name.into(), values.into_iter().map(Into::into).collect());
		self
	}

	pub fn with_any(mut self, name: impl Into<DataKey>, data: impl DataAny) -> Self {
		self.args.insert(name.into(), Data::Any(Box::new(data)));
		self
	}

	pub fn with_opt(mut self, name: impl Into<DataKey>, value: Option<impl Into<Data>>) -> Self {
		if let Some(value) = value {
			self.args.insert(name.into(), value.into());
		}
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

	pub fn with_replier(mut self, tx: Replier) -> Self {
		self.args.insert("replier".into(), Data::Any(Box::new(tx)));
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

	pub fn any<T: 'static>(&self, name: impl Into<DataKey>) -> Option<&T> {
		self.args.get(&name.into())?.as_any()
	}

	pub fn replier(&self) -> Option<&Replier> { self.any("replier") }

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

	pub fn take_any_iter<T: 'static>(&mut self) -> impl Iterator<Item = T> {
		(0..self.len()).filter_map(|i| self.args.remove(&DataKey::from(i))?.into_any())
	}

	pub fn take_replier(&mut self) -> Option<Replier> { self.take_any("replier") }
}

impl Display for Action {
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

impl FromStr for Action {
	type Err = anyhow::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let cmd = Cmd::from_str(s)?;

		let mut me = Self::new(cmd.name, Source::Unknown, Layer::Null)?;
		me.args = cmd.args;

		Ok(me)
	}
}
