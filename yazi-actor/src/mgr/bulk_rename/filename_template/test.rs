use super::*;

#[test]
fn test_new_creates_default_builder() {
	let builder = CounterBuilder::new();
	assert_eq!(builder.format(), CounterFormat::Digits(Digits));
	assert_eq!(builder.start(), None);
	assert_eq!(builder.step(), None);
	assert_eq!(builder.width(), None);
}

#[test]
fn test_try_set_format() {
	let builder = CounterBuilder::new();

	// Test all supported formats
	let formats = [
		("D", "  D  ", CounterFormat::Digits(Digits)),
		("d", "  d  ", CounterFormat::Digits(Digits)),
		("N", "  N  ", CounterFormat::Digits(Digits)),
		("n", "  n  ", CounterFormat::Digits(Digits)),
		("A", "  A  ", CounterFormat::AnsiUpper(AnsiUpper)),
		("a", "  a  ", CounterFormat::AnsiLower(AnsiLower)),
		("R", "  R  ", CounterFormat::RomanUpper(RomanUpper)),
		("r", "  r  ", CounterFormat::RomanLower(RomanLower)),
		("C", "  C  ", CounterFormat::CyrillicUpper(CyrillicUpper)),
		("c", "  c  ", CounterFormat::CyrillicLower(CyrillicLower)),
	];

	for (without_space, with_space, expected) in formats.iter() {
		let result = builder.try_set_format(without_space, 0..1, without_space);
		assert_eq!(result.unwrap().format(), *expected,);
		let result = builder.try_set_format(with_space, 0..5, with_space);
		assert_eq!(result.unwrap().format(), *expected,);
	}

	let result = builder.try_set_format("", 0..0, "");
	let error = result.unwrap_err();
	assert_eq!(error.reason, "Empty counter kind");
	assert_eq!(error.span, 0..0);
	assert_eq!(error.expected, Some("one of D, d, N, n, A, a, R, r, C, c"));
	assert_eq!(error.found, None);

	let result =
		CounterBuilder::new().try_set_format("  Ü-Wagen as examplé  ", 2..27, "  Ü-Wagen as examplé  ");
	let error = result.unwrap_err();
	assert_eq!(error.reason, "Unexpected counter kind");
	assert_eq!(error.span, 4..25);
	assert_eq!(error.expected, Some("one of D, d, N, n, A, a, R, r, C, c"));
	assert_eq!(error.found, Some("Ü-Wagen as examplé"));
}

#[test]
fn test_try_set_start() {
	let builder = CounterBuilder::new();
	let result = builder.try_set_start("n,5", Some((2..3, "5"))).unwrap();
	assert_eq!(result.start(), Some(5));

	let formats = [
		("D", "25", 25),
		("d", "25", 25),
		("N", "25", 25),
		("n", "25", 25),
		("A", "AB", 28),
		("a", "ab", 28),
		("R", "IV", 4),
		("r", "iv", 4),
		("C", "АБ", 30),
		("c", "аб", 30),
	];

	for (format, start, expected) in formats.iter() {
		let builder = builder.try_set_format(format, 0..1, format).unwrap();
		let result = builder.try_set_start(format, Some((2..4, start))).unwrap();
		assert_eq!(result.start(), Some(*expected));
	}

	let result = builder.try_set_start("_", Some((2..3, "_"))).unwrap();
	assert_eq!(result.start(), None);

	let result = builder.try_set_start("  5  ", Some((0..5, "  5  "))).unwrap();
	assert_eq!(result.start(), Some(5));

	let result =
		builder.try_set_start("  Ü-Wagen as examplé  ", Some((2..27, "  Ü-Wagen as examplé  ")));
	let error = result.unwrap_err();
	assert_eq!(error.span, 4..25);
	assert_eq!(error.expected, Some("digit"));
	assert_eq!(error.found, Some("Ü-Wagen as examplé"));
}

