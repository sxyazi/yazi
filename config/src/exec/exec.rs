use std::{collections::BTreeSet, fmt::{self, Debug}};

use regex::Regex;
use serde::{de::{self, Visitor}, Deserializer};

use super::ExecItem;

#[derive(Clone, Debug)]
pub struct Exec {
	items: Vec<ExecItem>,
}

impl From<&str> for Exec {
	fn from(s: &str) -> Self { Self { items: Self::parse(s) } }
}

impl ToString for Exec {
	fn to_string(&self) -> String {
		self.items.iter().map(|i| i.to_string()).collect::<Vec<_>>().join("")
	}
}

impl Exec {
	pub fn parse(s: &str) -> Vec<ExecItem> {
		let mut item = ExecItem::Word(Default::default());
		let mut last = b'\0';
		let mut esc = 0;

		let mut items = vec![];
		#[inline]
		fn add(items: &mut Vec<ExecItem>, item: ExecItem) {
			if !item.is_empty() {
				items.push(item);
			}
		}

		for c in s.trim().chars() {
			if last == b'\\' && !matches!(c, '\\' | '"' | '\'') {
				item.push('\\');
				esc = 0;
				last = b'\0';
			}

			match c {
				' ' => match item {
					ExecItem::Str(ref mut s, ..) => s.push(c),
					ExecItem::Word(ref mut s) if last == b'\\' => {
						s.push('\\');
						s.push(c);
					}
					_ => {
						item.push(c);
						add(&mut items, item);
						item = ExecItem::Word(Default::default());
					}
				},
				'-' => match item {
					ExecItem::Word(ref mut w) => {
						if w.is_empty() {
							item = ExecItem::Arg(Default::default(), false);
						} else {
							w.push(c);
						}
					}
					ExecItem::Arg(_, ref mut b) => *b = true,
					ExecItem::Str(ref mut s, ..) => s.push(c),
				},
				'\\' => {
					if last == b'\\' {
						esc += 1;
						last = b'\0';
					} else {
						last = b'\\';
					}
				}
				'"' | '\'' => {
					if last == b'\\' {
						esc += 1;
						last = b'\0';
					}
					if matches!(item, ExecItem::Str(_, e) if e == esc) {
						item.push(c);
						add(&mut items, item);
						item = ExecItem::Str(Default::default(), esc);
					} else {
						add(&mut items, item);
						item = ExecItem::Str(c.to_string(), esc);
					}
					esc = 0;
				}
				c => {
					item.push(c);
				}
			}
		}

		add(&mut items, item);
		items
	}

	pub fn build(&self, args: Vec<String>) -> String {
		let re = Regex::new(r"\$(\d+|\*)").unwrap();
		let mut occurs = BTreeSet::new();
		let mut replace = |s: &mut String| {
			*s = re
				.replace_all(s, |caps: &regex::Captures| {
					let idx = caps.get(1).unwrap().as_str();
					if idx == "*" {
						return args
							.iter()
							.enumerate()
							.filter(|(i, _)| !occurs.contains(i))
							.map(|(_, s)| s.as_str())
							.collect::<Vec<_>>()
							.join(" ");
					}

					if let Ok(idx) = idx.parse::<usize>() {
						if idx < args.len() {
							occurs.insert(idx);
							return args[idx].to_owned();
						}
					}

					Default::default()
				})
				.into_owned();
		};

		let items = self
			.items
			.iter()
			.cloned()
			.map(|mut i| {
				match i {
					ExecItem::Word(ref mut s) => replace(s),
					ExecItem::Arg(..) => (),
					ExecItem::Str(ref mut s, _) => replace(s),
				}
				i
			})
			.collect::<Vec<_>>();

		Self { items }.to_string()
	}

	pub fn has() {}

	pub fn arg() {}

	pub fn named() {}
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
				Ok(vec![Exec::from(value)])
			}
		}

		deserializer.deserialize_any(ExecVisitor)
	}
}
