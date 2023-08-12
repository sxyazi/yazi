use std::{collections::BTreeMap, fmt::{self, Debug}};

use anyhow::bail;
use serde::{de::{self, Visitor}, Deserializer};

#[derive(Clone, Debug)]
pub struct Exec {
	pub cmd:   String,
	pub args:  Vec<String>,
	pub named: BTreeMap<String, String>,
}

impl TryFrom<&str> for Exec {
	type Error = anyhow::Error;

	fn try_from(s: &str) -> Result<Self, Self::Error> {
		let s = shell_words::split(s)?;
		if s.is_empty() {
			bail!("`exec` cannot be empty");
		}

		let mut exec = Self { cmd: s[0].clone(), args: Vec::new(), named: BTreeMap::new() };
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
}

impl ToString for Exec {
	fn to_string(&self) -> String {
		let mut s = Vec::with_capacity(self.args.len() + self.named.len() + 1);
		s.push(self.cmd.clone());
		s.extend(self.args.iter().cloned());
		for (key, val) in self.named.iter() {
			s.push(format!("--{}={}", key, val));
		}

		shell_words::join(s)
	}
}

impl Exec {
	pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<Exec>, D::Error>
	where
		D: Deserializer<'de>,
	{
		struct ExecVisitor;

		impl<'de> Visitor<'de> for ExecVisitor {
			type Value = Vec<Exec>;

			fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
				formatter.write_str("a exec string, e.g. tab_switch 0")
			}

			fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
			where
				A: de::SeqAccess<'de>,
			{
				let mut execs = Vec::new();
				while let Some(value) = &seq.next_element::<String>()? {
					execs.push(Exec::try_from(value.as_str()).map_err(de::Error::custom)?);
				}
				Ok(execs)
			}

			fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
			where
				E: de::Error,
			{
				Ok(vec![Exec::try_from(value).map_err(de::Error::custom)?])
			}
		}

		deserializer.deserialize_any(ExecVisitor)
	}
}
