// cargo test --package config --lib -- exec::tests::build --exact --nocapture

#[test]
fn parse() {
	use crate::exec::Exec;

	fn assert(a: &str) {
		let exec = Exec::parse(a);
		println!("{:?}", exec);

		let a = a.trim();
		let b = Exec::from(a).to_string();

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

	assert(r#"sh -c 'exiftool $0; echo \"\n\nPress enter to exit\"; read'"#)
}

#[test]
fn build() {
	use crate::exec::Exec;

	fn assert(s: &str, args: Vec<String>, expected: &str) {
		let exec = Exec::parse(s);
		println!("{:?}", exec);

		let got = Exec::from(s).build(args);
		if got != expected {
			println!("A: {}", expected);
			println!("B: {}", got);
		}
	}

	assert(
		r#"sh -c 'exiftool $0 "$1" \'$2\'; echo "\n\nPress enter to exit"; read'"#,
		vec!["fo o".into(), "b'a\"r".into(), "b\"a'z".into()],
		r#"sh -c 'exiftool foo; echo "\n\nPress enter to exit"; read'"#,
	);
}
