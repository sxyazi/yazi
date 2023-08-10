use std::fmt::{Debug, Formatter};

#[derive(Debug)]
pub struct ExecNew {
	items: Vec<ExecItem>,
}

pub enum ExecItem {
	Word(String),
	Arg(String, bool),
	Str(String, usize),
}

impl ExecItem {
	#[inline]
	fn push(&mut self, c: char) {
		match self {
			ExecItem::Word(s) => s.push(c),
			ExecItem::Arg(s, _) => s.push(c),
			ExecItem::Str(s, _) => s.push(c),
		}
	}

	#[inline]
	fn is_empty(&self) -> bool {
		match self {
			ExecItem::Word(s) => s.is_empty(),
			ExecItem::Arg(..) => false,
			ExecItem::Str(s, _) => s.is_empty(),
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
				if *n == 0 {
					return s.to_string();
				}

				let rep = "\\\\".repeat(*n);
				let rep = rep[..rep.len() - 1].to_string();

				if s == "'" || s == "\"" {
					return format!("{rep}{s}");
				}

				let mut s = s.clone();
				if let Some(ss) = s.strip_prefix('"') {
					s = format!("{rep}\"{ss}");
				}
				if let Some(ss) = s.strip_suffix('"') {
					s = format!("{ss}{rep}\"");
				}
				if let Some(ss) = s.strip_prefix('\'') {
					s = format!("{rep}'{ss}");
				}
				if let Some(ss) = s.strip_suffix('\'') {
					s = format!("{ss}{rep}'");
				}
				s
			}
		}
	}
}

impl ExecNew {
	pub fn parse(s: &str) -> Self {
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
		Self { items }
	}
}

impl ToString for ExecNew {
	fn to_string(&self) -> String {
		self.items.iter().map(|i| i.to_string()).collect::<Vec<_>>().join("")
	}
}

#[test]
fn test() {
	fn assert(a: &str) {
		let exec = ExecNew::parse(a);

		let a = a.trim();
		let b = exec.to_string().trim().to_string();

		println!("{:?}", exec);
		if a != b {
			println!("A: {}", a);
			println!("B: {}", b);
		}
	}

	assert(r#"  echo 123 "foo" 'bar'  "#);
	assert(r#"  sh -c "sh -c \"\";"  "#);
	assert(r#"  aaa - "bbb --opt \"ccc \\\"Meow\\\"\""  "#);
	assert(r#"  python4 --code 'bash -c "echo \'\\\'\'"';  "#);
	assert(r#"  sh -c "sh -c \"exiftool $0; echo \\\"\nPress enter to exit: \\\"; read\""  "#);
}
