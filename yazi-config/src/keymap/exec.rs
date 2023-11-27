use std::fmt;

use anyhow::{bail, Result};
use serde::{de::{self, Visitor}, Deserializer};
use yazi_shared::event::Exec;

pub(super) fn exec_deserialize<'de, D>(deserializer: D) -> Result<Vec<Exec>, D::Error>
where
	D: Deserializer<'de>,
{
	struct ExecVisitor;

	fn parse(s: &str) -> Result<Exec> {
		let s = shell_words::split(s)?;
		if s.is_empty() {
			bail!("`exec` cannot be empty");
		}

		let mut exec = Exec { cmd: s[0].clone(), ..Default::default() };
		for arg in s.into_iter().skip(1) {
			if arg.starts_with("--") {
				let mut arg = arg.splitn(2, '=');
				let key = arg.next().unwrap().trim_start_matches('-');
				let val = arg.next().unwrap_or("").to_string();
				exec.named.insert(key.to_string(), val);
			} else {
				exec.args.push(arg);
			}
		}
		Ok(exec)
	}

	impl<'de> Visitor<'de> for ExecVisitor {
		type Value = Vec<Exec>;

		fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
			formatter.write_str("a exec string, e.g. `tab_switch 0`")
		}

		fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
		where
			A: de::SeqAccess<'de>,
		{
			let mut execs = Vec::new();
			while let Some(value) = &seq.next_element::<String>()? {
				execs.push(parse(value).map_err(de::Error::custom)?);
			}
			Ok(execs)
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