#[test]
fn test_try_set_step() {
	let builder = CounterBuilder::new();
	let result = builder.try_set_step("5", Some((2..3, "5"))).unwrap();
	assert_eq!(result.step(), Some(5));

	let result = builder.try_set_step("_", Some((2..3, "_"))).unwrap();
	assert_eq!(result.step(), None);

	let result = builder.try_set_step("  5  ", Some((0..5, "  5  "))).unwrap();
	assert_eq!(result.step(), Some(5));

	let result =
		builder.try_set_step("  Ü-Wagen as examplé  ", Some((2..27, "  Ü-Wagen as examplé  ")));
	let error = result.unwrap_err();
	assert_eq!(error.span, 4..25);
	assert_eq!(error.expected, Some("digit"));
	assert_eq!(error.found, Some("Ü-Wagen as examplé"));
}

#[test]
fn test_try_set_width() {
	let builder = CounterBuilder::new();
	let result = builder.try_set_width("5", Some((2..3, "5"))).unwrap();
	assert_eq!(result.width(), Some(5));

	let result = builder.try_set_width("_", Some((2..3, "_"))).unwrap();
	assert_eq!(result.width(), None);

	let result = builder.try_set_width("  5  ", Some((0..5, "  5  "))).unwrap();
	assert_eq!(result.width(), Some(5));

	let result =
		builder.try_set_width("  Ü-Wagen as examplé  ", Some((2..27, "  Ü-Wagen as examplé  ")));
	let error = result.unwrap_err();
	assert_eq!(error.span, 4..25);
	assert_eq!(error.expected, Some("digit"));
	assert_eq!(error.found, Some("Ü-Wagen as examplé"));
}

#[test]
fn test_build_with_all_parameters() {
	let builder = CounterBuilder::new()
		.try_set_format("N,10,2,3", 0..1, "N")
		.unwrap()
		.try_set_start("N,10,2,3", Some((2..4, "10")))
		.unwrap()
		.try_set_step("N,10,2,3", Some((5..6, "2")))
		.unwrap()
		.try_set_width("N,10,2,3", Some((7..8, "3")))
		.unwrap();
	let counter = builder.build();
	let mut buf = String::new();
	counter.write_value(&mut buf).unwrap();
	assert_eq!(buf, "010"); // Digits format, width 3
}

#[test]
fn test_template_parse_no_counters() {
	match Template::parse("plain text without counters") {
		Err(TemplateError::NotCounter) => {}
		_ => panic!("Expected NotCounter error"),
	}
}

