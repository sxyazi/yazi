// A natural sort implementation in Rust.
// Copyright (c) 2023, sxyazi.
//
// This is a port of the C version of Martin Pool's `strnatcmp.c`:
// http://sourcefrog.net/projects/natsort/

use std::cmp::Ordering;

use deunicode::deunicode;

macro_rules! return_unless_equal {
	($ord:expr) => {
		match $ord {
			Ordering::Equal => {}
			ord => return ord,
		}
	};
}

#[inline(always)]
fn compare_left(left: &[u8], right: &[u8], li: &mut usize, ri: &mut usize) -> Ordering {
	let mut l;
	let mut r;

	loop {
		l = left.get(*li);
		r = right.get(*ri);

		match (l.is_some_and(|b| b.is_ascii_digit()), r.is_some_and(|b| b.is_ascii_digit())) {
			(true, true) => {
				return_unless_equal!(unsafe { l.unwrap_unchecked().cmp(r.unwrap_unchecked()) })
			}
			(true, false) => return Ordering::Greater,
			(false, true) => return Ordering::Less,
			(false, false) => return Ordering::Equal,
		}

		*li += 1;
		*ri += 1;
	}
}

#[inline(always)]
fn compare_right(left: &[u8], right: &[u8], li: &mut usize, ri: &mut usize) -> Ordering {
	let mut l;
	let mut r;
	let mut bias = Ordering::Equal;

	loop {
		l = left.get(*li);
		r = right.get(*ri);

		match (l.is_some_and(|b| b.is_ascii_digit()), r.is_some_and(|b| b.is_ascii_digit())) {
			(true, true) => {
				if bias == Ordering::Equal {
					bias = unsafe { l.unwrap_unchecked().cmp(r.unwrap_unchecked()) };
				}
			}
			(true, false) => return Ordering::Greater,
			(false, true) => return Ordering::Less,
			(false, false) => return bias,
		}

		*li += 1;
		*ri += 1;
	}
}

pub fn natsort(left: &[u8], right: &[u8], insensitive: bool) -> Ordering {
	let mut li = 0;
	let mut ri = 0;

	let mut l = left.get(li);
	let mut r = right.get(ri);

	macro_rules! left_next {
		() => {{
			li += 1;
			l = left.get(li);
		}};
	}

	macro_rules! right_next {
		() => {{
			ri += 1;
			r = right.get(ri);
		}};
	}

	loop {
		while l.is_some_and(|c| c.is_ascii_whitespace()) {
			left_next!();
		}
		while r.is_some_and(|c| c.is_ascii_whitespace()) {
			right_next!();
		}

		match (l, r) {
			(Some(&ll), Some(&rr)) => {
				if ll.is_ascii_digit() && rr.is_ascii_digit() {
					if ll == b'0' || rr == b'0' {
						return_unless_equal!(compare_left(left, right, &mut li, &mut ri));
					} else {
						return_unless_equal!(compare_right(left, right, &mut li, &mut ri));
					}

					l = left.get(li);
					r = right.get(ri);
					continue;
				}

				if insensitive {
					return_unless_equal!(ll.to_ascii_lowercase().cmp(&rr.to_ascii_lowercase()));
				} else {
					return_unless_equal!(ll.cmp(&rr));
				}
			}
			(Some(_), None) => return Ordering::Greater,
			(None, Some(_)) => return Ordering::Less,
			(None, None) => return Ordering::Equal,
		}

		left_next!();
		right_next!();
	}
}

pub fn deunicode_natsort(left: &[u8], right: &[u8], insensitive: bool) -> Ordering {
	let utf8_left = std::str::from_utf8(left);
	let utf8_right = std::str::from_utf8(right);
	match (utf8_left, utf8_right) {
		(Ok(l), Ok(r)) => natsort(deunicode(l).as_bytes(), deunicode(r).as_bytes(), insensitive),
		(Ok(l), Err(_)) => natsort(deunicode(l).as_bytes(), right, insensitive),
		(Err(_), Ok(r)) => natsort(left, deunicode(r).as_bytes(), insensitive),
		_ => natsort(left, right, insensitive),
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn cmp(left: &[&str]) {
		let mut right = left.to_vec();
		right.sort_by(|a, b| natsort(a.as_bytes(), b.as_bytes(), true));
		assert_eq!(left, right);
	}

	fn cmp_deun(left: &[&str]) {
		let mut right = left.to_vec();
		right.sort_by(|a, b| deunicode_natsort(a.as_bytes(), b.as_bytes(), true));
		assert_eq!(left, right);
	}

	#[test]
	fn test_natsort() {
		let dates = vec!["1999-3-3", "1999-12-25", "2000-1-2", "2000-1-10", "2000-3-23"];
		let fractions = vec![
			"1.002.01", "1.002.03", "1.002.08", "1.009.02", "1.009.10", "1.009.20", "1.010.12",
			"1.011.02",
		];
		let words = vec![
			"1-02",
			"1-2",
			"1-20",
			"10-20",
			"fred",
			"jane",
			"pic01",
			"pic02",
			"pic02a",
			"pic02000",
			"pic05",
			"pic2",
			"pic3",
			"pic4",
			"pic 4 else",
			"pic 5",
			"pic 5 ",
			"pic 5 something",
			"pic 6",
			"pic   7",
			"pic100",
			"pic100a",
			"pic120",
			"pic121",
			"tom",
			"x2-g8",
			"x2-y08",
			"x2-y7",
			"x8-y8",
		];

		cmp(&dates);
		cmp(&fractions);
		cmp(&words);
	}

	#[test]
	fn test_deunicode_natsort() {
		let deun = vec![
			"Acorn",
			"Ætt",
			"Alpha",
			"Átriu",
			"Attention",
			"Beta",
			"Elemér",
			"Érvényes",
			"Große",
			"Grotto",
			"Hedvig",
			"Ilona",
			"Írott",
			"Olga",
			"Órmotlan",
			"Öveges",
			"Őzike",
			"Ubul",
			"Űr",
			"Útvonal",
			"Üveg",
		];

		cmp_deun(&deun)
	}
}
