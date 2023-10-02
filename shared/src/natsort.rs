// A natural sort implementation in Rust.
// Copyright (c) 2023, sxyazi.
//
// This is a port of the C version of Martin Pool's `strnatcmp.c`:
// http://sourcefrog.net/projects/natsort/

use std::cmp::Ordering;

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

pub fn natsort(left: &str, right: &str, insensitive: bool) -> Ordering {
	let left = left.as_bytes();
	let right = right.as_bytes();

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

#[cfg(test)]
mod tests {
	use super::*;

	fn cmp(left: &[&str]) {
		let mut right = left.to_vec();
		right.sort_by(|a, b| natsort(a, b, true));
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

	// #[test]
	// fn test_bench() {
	// 	use std::time::Instant;
	//
	// 	let files = vec![
	// 		"pexels-asad-photo-maldives-1024967.jpg",
	// 		"154586 (540p).mp4",
	// 		"163333 (1080p).mp4",
	// 		"166808 (540p).mp4",
	// 		"178732 (1080p).mp4",
	// 		"archive",
	// 		"file.rs",
	// 		"no copyright.pdf",
	// 		"pexels-alex-fu-1302436.jpg",
	// 		"pexels-alexander-grey-1191710.jpg",
	// 		"pexels-benjamin-suter-2362002.jpg",
	// 		"pexels-blaque-x-863963.jpg",
	// 		"pexels-brakou-abdelghani-1723637.jpg",
	// 		"pexels-chevanon-photography-1335971.jpg",
	// 		"pexels-craig-adderley-1563356.jpg",
	// 		"pexels-danne-516541.jpg",
	// 		"pexels-eberhard-grossgasteiger-443446.jpg",
	// 		"pexels-egil-sjøholt-1906658.jpg",
	// 		"pexels-felix-mittermeier-2832041.jpg",
	// 		"pexels-gabriel-peter-719396.jpg",
	// 		"pexels-james-wheeler-1519088.jpg",
	// 		"pexels-jonas-kakaroto-736230.jpg",
	// 		"pexels-katie-burandt-1212693.jpg",
	// 		"pexels-marta-branco-1173576.jpg",
	// 		"pexels-matthew-montrone-1324803.jpg",
	// 		"pexels-max-andrey-1366630.jpg",
	// 		"pexels-nick-collins-1266741.jpg",
	// 		"pexels-oliver-sjöström-1433052.jpg",
	// 		"pexels-photomix-company-1002725.jpg",
	// 		"pexels-pixabay-15239.jpg",
	// 		"pexels-pixabay-33045.jpg",
	// 		"pexels-pixabay-33101.jpg",
	// 		"pexels-pixabay-33109.jpg",
	// 		"pexels-pixabay-36717.jpg",
	// 		"pexels-pixabay-36729.jpg",
	// 		"pexels-pixabay-36762.jpg",
	// 		"pexels-pixabay-45911.jpg",
	// 		"pexels-pixabay-47334.jpg",
	// 		"pexels-pixabay-50594.jpg",
	// 		"pexels-pixabay-59990.jpg",
	// 		"pexels-pixabay-60597.jpg",
	// 		"pexels-pixabay-68507.jpg",
	// 		"pexels-pixabay-158536.jpg",
	// 		"pexels-pixabay-207088.jpg",
	// 		"pexels-pixabay-327509.jpg",
	// 		"pexels-pixabay-358457.jpg",
	// 		"pexels-pixabay-372166.jpg",
	// 		"pexels-pixabay-459203.jpg",
	// 		"pexels-sevenstorm-juhaszimrus-891030.jpg",
	// 		"pexels-steve-johnson-1266808.jpg",
	// 		"pexels-suneo-103573.jpg",
	// 		"pexels-tetyana-kovyrina-937980.jpg",
	// 		"pexels-valeria-boltneva-1484657.jpg",
	// 		"pexels-vlad-chețan-2604929.jpg",
	// 		"pexels-wang-teck-heng-117139.jpg",
	// 		"pexels-yuliya-strizhkina-1198802.jpg",
	// 		"precache.rs",
	// 		"scheduler.rs",
	// 		"Symbols-0.73.0-x64.zip",
	// 		"tasks.rs",
	// 	];
	//
	// 	{
	// 		let mut large1 = files.repeat(2000);
	// 		let mut large2 = files.repeat(2000);
	//
	// 		let now = Instant::now();
	// 		large1.sort_unstable_by(|a, b| natord::compare_ignore_case(a, b));
	// 		println!("natord crate (insensitive) - Elapsed: {:.2?}", now.elapsed());
	//
	// 		let now = Instant::now();
	// 		large2.sort_unstable_by(|a, b| natsort(a, b, true));
	// 		println!("Yazi (insensitive) - Elapsed: {:.2?}", now.elapsed());
	// 	}
	// 	println!();
	// 	{
	// 		let mut large1 = files.repeat(2000);
	// 		let mut large2 = files.repeat(2000);
	//
	// 		let now = Instant::now();
	// 		large1.sort_unstable_by(|a, b| natord::compare(a, b));
	// 		println!("natord crate (sensitive) - Elapsed: {:.2?}", now.elapsed());
	//
	// 		let now = Instant::now();
	// 		large2.sort_unstable_by(|a, b| natsort(a, b, false));
	// 		println!("Yazi (sensitive) - Elapsed: {:.2?}", now.elapsed());
	// 	}
	// }
}
