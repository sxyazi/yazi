use std::fmt::{Debug, Formatter};

#[derive(Clone)]
pub enum ExecItem {
	Word(String),
	Arg(String, bool),
	Str(String, usize),
}

impl ExecItem {
	#[inline]
	pub(super) fn push(&mut self, c: char) {
		match self {
			ExecItem::Word(s) => s.push(c),
			ExecItem::Arg(s, _) => s.push(c),
			ExecItem::Str(s, _) => s.push(c),
		}
	}

	#[inline]
	pub(super) fn is_empty(&self) -> bool {
		match self {
			ExecItem::Word(s) => s.is_empty(),
			ExecItem::Arg(..) => false,
			ExecItem::Str(s, _) => s.is_empty(),
		}
	}

	#[inline]
	pub(super) fn slash(&self) -> Option<&str> {
		match self {
			ExecItem::Str(_, n) => {
				if *n == 0 {
					return None;
				}

				let s = "\\\\".repeat(*n);
				Some(&s[..s.len() - 1])
			}
			_ => None,
		}
	}
}

impl Debug for ExecItem {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			ExecItem::Word(s) => write!(f, "Word({s})"),
			ExecItem::Arg(s, _) => write!(f, "Arg({s})"),
			ExecItem::Str(s, n) => write!(f, "Str{n}({s})"),
		}
	}
}

impl ToString for ExecItem {
	fn to_string(&self) -> String {
		match self {
			ExecItem::Word(s) => s.clone(),
			ExecItem::Arg(s, b) => format!("-{}{s}", if *b { "-" } else { "" }),
			ExecItem::Str(s, n) => {
				let slash = if let Some(s) = self.slash() {
					s
				} else {
					return s.to_owned();
				};

				if s == "'" || s == "\"" {
					return format!("{slash}{s}");
				}

				let mut s = s.clone();
				if let Some(sub) = s.strip_prefix('"') {
					s = format!("{slash}\"{sub}");
				}
				if let Some(sub) = s.strip_suffix('"') {
					s = format!("{sub}{slash}\"");
				}
				if let Some(sub) = s.strip_prefix('\'') {
					s = format!("{slash}'{sub}");
				}
				if let Some(sub) = s.strip_suffix('\'') {
					s = format!("{sub}{slash}'");
				}
				s
			}
		}
	}
}
