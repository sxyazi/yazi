use std::{any::Any, collections::BTreeMap, fmt::{self, Debug, Display}};

use anyhow::bail;
use serde::{de::{self, Visitor}, Deserializer};

#[derive(Debug, Default)]
pub struct Exec {
	pub cmd:   String,
	pub args:  Vec<String>,
	pub named: BTreeMap<String, String>,
	pub data:  Option<Box<dyn Any + Send>>,
}

impl TryFrom<&str> for Exec {
	type Error = anyhow::Error;

	fn try_from(s: &str) -> Result<Self, Self::Error> {
		let s = shell_words::split(s)?;
		if s.is_empty() {
			bail!("`exec` cannot be empty");
		}

		let mut exec = Self { cmd: s[0].clone(), ..Default::default() };
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

impl Display for Exec {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.cmd)?;
		if !self.args.is_empty() {
			write!(f, " {}", self.args.join(" "))?;
		}
		for (k, v) in &self.named {
			write!(f, " --{k}")?;
			if !v.is_empty() {
				write!(f, "={v}")?;
			}
		}
		Ok(())
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

impl Exec {
	#[inline]
	pub fn call(cwd: &str, args: Vec<String>) -> Self {
		Exec { cmd: cwd.to_owned(), args, ..Default::default() }
	}

	#[inline]
	pub fn call_named(cwd: &str, named: BTreeMap<String, String>) -> Self {
		Exec { cmd: cwd.to_owned(), named, ..Default::default() }
	}

	#[inline]
	pub fn vec(self) -> Vec<Self> { vec![self] }

	#[inline]
	pub fn with(mut self, name: impl ToString, value: impl ToString) -> Self {
		self.named.insert(name.to_string(), value.to_string());
		self
	}

	#[inline]
	pub fn with_bool(mut self, name: impl ToString, state: bool) -> Self {
		if state {
			self.named.insert(name.to_string(), Default::default());
		}
		self
	}
}