#[test]
fn test_template_parse_counters() {
	use super::{CounterFormat as CF, TemplatePart as TP};

	// test includes escaped counters %%{R,3,4,5}
	let inputs: [(&str, CF, CF, CF, CF, CF, Option<u32>, Option<u32>, Option<usize>); 8] = [
		(
			"Ü-%%{R,3,4,5}_%{D}_examplé_%{d}_你好_%{N}_слово_%{n}_word_%{A}.txt",
			CF::Digits(Digits),
			CF::Digits(Digits),
			CF::Digits(Digits),
			CF::Digits(Digits),
			CF::AnsiUpper(AnsiUpper),
			None,
			None,
			None,
		),
		(
			"Ü-%%{R,3,4,5}_%{a}_examplé_%{R}_你好_%{r}_слово_%{C}_word_%{c}.txt",
			CF::AnsiLower(AnsiLower),
			CF::RomanUpper(RomanUpper),
			CF::RomanLower(RomanLower),
			CF::CyrillicUpper(CyrillicUpper),
			CF::CyrillicLower(CyrillicLower),
			None,
			None,
			None,
		),
		(
			"Ü-%%{R,3,4,5}_%{D,3}_examplé_%{d,3}_你好_%{N,3}_слово_%{n,3}_word_%{A,3}.txt",
			CF::Digits(Digits),
			CF::Digits(Digits),
			CF::Digits(Digits),
			CF::Digits(Digits),
			CF::AnsiUpper(AnsiUpper),
			Some(3),
			None,
			None,
		),
		(
			"Ü-%%{R,3,4,5}_%{a,c}_examplé_%{R,3}_你好_%{r,iii}_слово_%{C,3}_word_%{c,в}.txt",
			CF::AnsiLower(AnsiLower),
			CF::RomanUpper(RomanUpper),
			CF::RomanLower(RomanLower),
			CF::CyrillicUpper(CyrillicUpper),
			CF::CyrillicLower(CyrillicLower),
			Some(3),
			None,
			None,
		),
		(
			"Ü-%%{R,3,4,5}_%{D,3,4}_examplé_%{d,3,4}_你好_%{N,3,4}_слово_%{n,3,4}_word_%{A,3,4}.txt",
			CF::Digits(Digits),
			CF::Digits(Digits),
			CF::Digits(Digits),
			CF::Digits(Digits),
			CF::AnsiUpper(AnsiUpper),
			Some(3),
			Some(4),
			None,
		),
		(
			"Ü-%%{R,3,4,5}_%{a,c,14}_examplé_%{R,3,14}_你好_%{r,iii,14}_слово_%{C,3,14}_word_%{c,в,14}.txt",
			CF::AnsiLower(AnsiLower),
			CF::RomanUpper(RomanUpper),
			CF::RomanLower(RomanLower),
			CF::CyrillicUpper(CyrillicUpper),
			CF::CyrillicLower(CyrillicLower),
			Some(3),
			Some(14),
			None,
		),
		(
			"Ü-%%{R,3,4,5}_%{D,3,4,55}_examplé_%{d,3,4,55}_你好_%{N,3,4,55}_слово_%{n,3,4,55}_word_%{A,3,4,55}.txt",
			CF::Digits(Digits),
			CF::Digits(Digits),
			CF::Digits(Digits),
			CF::Digits(Digits),
			CF::AnsiUpper(AnsiUpper),
			Some(3),
			Some(4),
			Some(55),
		),
		(
			"Ü-%%{R,3,4,5}_%{a,c,14,6}_examplé_%{R,3,14,6}_你好_%{r,iii,14,6}_слово_%{C,3,14,6}_word_%{c,в,14,6}.txt",
			CF::AnsiLower(AnsiLower),
			CF::RomanUpper(RomanUpper),
			CF::RomanLower(RomanLower),
			CF::CyrillicUpper(CyrillicUpper),
			CF::CyrillicLower(CyrillicLower),
			Some(3),
			Some(14),
			Some(6),
		),
	];

	for (idx, &(input, c1, c2, c3, c4, c5, start, step, width)) in inputs.iter().enumerate() {
		let parsed = Template::parse(input).expect("Should parse a single counter");
		assert_eq!(parsed.counter_count(), 5);

		assert_eq!(
			parsed.parts(),
			&[
				TP::Text("Ü-"),
				TP::Text("%{"),
				TP::Text("R,3,4,5}_"),
				TP::CounterBuilder(CounterBuilder { format: c1, start, step, width }),
				TP::Text("_example\u{301}_"),
				TP::CounterBuilder(CounterBuilder { format: c2, start, step, width }),
				TP::Text("_你好_"),
				TP::CounterBuilder(CounterBuilder { format: c3, start, step, width }),
				TP::Text("_слово_"),
				TP::CounterBuilder(CounterBuilder { format: c4, start, step, width }),
				TP::Text("_word_"),
				TP::CounterBuilder(CounterBuilder { format: c5, start, step, width }),
				TP::Text(".txt")
			],
			"Failed to pass {} index",
			idx
		);
	}
}

#[test]
fn test_template_parse_unclosed_delimiter() {
	let input = "file_%{N.txt";
	match Template::parse(input) {
		Err(TemplateError::Parse(parse_err)) => {
			assert_eq!(parse_err.reason, "Unclosed delimiter");
			assert_eq!(parse_err.expected, Some("}"));
		}
		_ => panic!("Expected 'unclosed delimiter'."),
	}
}

#[test]
fn test_template_parse_extra_commas() {
	let input = "some %{N,2,2,2,} text";
	let result = Template::parse(input);
	match result {
		Err(TemplateError::Parse(parse_err)) => {
			assert_eq!(
				parse_err,
				ParseError {
					input: "some %{N,2,2,2,} text",
					span: 14..15,
					reason: "Extra arguments",
					expected: Some("no additional arguments"),
					found: None,
				}
			);
		}
		Ok(_) => panic!("Expected TemplateError::Parse"),
		Err(_) => panic!("Expected TemplateError::Parse"),
	}
}
