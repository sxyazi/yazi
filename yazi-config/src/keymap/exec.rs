use std::fmt;

use anyhow::{bail, Result};
use serde::{de::{self, Visitor}, Deserializer};
use yazi_shared::event::Cmd;

pub(super) fn exec_deserialize<'de, D>(deserializer: D) -> Result<Vec<Cmd>, D::Error>
where
	D: Deserializer<'de>,
{
	struct ExecVisitor;

	fn parse(s: &str) -> Result<Cmd> {
		let s = shell_words::split(s)?;
		if s.is_empty() {
			bail!("`exec` cannot be empty");
		}

		let mut cmd = Cmd { name: s[0].clone(), ..Default::default() };
		for arg in s.into_iter().skip(1) {
			if arg.starts_with("--") {
				let mut arg = arg.splitn(2, '=');
				let key = arg.next().unwrap().trim_start_matches('-');
				let val = arg.next().unwrap_or("").to_string();
				cmd.named.insert(key.to_string(), val);
			} else {
				cmd.args.push(arg);
			}
		}
		Ok(cmd)
	}

	impl<'de> Visitor<'de> for ExecVisitor {
		type Value = Vec<Cmd>;

		fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
			formatter.write_str("a `exec` string or array of strings within [keymap]")
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
				return Err(de::Error::custom("`exec` within [keymap] cannot be empty"));
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

	deserializer.deserialize_any(ExecVisitor)
}
