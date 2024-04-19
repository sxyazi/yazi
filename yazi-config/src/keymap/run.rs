use std::{fmt, mem};

use anyhow::{bail, Result};
use serde::{de::{self, Visitor}, Deserializer};
use yazi_shared::event::{Cmd, Data};

pub(super) fn run_deserialize<'de, D>(deserializer: D) -> Result<Vec<Cmd>, D::Error>
where
	D: Deserializer<'de>,
{
	struct RunVisitor;

	#[allow(clippy::explicit_counter_loop)]
	fn parse(s: &str) -> Result<Cmd> {
		let mut args = shell_words::split(s)?;
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

	impl<'de> Visitor<'de> for RunVisitor {
		type Value = Vec<Cmd>;

		fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
			formatter.write_str("a `run` string or array of strings within keymap.toml")
		}

		fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
		where
			A: de::SeqAccess<'de>,
		{
			let mut cmds = vec![];
			while let Some(value) = &seq.next_element::<String>()? {
				cmds.push(parse(value).map_err(de::Error::custom)?);
			}
			if cmds.is_empty() {
				return Err(de::Error::custom("`run` within keymap.toml cannot be empty"));
			}
			Ok(cmds)
		}

		fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
		where
			E: de::Error,
		{
			Ok(vec![parse(value).map_err(de::Error::custom)?])
		}
	}

	deserializer.deserialize_any(RunVisitor)
}
