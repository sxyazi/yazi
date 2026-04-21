use std::{mem, str::FromStr};

use anyhow::{Result, bail};
use hashbrown::HashMap;
use serde_with::DeserializeFromStr;

use crate::{SStr, data::{Data, DataKey}};

#[derive(Clone, Debug, Default, DeserializeFromStr)]
pub struct Cmd {
	pub name: SStr,
	pub args: HashMap<DataKey, Data>,
}

impl FromStr for Cmd {
	type Err = anyhow::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let (mut words, last) = crate::shell::unix::split(s, true)?;
		if words.is_empty() || words[0].is_empty() {
			bail!("command name cannot be empty");
		}

		Ok(Self {
			name: mem::take(&mut words[0]).into(),
			args: Self::parse_args(words.into_iter().skip(1), last)?,
		})
	}
}

impl Cmd {
	pub fn null() -> Self { Self { name: "null".into(), ..Default::default() } }

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

				Ok((key.to_owned().into(), val))
			})
			.collect()
	}
}
