use std::{collections::BTreeMap, fmt};

use serde::{de::{self, Visitor}, Deserializer};

#[derive(Clone, Debug, Default)]
pub struct Exec {
	pub cmd:   String,
	pub args:  Vec<String>,
	pub named: BTreeMap<String, String>,
}

impl From<&str> for Exec {
	fn from(value: &str) -> Self {
		let mut exec = Self::default();
		for x in value.split_whitespace() {
			if x.starts_with("--") {
				let mut it = x[2..].splitn(2, '=');
				let name = it.next().unwrap();
				let value = it.next().unwrap_or("");
				exec.named.insert(name.to_string(), value.to_string());
			} else if exec.cmd.is_empty() {
				exec.cmd = x.to_string();
			} else {
				exec.args.push(x.to_string());
			}
		}
		exec
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
				formatter.write_str("a command string, e.g. tab_switch 0")
			}

			fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
			where
				A: de::SeqAccess<'de>,
			{
				let mut execs = Vec::new();
				while let Some(value) = &seq.next_element::<String>()? {
					execs.push(Exec::from(value.as_str()));
				}
				Ok(execs)
			}

			fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
			where
				E: de::Error,
			{
				Ok(value.split(';').map(Exec::from).collect())
			}
		}

		deserializer.deserialize_any(ExecVisitor)
	}
}
