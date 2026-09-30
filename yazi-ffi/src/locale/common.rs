use super::{Locale, Width};

#[derive(Clone, Copy)]
pub(super) enum Field {
	Year,
	Month,
	Day,
	Hour,
	Minute,
	Second,
	Meridiem,
	Ignore,
}

pub(super) struct Part {
	pub(super) text:  String,
	pub(super) field: Option<Field>,
}

struct Item<'a> {
	field:     Field,
	token:     &'a str,
	prefix:    String,
	suffix:    String,
	connector: String,
}

impl Field {
	fn group(self) -> Option<bool> {
		match self {
			Self::Year | Self::Month | Self::Day => Some(true),
			Self::Ignore => None,
			_ => Some(false),
		}
	}
}

impl Item<'_> {
	fn affixes(text: &str) -> (&str, &str, &str) {
		// ponytail: format strings have no literal ownership metadata; use adjacency,
		// not language-specific guesses. Arbitrary prose needs a semantic format source.
		let last =
			text.rsplit_once(char::is_whitespace).map_or(text.len(), |(_, tail)| text.len() - tail.len());
		let suffix = text.len() - text.trim_start_matches(char::is_alphabetic).len();
		let prefix = if text[last..].chars().any(char::is_alphabetic) { last } else { text.len() };
		(&text[..suffix], &text[suffix..prefix], &text[prefix..])
	}
}

impl Locale {
	pub(super) fn has_date(self) -> bool {
		self.year.is_some() || self.month.is_some() || self.day.is_some()
	}

	pub(super) fn has_time(self) -> bool {
		self.hour.is_some() || self.minute.is_some() || self.second.is_some()
	}

	pub(super) fn width(self, field: Field) -> Option<Width> {
		match field {
			Field::Year => self.year,
			Field::Month => self.month,
			Field::Day => self.day,
			Field::Hour => self.hour,
			Field::Meridiem if !self.hour24 => self.hour,
			Field::Minute => self.minute,
			Field::Second => self.second,
			_ => None,
		}
	}

	pub(super) fn render(self, parts: &[Part]) -> String {
		let mut items: Vec<Item> = Vec::new();
		let mut literal = String::new();
		for part in parts {
			let Some(field) = part.field else {
				literal.push_str(&part.text);
				continue;
			};
			let (connector, prefix) = if let Some(previous) = items.last_mut() {
				let (suffix, connector, prefix) = Item::affixes(&literal);
				previous.suffix.push_str(suffix);
				(connector, prefix)
			} else {
				("", literal.trim_start())
			};
			items.push(Item {
				field,
				token: &part.text,
				prefix: prefix.into(),
				suffix: String::new(),
				connector: connector.into(),
			});
			literal.clear();
		}
		if let Some(last) = items.last_mut() {
			last.suffix.push_str(literal.trim_end());
		}

		let mut output = String::new();
		let mut previous: Option<usize> = None;
		for (index, item) in items.iter().enumerate() {
			let Some(width) = self.width(item.field) else { continue };
			if let Some(previous) = previous {
				Self::literal(&items[previous].suffix, &mut output);
				let between = &items[previous + 1..=index];
				let connector = if items[previous].field.group() != item.field.group() {
					between.iter().find(|part| part.field.group() == item.field.group())
				} else {
					between.iter().rev().find(|part| !part.connector.is_empty())
				};
				if let Some(connector) = connector {
					Self::literal(&connector.connector, &mut output);
				}
			}
			Self::literal(&item.prefix, &mut output);
			self.token(item.field, width, item.token, &mut output);
			previous = Some(index);
		}
		if let Some(previous) = previous {
			Self::literal(&items[previous].suffix, &mut output);
		}
		output
	}
}
